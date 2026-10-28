//! The `gpuicn` installer: copies editable gpuicn components into GPUI apps.

use anyhow::{Context, Result, ensure};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};
use tempfile::NamedTempFile;

/// Raw files from the release this CLI was built from, so the default registry,
/// fonts, and Cargo pins always match the CLI version.
const RELEASE: &str = concat!(
    "https://raw.githubusercontent.com/devaryakjha/gpuicn/v",
    env!("CARGO_PKG_VERSION")
);
const MAX_ITEM_BYTES: u64 = 4 * 1024 * 1024;
const MAX_FONT_BYTES: u64 = 2 * 1024 * 1024;
/// Cargo dependencies for this release; kept equal to the install fixture by tests.
const DEPENDENCIES: &str = include_str!("../templates/dependencies.toml");
/// The installation starter; kept equal to the install fixture by tests.
const STARTER: &str = include_str!("../templates/main.rs");
/// Component source inside the repository; the published `gpuicn` crate compiles it too.
const REGISTRY_SOURCE: &str = "crates/ui/registry";
/// `src/ui/mod.rs` for crate mode: every component comes from the `gpuicn` crate, and an
/// ejected module declared below the re-export replaces the crate's copy.
const CRATE_MOD: &str = "\
//! gpuicn components, re-exported from the `gpuicn` crate.
//! `gpuicn eject <component>` copies one here to edit; its `pub mod` replaces the crate's.
pub use gpuicn::*;
";
const CARGO_NEW_MAIN: &str = "fn main() {\n    println!(\"Hello, world!\");\n}\n";
/// Geist files and their SHA-256 from release-pins.toml.
const FONTS: [(&str, &str); 3] = [
    (
        "Geist-Regular.ttf",
        "85a1c6b18a6b0a06dfe9fd4f6d6a5d4979f74ec861eaef4bc7868b5492b8a117",
    ),
    (
        "Geist-Medium.ttf",
        "3a3b36f0d0b981f4857f7f00eeef4a5ee123605575d362ab31ee7c19e3d11f2f",
    ),
    (
        "GeistMono-Regular.ttf",
        "5a0de4b3d54ab272f76a1d8c84b7fb24c67bbec6591d5300e61c7bc10094b6c8",
    ),
];

fn default_registry() -> String {
    format!("{RELEASE}/site/pages/r")
}

#[derive(Parser)]
#[command(
    name = "gpuicn",
    version,
    about = "Install editable Rust UI components. No Node.js or runtime service required."
)]
struct Cli {
    /// Configuration path; relative paths inside it resolve beside this file.
    #[arg(long, global = true, default_value = "gpuicn.toml")]
    config: PathBuf,
    /// Override the registry directory (HTTPS URL or local path).
    #[arg(long, global = true)]
    registry: Option<String>,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    /// Set up an app: gpuicn.toml, Cargo dependencies, Geist fonts, and a starter
    /// window when src/main.rs is still the `cargo new` default.
    Init {
        #[arg(long, default_value = "src/ui")]
        output: PathBuf,
        /// Only create gpuicn.toml; leave Cargo.toml, fonts, and src/main.rs alone.
        #[arg(long)]
        config_only: bool,
        /// Depend on the `gpuicn` crate instead of copying source; `eject` copies a
        /// component when you want to edit it.
        #[arg(long = "crate")]
        crate_mode: bool,
    },
    /// Create a new Cargo app and set it up like `init`, ready for `cargo run`.
    New {
        /// Directory for the new app, as for `cargo new`.
        path: PathBuf,
        #[arg(long, default_value = "src/ui")]
        output: PathBuf,
        /// Depend on the `gpuicn` crate instead of copying source.
        #[arg(long = "crate")]
        crate_mode: bool,
    },
    /// List the components available in the configured registry.
    List,
    /// Show which installed components differ from the registry.
    Status,
    /// Show what `add --overwrite` would change, as a unified diff.
    Diff {
        /// Components to compare; defaults to every installed component.
        components: Vec<String>,
    },
    /// Delete components' own source files and their mod.rs declarations.
    /// Files shared with other installed components are kept.
    Remove {
        components: Vec<String>,
        /// Also delete files you have edited.
        #[arg(long)]
        force: bool,
        /// Report changes without deleting files.
        #[arg(long)]
        dry_run: bool,
    },
    /// Copy components and their shared source files. Existing edits are kept by default.
    Add {
        components: Vec<String>,
        #[arg(long)]
        all: bool,
        /// Replace existing source files. Review your local changes first.
        #[arg(long)]
        overwrite: bool,
        /// Report changes without writing files.
        #[arg(long)]
        dry_run: bool,
    },
    /// Copy components out of the `gpuicn` crate so you can edit them (crate mode).
    /// `remove` switches a component back to the crate.
    Eject {
        components: Vec<String>,
        /// Eject every component except the shared theme.
        #[arg(long)]
        all: bool,
        /// Report changes without writing files.
        #[arg(long)]
        dry_run: bool,
    },
    /// Build an open, shadcn-compatible source registry from registry.toml and crates/ui/registry/.
    Build {
        #[arg(long, default_value = ".")]
        source: PathBuf,
        #[arg(long, default_value = "site/pages/r")]
        output: PathBuf,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    version: u32,
    registry: String,
    output: PathBuf,
    #[serde(default, skip_serializing_if = "Mode::is_source")]
    mode: Mode,
}

/// How an app gets components: copied source, or the `gpuicn` crate plus ejected copies.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Mode {
    #[default]
    Source,
    Crate,
}

impl Mode {
    fn is_source(&self) -> bool {
        *self == Mode::Source
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct SourceFile {
    path: String,
    #[serde(rename = "type")]
    kind: String,
    target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
struct Item {
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    schema: Option<String>,
    name: String,
    #[serde(rename = "type")]
    kind: String,
    title: String,
    description: String,
    files: Vec<SourceFile>,
}
#[derive(Serialize, Deserialize)]
struct Registry {
    #[serde(rename = "$schema")]
    schema: String,
    name: String,
    homepage: String,
    items: Vec<Item>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    name: String,
    homepage: String,
    #[serde(default)]
    components: BTreeMap<String, Metadata>,
}
#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Metadata {
    title: Option<String>,
    description: Option<String>,
    dependencies: Vec<String>,
}

/// Runs the CLI with the process arguments and exits on error.
pub fn main() {
    // `cargo gpuicn …` runs `cargo-gpuicn gpuicn …`; drop the subcommand name Cargo repeats.
    let mut args: Vec<_> = std::env::args_os().collect();
    if args.get(1).is_some_and(|arg| arg == "gpuicn")
        && Path::new(&args[0])
            .file_stem()
            .is_some_and(|stem| stem == "cargo-gpuicn")
    {
        args.remove(1);
    }
    if let Err(error) = run(Cli::parse_from(args)) {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn config_root(config: &Path) -> Result<PathBuf> {
    Ok(config
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .canonicalize()?)
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Init {
            output,
            config_only,
            crate_mode,
        } => init(&cli.config, cli.registry, output, config_only, crate_mode)?,
        Command::New {
            path,
            output,
            crate_mode,
        } => {
            ensure!(
                fs::symlink_metadata(&path).is_err(),
                "{} already exists",
                path.display()
            );
            let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
            let status = std::process::Command::new(cargo)
                .arg("new")
                .arg(&path)
                .status()
                .context("run `cargo new`; is Cargo installed?")?;
            ensure!(status.success(), "`cargo new` failed");
            init(
                &path.join("gpuicn.toml"),
                cli.registry,
                output,
                false,
                crate_mode,
            )?;
            println!("Next: cd {} && cargo run", path.display());
        }
        Command::Build { source, output } => build(&source, &output)?,
        command => {
            let config = read_config(&cli.config, cli.registry)?;
            let root = config_root(&cli.config)?;
            let source = Source::new(&config.registry, &root)?;
            let own = config.mode == Mode::Crate;
            let output = root.join(&config.output);
            match command {
                Command::List => {
                    for item in source.index()?.items {
                        println!("{}\t{}", item.name, item.description);
                    }
                }
                Command::Status => status(&source, &output, own)?,
                Command::Diff { components } => diff(&source, &output, &components, own)?,
                Command::Remove {
                    components,
                    force,
                    dry_run,
                } => remove(&source, &output, &components, force, dry_run, own)?,
                Command::Eject {
                    mut components,
                    all,
                    dry_run,
                } => {
                    ensure!(
                        !all || components.is_empty(),
                        "use component names or --all, not both"
                    );
                    if all {
                        components = source
                            .index()?
                            .items
                            .into_iter()
                            .map(|item| item.name)
                            .filter(|name| name != "theme")
                            .collect();
                    }
                    ensure!(
                        own,
                        "eject is for crate mode (`gpuicn init --crate`); in source mode installed components are already in {}",
                        output.display()
                    );
                    eject(&source, &root, &output, &components, dry_run)?
                }
                Command::Add {
                    mut components,
                    all,
                    overwrite,
                    dry_run,
                } => {
                    ensure!(
                        !own,
                        "this app uses the gpuicn crate, so every component is already available as `ui::<component>`; `gpuicn eject <component>` copies one to edit"
                    );
                    ensure!(
                        !all || components.is_empty(),
                        "use component names or --all, not both"
                    );
                    if all {
                        components = source
                            .index()?
                            .items
                            .into_iter()
                            .map(|item| item.name)
                            .collect();
                    }
                    ensure!(!components.is_empty(), "name a component or use --all");
                    let components: Vec<_> =
                        components.iter().map(|name| normalize(name)).collect();
                    let files = collect_files(&source, &components, false)?;
                    let required = required_dependencies(&files);
                    install(&output, files, overwrite, dry_run)?;
                    if !dry_run {
                        report_missing_dependencies(&root, &required)?;
                    }
                }
                _ => unreachable!(),
            }
        }
    }
    Ok(())
}

fn init(
    path: &Path,
    registry: Option<String>,
    output: PathBuf,
    config_only: bool,
    crate_mode: bool,
) -> Result<()> {
    if current_file(path)?.is_none() {
        let config = Config {
            version: 1,
            registry: registry.clone().unwrap_or_else(default_registry),
            output,
            mode: if crate_mode {
                Mode::Crate
            } else {
                Mode::Source
            },
        };
        let text = toml::to_string_pretty(&config)?;
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .context("create config (an existing config is never overwritten)")?
            .write_all(text.as_bytes())?;
        println!("Write {}", path.display());
    } else {
        ensure!(
            !config_only,
            "{} already exists; an existing config is never overwritten",
            path.display()
        );
        println!("Keep {}", path.display());
    }
    if config_only {
        println!(
            "Run `gpuicn add button` next. Add `mod ui;` to your crate root and call `ui::theme::init(cx)` at application startup."
        );
        return Ok(());
    }
    let config = read_config(path, registry)?;
    ensure!(
        !crate_mode || config.mode == Mode::Crate,
        "{} already uses source mode; set `mode = \"crate\"` in it to switch",
        path.display()
    );
    setup(&config_root(path)?, &config)
}

fn read_config(path: &Path, registry: Option<String>) -> Result<Config> {
    let mut config: Config = toml::from_str(
        &fs::read_to_string(path).context("read gpuicn.toml; run `gpuicn init` first")?,
    )?;
    ensure!(
        config.version == 1,
        "unsupported config version {}",
        config.version
    );
    if let Some(registry) = registry {
        config.registry = registry;
    }
    Ok(config)
}

/// Accepts module-style and display-style names: `alert_dialog`, `Alert-Dialog`.
fn normalize(name: &str) -> String {
    name.trim().to_ascii_lowercase().replace('_', "-")
}

/// A registry item that does not exist, as opposed to a network or I/O failure.
#[derive(Debug)]
struct Missing;
impl std::fmt::Display for Missing {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("not found in the registry")
    }
}
impl std::error::Error for Missing {}

fn edit_distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut previous = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let current = row[j + 1];
            row[j + 1] = if ca == *cb {
                previous
            } else {
                1 + previous.min(row[j]).min(current)
            };
            previous = current;
        }
    }
    row[b.len()]
}

/// Names close to an unknown component, best first.
fn suggestions<'a>(name: &str, available: impl Iterator<Item = &'a str>) -> Vec<&'a str> {
    let mut close: Vec<_> = available
        .filter_map(|candidate| {
            let distance = edit_distance(name, candidate);
            (distance <= 2.max(name.len() / 3) || (name.len() >= 3 && candidate.contains(name)))
                .then_some((distance, candidate))
        })
        .collect();
    close.sort();
    close.into_iter().take(3).map(|(_, name)| name).collect()
}

fn unknown_component(source: &Source, name: &str) -> anyhow::Error {
    let hint = match source.index() {
        Ok(index) => {
            let close = suggestions(name, index.items.iter().map(|item| item.name.as_str()));
            if close.is_empty() {
                String::new()
            } else {
                format!("; did you mean {}?", close.join(", "))
            }
        }
        Err(_) => String::new(),
    };
    anyhow::anyhow!(
        "unknown component {name:?}{hint}\nRun `gpuicn list` to see every component in {}",
        source.location
    )
}

struct Source {
    location: String,
    root: PathBuf,
    client: Option<reqwest::blocking::Client>,
}
impl Source {
    fn new(location: &str, root: &Path) -> Result<Self> {
        let remote = location.starts_with("https://") || location.starts_with("http://");
        ensure!(
            remote || !location.contains("://"),
            "registry must be HTTP(S) or a local directory"
        );
        Ok(Self {
            location: location.trim_end_matches('/').to_owned(),
            root: root.to_path_buf(),
            client: if remote { Some(http_client()?) } else { None },
        })
    }
    fn read(&self, name: &str) -> Result<Vec<u8>> {
        ensure!(valid_name(name), "invalid component name {name:?}");
        if let Some(client) = &self.client {
            let url = format!("{}/{name}.json", self.location);
            return download(client, &url, MAX_ITEM_BYTES).with_context(|| format!("fetch {url}"));
        }
        let path = self.root.join(&self.location).join(format!("{name}.json"));
        let file = match fs::File::open(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(anyhow::Error::new(Missing).context(path.display().to_string()));
            }
            Err(error) => return Err(error.into()),
        };
        read_limited(file, MAX_ITEM_BYTES)
    }
    fn index(&self) -> Result<Registry> {
        serde_json::from_slice(&self.read("registry")?).context("parse registry index")
    }
}

fn http_client() -> Result<reqwest::blocking::Client> {
    Ok(reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()?)
}
fn download(client: &reqwest::blocking::Client, url: &str, limit: u64) -> Result<Vec<u8>> {
    let response = client
        .get(url)
        .send()
        .map_err(reqwest::Error::without_url)?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(Missing.into());
    }
    let response = response
        .error_for_status()
        .map_err(reqwest::Error::without_url)?;
    read_limited(response, limit)
}
fn read_limited(reader: impl Read, limit: u64) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.take(limit + 1).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= limit,
        "download exceeds {} MiB",
        limit / 1024 / 1024
    );
    Ok(bytes)
}

fn valid_name(name: &str) -> bool {
    name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && name.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_'
        })
}
fn module_name(path: &str) -> Result<String> {
    ensure!(
        !path.contains('\\')
            && path
                .split('/')
                .all(|part| !part.is_empty() && part != "." && part != ".."),
        "unsafe source path {path:?}"
    );
    let name = path
        .rsplit('/')
        .next()
        .and_then(|part| part.strip_suffix(".rs"))
        .context("only Rust source files can be installed")?;
    ensure!(
        name != "mod"
            && name
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
            && syn::parse_str::<syn::Ident>(name).is_ok(),
        "invalid Rust module name {name:?}"
    );
    Ok(name.to_owned())
}

/// Source modules for `components`. With `own`, only each component's own modules
/// (its root and `#[path]` children), as ejected into a crate-mode app.
fn collect_files(
    source: &Source,
    components: &[String],
    own: bool,
) -> Result<BTreeMap<String, String>> {
    let mut files = BTreeMap::new();
    for name in components.iter().collect::<BTreeSet<_>>() {
        let bytes = match source.read(name) {
            Ok(bytes) => bytes,
            Err(error) if error.downcast_ref::<Missing>().is_some() => {
                return Err(unknown_component(source, name));
            }
            Err(error) => return Err(error.context(format!("load component {name}"))),
        };
        let item: Item = serde_json::from_slice(&bytes)?;
        ensure!(
            &item.name == name,
            "requested {name}, registry returned {}",
            item.name
        );
        ensure!(
            !item.files.is_empty() && item.files.len() <= 256,
            "invalid source-file count for {name}"
        );
        let mut item_files = BTreeMap::new();
        for file in item.files {
            ensure!(
                file.kind == "registry:file",
                "unsupported file kind {}",
                file.kind
            );
            let module = module_name(&file.path)?;
            let content = file
                .content
                .context("registry source has no inline content")?;
            syn::parse_file(&content).with_context(|| format!("parse source module {module}"))?;
            item_files.insert(module, content);
        }
        if own {
            let mut wanted = vec![name.replace('-', "_")];
            let mut owned = BTreeMap::new();
            while let Some(module) = wanted.pop() {
                if let Some(content) = item_files.remove(&module) {
                    wanted.extend(path_children(&content)?);
                    owned.insert(module, content);
                }
            }
            item_files = owned;
        }
        for (module, content) in item_files {
            // The destination is always derived locally; remote `target` never controls filesystem writes.
            if let Some(previous) = files.insert(module.clone(), content.clone()) {
                ensure!(
                    previous == content,
                    "conflicting versions of shared module {module}"
                );
            }
        }
    }
    Ok(files)
}

fn declarations(existing: &str, modules: impl Iterator<Item = String>) -> Result<String> {
    let modules: BTreeSet<_> = modules.collect();
    let parsed =
        syn::parse_file(existing).context("parse existing mod.rs; no source was changed")?;
    let mut declared = BTreeSet::new();
    for item in parsed.items {
        if let syn::Item::Mod(module) = item {
            let name = module.ident.to_string();
            if modules.contains(&name) {
                ensure!(
                    module.content.is_none()
                        && !module
                            .attrs
                            .iter()
                            .any(|attribute| attribute.path().is_ident("path")),
                    "module {name} uses inline content or an alternate path; no source was changed"
                );
            }
            declared.insert(name);
        }
    }
    let mut text = existing.to_owned();
    for module in modules {
        if declared.insert(module.clone()) {
            if !text.is_empty() && !text.ends_with('\n') {
                text.push('\n');
            }
            text.push_str(&format!("pub mod {module};\n"));
        }
    }
    Ok(text)
}
fn root_modules(files: &BTreeMap<String, String>) -> Result<BTreeSet<String>> {
    let mut modules: BTreeSet<_> = files.keys().cloned().collect();
    for content in files.values() {
        for child in path_children(content)? {
            modules.remove(&child);
        }
    }
    Ok(modules)
}
/// Modules a source file declares with `#[path = "..."]`, which live beside it.
fn path_children(content: &str) -> Result<Vec<String>> {
    let mut children = Vec::new();
    for item in syn::parse_file(content)?.items {
        if let syn::Item::Mod(module) = item {
            for attribute in module.attrs {
                if let syn::Meta::NameValue(meta) = attribute.meta
                    && meta.path.is_ident("path")
                    && let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(path),
                        ..
                    }) = meta.value
                {
                    children.push(module_name(&path.value())?);
                }
            }
        }
    }
    Ok(children)
}
fn current_file(path: &Path) -> Result<Option<Vec<u8>>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            ensure!(
                metadata.is_file() && !metadata.file_type().is_symlink(),
                "refusing non-file destination {}",
                path.display()
            );
            Ok(Some(fs::read(path)?))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}
fn install(
    output: &Path,
    files: BTreeMap<String, String>,
    overwrite: bool,
    dry_run: bool,
) -> Result<()> {
    ensure!(!files.is_empty(), "no Rust source to install");
    let mod_path = output.join("mod.rs");
    let existing_mod = current_file(&mod_path)?.unwrap_or_default();
    let updated_mod = declarations(
        std::str::from_utf8(&existing_mod)?,
        root_modules(&files)?.into_iter(),
    )?;
    let mut writes = Vec::new();
    for (module, content) in files {
        let path = output.join(format!("{module}.rs"));
        let existing = current_file(&path)?;
        if existing.as_deref() == Some(content.as_bytes()) {
            continue;
        }
        if existing.is_some() && !overwrite {
            println!("Keep {} (differs; --overwrite replaces it)", path.display());
        } else {
            writes.push((path, content.into_bytes(), existing.is_some()));
        }
    }
    if existing_mod != updated_mod.as_bytes() {
        writes.push((mod_path, updated_mod.into_bytes(), true));
    }
    for (path, _, _) in &writes {
        println!(
            "{} {}",
            if dry_run { "Would write" } else { "Write" },
            path.display()
        );
    }
    if dry_run || writes.is_empty() {
        return Ok(());
    }
    fs::create_dir_all(output)?;
    // Download and validate everything before staging. Persist each file atomically,
    // and publish mod.rs last so failed staging cannot leave unresolved declarations.
    let mut staged = Vec::new();
    for (path, bytes, replace) in writes {
        let mut file = NamedTempFile::new_in(output)?;
        file.write_all(&bytes)?;
        file.flush()?;
        staged.push((file, path, replace));
    }
    for (file, path, replace) in staged {
        if replace {
            file.persist(&path).map_err(|error| error.error)?;
        } else {
            file.persist_noclobber(&path).map_err(|error| error.error)?;
        }
    }
    Ok(())
}

/// Prepares an app beside gpuicn.toml: Cargo pins, fonts, and a starter window.
fn setup(root: &Path, config: &Config) -> Result<()> {
    let template = dependency_template(config.mode)?;
    let manifest_path = root.join("Cargo.toml");
    match current_file(&manifest_path)? {
        Some(bytes) => {
            let merged = merge_cargo_dependencies(std::str::from_utf8(&bytes)?, &template)?;
            for (table, key) in &merged.kept {
                println!(
                    "Keep {table}.{key} in Cargo.toml (differs from gpuicn v{}; mismatched GPUI pins may not compile)",
                    env!("CARGO_PKG_VERSION")
                );
            }
            if !merged.added.is_empty() {
                fs::write(&manifest_path, merged.text)?;
                println!(
                    "Write {} (add {})",
                    manifest_path.display(),
                    merged.added.join(", ")
                );
            }
        }
        None => println!(
            "No Cargo.toml beside gpuicn.toml. Run `cargo new my-app` first, or add:\n\n{template}"
        ),
    }

    let fonts = root.join("assets/fonts");
    let mut client = None;
    for (file, sha256) in FONTS {
        let path = fonts.join(file);
        if let Some(existing) = current_file(&path)? {
            if hex_sha256(&existing) != sha256 {
                println!("Keep {} (differs from Geist)", path.display());
            }
            continue;
        }
        let url = release_location(&format!("site/assets/fonts/{file}"));
        let bytes = release_file(&mut client, &url, MAX_FONT_BYTES)?;
        ensure!(
            hex_sha256(&bytes) == sha256,
            "{url} does not match its pinned SHA-256"
        );
        write_new(&path, &bytes)?;
    }
    let license = fonts.join("OFL.txt");
    if current_file(&license)?.is_none() {
        let url = release_location("LICENSES/Geist-OFL-1.1");
        let bytes = release_file(&mut client, &url, MAX_ITEM_BYTES)?;
        write_new(&license, &bytes)?;
    }

    let main_path = root.join("src/main.rs");
    let main = current_file(&main_path)?;
    let starter = match &main {
        None => true,
        Some(text) => text == CARGO_NEW_MAIN.as_bytes(),
    };
    let next = match config.mode {
        Mode::Source => "`gpuicn add <component>`",
        Mode::Crate => "use any `ui::<component>`; `gpuicn eject <component>` copies one to edit",
    };
    let output = root.join(&config.output);
    if config.mode == Mode::Crate {
        let mod_path = output.join("mod.rs");
        match current_file(&mod_path)? {
            None => {
                write_new(&mod_path, CRATE_MOD.as_bytes())?;
            }
            Some(text) if !String::from_utf8_lossy(&text).contains("pub use gpuicn::*;") => {
                println!(
                    "Keep {} (add `pub use gpuicn::*;` to use the crate's components)",
                    mod_path.display()
                );
            }
            Some(_) => {}
        }
    }
    if starter {
        if config.mode == Mode::Source {
            let source = Source::new(&config.registry, root)?;
            let files = collect_files(&source, &["button".into()], false)?;
            install(&output, files, false, false)?;
        }
        fs::create_dir_all(main_path.parent().context("src directory")?)?;
        fs::write(&main_path, STARTER)?;
        println!("Write {} (starter window)", main_path.display());
        println!("\nReady. Run `cargo run`, then {next}; `gpuicn list` shows them all.");
    } else if main.as_deref() == Some(STARTER.as_bytes()) {
        println!("\nReady. Run `cargo run`, then {next}.");
    } else {
        let text = String::from_utf8_lossy(main.as_deref().unwrap_or_default());
        println!("\nKeep {} (your code).", main_path.display());
        if !text.contains("mod ui;") {
            println!("Add `mod ui;` to your crate root.");
        }
        if !text.contains("theme::init") {
            println!("Call `ui::theme::init(cx)` once at application startup.");
        }
        println!("Then {next}; `gpuicn list` shows every component.");
    }
    Ok(())
}

/// A release file's URL, or its path under `GPUICN_RELEASE` (a checkout), which lets CI
/// set up apps before the release tag exists.
fn release_location(path: &str) -> String {
    let base = std::env::var("GPUICN_RELEASE").unwrap_or_else(|_| RELEASE.to_owned());
    format!("{}/{path}", base.trim_end_matches('/'))
}

fn release_file(
    client: &mut Option<reqwest::blocking::Client>,
    location: &str,
    limit: u64,
) -> Result<Vec<u8>> {
    if !location.starts_with("https://") {
        return fs::read(location).with_context(|| format!("read {location}"));
    }
    let client = match client {
        Some(client) => client,
        None => client.insert(http_client()?),
    };
    download(client, location, limit).with_context(|| format!("fetch {location}"))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    fs::create_dir_all(path.parent().context("parent directory")?)?;
    let mut file = NamedTempFile::new_in(path.parent().context("parent directory")?)?;
    file.write_all(bytes)?;
    file.persist_noclobber(path).map_err(|error| error.error)?;
    println!("Write {}", path.display());
    Ok(())
}

fn hex_sha256(bytes: &[u8]) -> String {
    use sha2::Digest;
    sha2::Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

struct MergedManifest {
    text: String,
    added: Vec<String>,
    kept: Vec<(String, String)>,
}

/// Comparable form of a dependency, ignoring formatting and table style.
fn dependency_value(item: &toml_edit::Item) -> Option<toml::Value> {
    if item.is_table() {
        toml::from_str::<toml::Table>(&item.to_string())
            .ok()
            .map(toml::Value::Table)
    } else {
        toml::from_str::<toml::Table>(&format!("v = {}", item.to_string().trim()))
            .ok()
            .and_then(|mut table| table.remove("v"))
    }
}

/// This release's Cargo pins; crate mode also depends on the `gpuicn` crate itself.
fn dependency_template(mode: Mode) -> Result<String> {
    let mut template: toml_edit::DocumentMut = DEPENDENCIES.parse()?;
    if mode == Mode::Crate {
        template["dependencies"]["gpuicn"] =
            toml_edit::value(concat!("=", env!("CARGO_PKG_VERSION")));
    }
    Ok(template.to_string())
}

/// Adds the template's pinned dependencies without replacing any the app already declares.
fn merge_cargo_dependencies(manifest: &str, template: &str) -> Result<MergedManifest> {
    let template: toml_edit::DocumentMut = template.parse()?;
    let mut document: toml_edit::DocumentMut = manifest
        .parse()
        .context("parse Cargo.toml; it was not changed")?;
    let mut added = Vec::new();
    let mut kept = Vec::new();
    for (table_name, wanted) in template.iter() {
        let wanted = wanted.as_table().context("dependency template table")?;
        let table = document
            .entry(table_name)
            .or_insert_with(toml_edit::table)
            .as_table_like_mut()
            .with_context(|| format!("Cargo.toml [{table_name}] is not a table"))?;
        for (key, item) in wanted.iter() {
            match table.get(key) {
                Some(existing) => {
                    if dependency_value(existing) != dependency_value(item) {
                        kept.push((table_name.to_owned(), key.to_owned()));
                    }
                }
                None => {
                    table.insert(key, item.clone());
                    added.push(if table_name == "dependencies" {
                        key.to_owned()
                    } else {
                        format!("{key} ({table_name})")
                    });
                }
            }
        }
    }
    Ok(MergedManifest {
        text: document.to_string(),
        added,
        kept,
    })
}

/// Cargo dependencies the installed source refers to.
fn required_dependencies(files: &BTreeMap<String, String>) -> BTreeSet<&'static str> {
    let mut required = BTreeSet::from(["gpui-kit"]);
    for content in files.values() {
        if content.contains("gpui_icons") {
            required.insert("gpui-icons");
        }
        if content.contains("web_time") {
            required.insert("web-time");
        }
    }
    required
}

fn report_missing_dependencies(root: &Path, required: &BTreeSet<&str>) -> Result<()> {
    let Some(bytes) = current_file(&root.join("Cargo.toml"))? else {
        return Ok(());
    };
    let Ok(manifest) = std::str::from_utf8(&bytes)?.parse::<toml::Table>() else {
        return Ok(());
    };
    let declared = manifest.get("dependencies").and_then(toml::Value::as_table);
    let missing: Vec<_> = required
        .iter()
        .filter(|name| declared.is_none_or(|table| !table.contains_key(**name)))
        .collect();
    if !missing.is_empty() {
        let template: toml::Table = toml::from_str(DEPENDENCIES)?;
        let pins = template["dependencies"]
            .as_table()
            .context("dependency template")?;
        println!(
            "\nCargo.toml is missing dependencies these components use. Run `gpuicn init`, or add:"
        );
        for name in missing {
            println!("{name} = {}", pins[*name]);
        }
    }
    Ok(())
}

/// Registry components whose own module exists in the output directory.
fn installed_components(source: &Source, output: &Path) -> Result<Vec<String>> {
    let mut installed = Vec::new();
    for item in source.index()?.items {
        if current_file(&output.join(format!("{}.rs", item.name.replace('-', "_"))))?.is_some() {
            installed.push(item.name);
        }
    }
    Ok(installed)
}

/// Normalized names, defaulting to every installed component.
fn targets(source: &Source, output: &Path, components: &[String]) -> Result<Vec<String>> {
    if components.is_empty() {
        let installed = installed_components(source, output)?;
        ensure!(
            !installed.is_empty(),
            "no gpuicn components are installed in {}",
            output.display()
        );
        return Ok(installed);
    }
    Ok(components.iter().map(|name| normalize(name)).collect())
}

fn status(source: &Source, output: &Path, own: bool) -> Result<()> {
    let installed = installed_components(source, output)?;
    if installed.is_empty() {
        println!(
            "No gpuicn components are installed in {}. Run `gpuicn {} button`.",
            output.display(),
            if own { "eject" } else { "add" }
        );
        return Ok(());
    }
    let width = installed.iter().map(String::len).max().unwrap_or(0);
    let mut changed = false;
    for name in &installed {
        let mut modified = Vec::new();
        let mut missing = Vec::new();
        for (module, content) in collect_files(source, std::slice::from_ref(name), own)? {
            match current_file(&output.join(format!("{module}.rs")))? {
                Some(local) if local == content.as_bytes() => {}
                Some(_) => modified.push(format!("{module}.rs")),
                None => missing.push(format!("{module}.rs")),
            }
        }
        let mut notes = Vec::new();
        if !modified.is_empty() {
            notes.push(format!("differs: {}", modified.join(", ")));
        }
        if !missing.is_empty() {
            notes.push(format!("missing: {}", missing.join(", ")));
        }
        changed |= !notes.is_empty();
        println!(
            "{name:width$}  {}",
            if notes.is_empty() {
                "matches registry".to_owned()
            } else {
                notes.join("; ")
            }
        );
    }
    if changed {
        println!(
            "\n`gpuicn diff <component>` shows the changes; `gpuicn add <component> --overwrite` replaces local files."
        );
    }
    Ok(())
}

fn diff(source: &Source, output: &Path, components: &[String], own: bool) -> Result<()> {
    let components = targets(source, output, components)?;
    let mut same = true;
    for (module, content) in collect_files(source, &components, own)? {
        let path = output.join(format!("{module}.rs"));
        let local = current_file(&path)?.unwrap_or_default();
        if local == content.as_bytes() {
            continue;
        }
        same = false;
        let local = String::from_utf8_lossy(&local);
        let display = path.display().to_string();
        print!(
            "{}",
            similar::TextDiff::from_lines(local.as_ref(), content.as_str())
                .unified_diff()
                .context_radius(3)
                .header(
                    &format!("{display} (local)"),
                    &format!("{display} (registry)")
                )
        );
    }
    if same {
        println!("Installed source matches the registry.");
    }
    Ok(())
}

fn remove(
    source: &Source,
    output: &Path,
    components: &[String],
    force: bool,
    dry_run: bool,
    own: bool,
) -> Result<()> {
    ensure!(!components.is_empty(), "name a component to remove");
    let components: Vec<_> = components.iter().map(|name| normalize(name)).collect();
    let installed = installed_components(source, output)?;
    for name in &components {
        ensure!(
            installed.contains(name),
            "{name} is not installed in {}",
            output.display()
        );
    }
    let remaining: Vec<_> = installed
        .into_iter()
        .filter(|name| !components.contains(name))
        .collect();
    let keep = if remaining.is_empty() {
        BTreeMap::new()
    } else {
        collect_files(source, &remaining, own)?
    };
    let mut doomed = Vec::new();
    let mut edited = Vec::new();
    for (module, content) in collect_files(source, &components, own)? {
        if keep.contains_key(&module) {
            continue;
        }
        let path = output.join(format!("{module}.rs"));
        if let Some(local) = current_file(&path)? {
            if local != content.as_bytes() {
                edited.push(path.display().to_string());
            }
            doomed.push((module, path));
        }
    }
    ensure!(
        force || edited.is_empty(),
        "these files have local edits; nothing was removed (--force deletes them anyway):\n  {}",
        edited.join("\n  ")
    );
    let mod_path = output.join("mod.rs");
    let existing_mod = String::from_utf8(current_file(&mod_path)?.unwrap_or_default())?;
    let declarations: BTreeSet<_> = doomed
        .iter()
        .map(|(module, _)| format!("pub mod {module};"))
        .collect();
    let updated_mod: String = existing_mod
        .split_inclusive('\n')
        .filter(|line| !declarations.contains(line.trim()))
        .collect();
    for (_, path) in &doomed {
        println!(
            "{} {}",
            if dry_run { "Would remove" } else { "Remove" },
            path.display()
        );
    }
    if updated_mod != existing_mod {
        println!(
            "{} {}",
            if dry_run { "Would write" } else { "Write" },
            mod_path.display()
        );
    }
    if dry_run {
        return Ok(());
    }
    // Drop declarations first so a failed delete never leaves mod.rs naming a missing file.
    if updated_mod != existing_mod {
        let mut file = NamedTempFile::new_in(output)?;
        file.write_all(updated_mod.as_bytes())?;
        file.persist(&mod_path).map_err(|error| error.error)?;
    }
    for (_, path) in doomed {
        fs::remove_file(path)?;
    }
    Ok(())
}

/// Copies components' own modules into a crate-mode app; their shared modules keep
/// coming from the crate through `pub use gpuicn::*`.
fn eject(
    source: &Source,
    root: &Path,
    output: &Path,
    components: &[String],
    dry_run: bool,
) -> Result<()> {
    ensure!(!components.is_empty(), "name a component to eject");
    let components: Vec<_> = components.iter().map(|name| normalize(name)).collect();
    ensure!(
        !components.iter().any(|name| name == "theme"),
        "theme stays in the crate so every component reads the same theme; customize it with `UiTheme::set` or `UiTheme::from_shadcn_css`"
    );
    let files = collect_files(source, &components, true)?;
    let required = required_dependencies(&files);
    install(output, files, false, dry_run)?;
    if dry_run {
        return Ok(());
    }
    report_missing_dependencies(root, &required)?;
    let index = source.index()?;
    for name in &components {
        let module = name.replace('-', "_");
        let path = format!("registry/{module}/{module}.rs");
        let dependents: Vec<_> = index
            .items
            .iter()
            .filter(|item| {
                !components.contains(&item.name) && item.files.iter().any(|file| file.path == path)
            })
            .map(|item| item.name.as_str())
            .collect();
        println!(
            "\n`ui::{module}` is now {} and yours to edit; `gpuicn remove {name}` goes back to the crate.",
            output.join(format!("{module}.rs")).display()
        );
        if !dependents.is_empty() {
            println!(
                "{} still use the crate's {name}; eject them too if you pass them {name} types.",
                dependents.join(", ")
            );
        }
    }
    Ok(())
}

fn dependencies(
    name: &str,
    manifest: &Manifest,
    available: &BTreeSet<String>,
    visiting: &mut BTreeSet<String>,
    result: &mut Vec<String>,
) -> Result<()> {
    ensure!(available.contains(name), "unknown dependency {name}");
    if result.iter().any(|item| item == name) {
        return Ok(());
    }
    ensure!(visiting.insert(name.into()), "dependency cycle at {name}");
    if name != "theme" {
        dependencies("theme", manifest, available, visiting, result)?;
    }
    if let Some(metadata) = manifest.components.get(name) {
        for dependency in &metadata.dependencies {
            dependencies(dependency, manifest, available, visiting, result)?;
        }
    }
    visiting.remove(name);
    result.push(name.to_owned());
    Ok(())
}
fn json_file(path: &Path, value: &impl Serialize) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(path, bytes)?;
    Ok(())
}
fn build(source: &Path, output: &Path) -> Result<()> {
    let manifest: Manifest = toml::from_str(&fs::read_to_string(source.join("registry.toml"))?)?;
    let components = source.join(REGISTRY_SOURCE);
    let available: BTreeSet<_> = fs::read_dir(&components)?
        .map(|entry| {
            let entry = entry?;
            Ok(entry
                .file_type()?
                .is_dir()
                .then(|| entry.file_name().to_string_lossy().into_owned()))
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect();
    for name in available.iter().chain(manifest.components.keys()) {
        ensure!(
            valid_name(name) && available.contains(name),
            "invalid or missing component {name}"
        );
    }
    let mut items = Vec::new();
    for name in available
        .iter()
        .filter(|name| name.as_str() == "theme")
        .chain(available.iter().filter(|name| name.as_str() != "theme"))
    {
        let mut included = Vec::new();
        dependencies(
            name,
            &manifest,
            &available,
            &mut BTreeSet::new(),
            &mut included,
        )?;
        let mut files = Vec::new();
        let mut names = BTreeSet::new();
        for dependency in included {
            let mut paths = fs::read_dir(components.join(&dependency))?
                .map(|entry| entry.map(|entry| entry.path()))
                .collect::<std::io::Result<Vec<_>>>()?;
            paths.sort();
            for path in paths
                .into_iter()
                .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
            {
                let filename = path
                    .file_name()
                    .context("source filename")?
                    .to_str()
                    .context("non-UTF8 source filename")?;
                let source_path = format!("registry/{dependency}/{filename}");
                let module = module_name(&source_path)?;
                ensure!(names.insert(module.clone()), "duplicate module {module}");
                files.push(SourceFile {
                    path: source_path,
                    kind: "registry:file".into(),
                    target: format!("~/src/ui/{filename}"),
                    content: Some(fs::read_to_string(path)?),
                });
            }
        }
        let metadata = manifest.components.get(name);
        let title = metadata
            .and_then(|meta| meta.title.clone())
            .unwrap_or_else(|| {
                name.split('_')
                    .filter(|part| !part.is_empty())
                    .map(|part| format!("{}{}", part[..1].to_uppercase(), &part[1..]))
                    .collect::<Vec<_>>()
                    .join(" ")
            });
        items.push(Item {
            schema: Some("https://ui.shadcn.com/schema/registry-item.json".into()),
            name: name.replace('_', "-"),
            kind: "registry:item".into(),
            title: title.clone(),
            description: metadata
                .and_then(|meta| meta.description.clone())
                .unwrap_or_else(|| {
                    format!("The shadcn default {title} visual port backed by GPUI Kit behavior.")
                }),
            files,
        });
    }
    fs::create_dir_all(output)?;
    for item in &items {
        json_file(&output.join(format!("{}.json", item.name)), item)?;
    }
    for item in &mut items {
        item.schema = None;
        for file in &mut item.files {
            file.content = None;
        }
    }
    let index = Registry {
        schema: "https://ui.shadcn.com/schema/registry.json".into(),
        name: manifest.name,
        homepage: manifest.homepage,
        items,
    };
    json_file(&source.join("registry.json"), &index)?;
    json_file(&output.join("registry.json"), &index)?;
    println!(
        "Built {} registry items in {}",
        index.items.len(),
        output.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn local_install_keeps_edits_updates_modules_and_rejects_unsafe_input_before_writing()
    -> Result<()> {
        let temp = tempfile::tempdir()?;
        let root = temp.path();
        let registry = root.join("r");
        fs::create_dir(&registry)?;
        let item = Item {
            schema: None,
            name: "button".into(),
            kind: "registry:item".into(),
            title: "Button".into(),
            description: String::new(),
            files: vec![SourceFile {
                path: "registry/button/button.rs".into(),
                kind: "registry:file".into(),
                target: "../../outside.rs".into(),
                content: Some("pub struct Button;\n".into()),
            }],
        };
        json_file(&registry.join("button.json"), &item)?;
        let source = Source::new("r", root)?;
        let files = collect_files(&source, &["button".into()], false)?;
        let output = root.join("src/ui");
        install(&output, files.clone(), false, true)?;
        assert!(!output.exists());
        install(&output, files.clone(), false, false)?;
        assert_eq!(
            fs::read_to_string(output.join("mod.rs"))?,
            "pub mod button;\n"
        );
        assert!(!root.join("outside.rs").exists());
        fs::write(output.join("button.rs"), "// caller edit\n")?;
        fs::write(
            output.join("mod.rs"),
            "// Keep me\npub mod button;\nfn custom() {}\n",
        )?;
        install(&output, files.clone(), false, false)?;
        assert_eq!(
            fs::read_to_string(output.join("button.rs"))?,
            "// caller edit\n"
        );
        assert!(fs::read_to_string(output.join("mod.rs"))?.contains("fn custom() {}"));
        install(&output, files, true, false)?;
        assert_eq!(
            fs::read_to_string(output.join("button.rs"))?,
            "pub struct Button;\n"
        );
        for bad in [
            "../outside.rs",
            "/absolute.rs",
            "registry/../bad.rs",
            "registry\\bad.rs",
            "registry/mod.rs",
            "registry/type.rs",
        ] {
            assert!(module_name(bad).is_err(), "{bad}");
        }
        assert!(collect_files(&source, &["button".into(), "missing".into()], false).is_err());
        assert!(!valid_name("../escape"));
        assert!(declarations("not valid Rust", ["button".into()].into_iter()).is_err());
        assert_eq!(
            root_modules(&BTreeMap::from([
                (
                    "dialog".into(),
                    "#[path = \"modal_focus.rs\"] mod focus;".into()
                ),
                ("modal_focus".into(), "fn helper() {}".into()),
            ]))?,
            BTreeSet::from(["dialog".into()])
        );
        Ok(())
    }
    #[test]
    fn crate_mode_ejects_own_modules_and_pins_the_crate() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let registry = temp.path().join("r");
        fs::create_dir(&registry)?;
        let file = |path: &str, content: &str| SourceFile {
            path: path.into(),
            kind: "registry:file".into(),
            target: String::new(),
            content: Some(content.into()),
        };
        let theme = file("registry/theme/theme.rs", "pub struct UiTheme;\n");
        let dialog = file(
            "registry/dialog/dialog.rs",
            "#[path = \"modal_focus.rs\"]\npub mod modal_focus;\n",
        );
        let focus = file("registry/dialog/modal_focus.rs", "pub fn prepare() {}\n");
        let alert = file(
            "registry/alert_dialog/alert_dialog.rs",
            "pub fn alert() {}\n",
        );
        for (name, files) in [
            ("dialog", vec![theme.clone(), dialog.clone(), focus.clone()]),
            ("alert-dialog", vec![theme, dialog, focus, alert]),
        ] {
            let item = Item {
                schema: None,
                name: name.into(),
                kind: "registry:item".into(),
                title: String::new(),
                description: String::new(),
                files,
            };
            json_file(&registry.join(format!("{name}.json")), &item)?;
        }
        let source = Source::new("r", temp.path())?;
        let modules = |names: &[&str]| -> Result<Vec<String>> {
            let names: Vec<String> = names.iter().map(|name| name.to_string()).collect();
            Ok(collect_files(&source, &names, true)?.into_keys().collect())
        };
        assert_eq!(modules(&["dialog"])?, ["dialog", "modal_focus"]);
        assert_eq!(modules(&["alert-dialog"])?, ["alert_dialog"]);
        assert_eq!(
            collect_files(&source, &["alert-dialog".into()], false)?.len(),
            4
        );

        let pinned: toml::Table = toml::from_str(&dependency_template(Mode::Crate)?)?;
        assert_eq!(
            pinned["dependencies"]["gpuicn"].as_str(),
            Some(concat!("=", env!("CARGO_PKG_VERSION")))
        );
        let copied: toml::Table = toml::from_str(&dependency_template(Mode::Source)?)?;
        assert!(
            !copied["dependencies"]
                .as_table()
                .unwrap()
                .contains_key("gpuicn")
        );

        let config = Config {
            version: 1,
            registry: "r".into(),
            output: "src/ui".into(),
            mode: Mode::Source,
        };
        assert!(!toml::to_string(&config)?.contains("mode"));
        let config: Config = toml::from_str(
            "version = 1\nregistry = \"r\"\noutput = \"src/ui\"\nmode = \"crate\"\n",
        )?;
        assert_eq!(config.mode, Mode::Crate);
        Ok(())
    }
    #[test]
    fn names_normalize_and_unknown_names_get_suggestions() {
        assert_eq!(normalize(" Alert_Dialog "), "alert-dialog");
        let names = ["alert-dialog", "dialog", "button", "checkbox-group"];
        assert_eq!(
            suggestions("alert-dialg", names.into_iter()),
            ["alert-dialog"]
        );
        assert_eq!(suggestions("buton", names.into_iter()), ["button"]);
        assert_eq!(
            suggestions("checkbox", names.into_iter()),
            ["checkbox-group"]
        );
        assert!(suggestions("zzzzzzzz", names.into_iter()).is_empty());
    }
    #[test]
    fn missing_local_component_reports_suggestions() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let registry = temp.path().join("r");
        fs::create_dir(&registry)?;
        let index = Registry {
            schema: String::new(),
            name: "test".into(),
            homepage: String::new(),
            items: vec![Item {
                schema: None,
                name: "alert-dialog".into(),
                kind: "registry:item".into(),
                title: "Alert Dialog".into(),
                description: String::new(),
                files: Vec::new(),
            }],
        };
        json_file(&registry.join("registry.json"), &index)?;
        let source = Source::new("r", temp.path())?;
        let message = format!(
            "{:#}",
            collect_files(&source, &["alert-dialg".into()], false)
                .err()
                .unwrap()
        );
        assert!(message.contains("did you mean alert-dialog?"), "{message}");
        assert!(message.contains("gpuicn list"), "{message}");
        Ok(())
    }
    #[test]
    fn cargo_merge_adds_pins_and_keeps_existing_choices() -> Result<()> {
        let fresh = "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\n";
        let merged = merge_cargo_dependencies(fresh, DEPENDENCIES)?;
        assert_eq!(
            merged.added,
            [
                "web-time",
                "gpui_platform",
                "gpui-icons",
                "gpui-kit",
                "gpui-kit (dev-dependencies)"
            ]
        );
        assert!(merged.kept.is_empty());
        assert!(merged.text.starts_with("[package]\nname = \"app\""));
        let again = merge_cargo_dependencies(&merged.text, DEPENDENCIES)?;
        assert!(again.added.is_empty() && again.kept.is_empty());

        let custom = "[dependencies]\nweb-time = \"1.1.0\"\n\n[dependencies.gpui-kit]\nversion = \"=0.8.1\"\ndefault-features = false\nfeatures = [\"component\"]\n";
        let merged = merge_cargo_dependencies(custom, DEPENDENCIES)?;
        assert_eq!(merged.kept, [("dependencies".into(), "web-time".into())]);
        assert!(!merged.added.contains(&"gpui-kit".to_owned()));
        assert!(merged.text.contains("web-time = \"1.1.0\""));
        assert!(merge_cargo_dependencies("not = [valid", DEPENDENCIES).is_err());
        Ok(())
    }
    #[test]
    fn release_templates_match_the_install_fixture_and_pins() -> Result<()> {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        assert_eq!(
            STARTER,
            fs::read_to_string(repo.join("fixtures/registry-install/src/main.rs"))?
        );
        let fixture: toml::Table = toml::from_str(&fs::read_to_string(
            repo.join("fixtures/registry-install/Cargo.toml"),
        )?)?;
        let template: toml::Table = toml::from_str(DEPENDENCIES)?;
        for table in ["dependencies", "dev-dependencies"] {
            assert_eq!(fixture[table], template[table], "{table}");
        }
        let docs = fs::read_to_string(repo.join("docs/registry.md"))?;
        assert!(
            docs.contains(DEPENDENCIES.trim()),
            "docs/registry.md dependency block"
        );
        let pins: toml::Table =
            toml::from_str(&fs::read_to_string(repo.join("release-pins.toml"))?)?;
        let deps = template["dependencies"].as_table().unwrap();
        assert_eq!(
            deps["gpui_platform"]["version"].as_str(),
            Some(format!("={}", pins["gpui"]["version"].as_str().unwrap()).as_str())
        );
        assert_eq!(
            deps["gpui-kit"]["version"].as_str(),
            Some(format!("={}", pins["gpui_kit"]["version"].as_str().unwrap()).as_str())
        );
        assert_eq!(
            deps["gpui-icons"].as_str(),
            Some(format!("={}", pins["gpui_icons"]["version"].as_str().unwrap()).as_str())
        );
        let geist = pins["geist"].as_table().unwrap();
        for ((_, sha), key) in FONTS.iter().zip([
            "sans_regular_sha256",
            "sans_medium_sha256",
            "mono_regular_sha256",
        ]) {
            assert_eq!(geist[key].as_str(), Some(*sha), "{key}");
        }
        for (file, sha) in FONTS {
            assert_eq!(
                hex_sha256(&fs::read(repo.join("site/assets/fonts").join(file))?),
                sha,
                "{file}"
            );
        }
        Ok(())
    }
    #[test]
    fn remove_keeps_shared_and_edited_files() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let registry = temp.path().join("r");
        fs::create_dir(&registry)?;
        let file = |module: &str| SourceFile {
            path: format!("registry/{module}/{module}.rs"),
            kind: "registry:file".into(),
            target: String::new(),
            content: Some(format!("pub struct {module};\n")),
        };
        let item = |name: &str, modules: &[&str]| Item {
            schema: None,
            name: name.into(),
            kind: "registry:item".into(),
            title: name.into(),
            description: String::new(),
            files: modules.iter().map(|module| file(module)).collect(),
        };
        let items = vec![
            item("theme", &["theme"]),
            item("button", &["theme", "button"]),
            item("menu", &["theme", "button", "menu"]),
        ];
        for item in &items {
            json_file(&registry.join(format!("{}.json", item.name)), item)?;
        }
        json_file(
            &registry.join("registry.json"),
            &Registry {
                schema: String::new(),
                name: "test".into(),
                homepage: String::new(),
                items,
            },
        )?;
        let source = Source::new("r", temp.path())?;
        let output = temp.path().join("ui");
        install(
            &output,
            collect_files(&source, &["menu".into()], false)?,
            false,
            false,
        )?;
        fs::write(
            output.join("mod.rs"),
            format!("// mine\n{}", fs::read_to_string(output.join("mod.rs"))?),
        )?;
        assert_eq!(
            installed_components(&source, &output)?,
            ["theme", "button", "menu"]
        );

        fs::write(output.join("menu.rs"), "// edited\n")?;
        assert!(remove(&source, &output, &["menu".into()], false, false, false).is_err());
        assert!(output.join("menu.rs").exists());
        remove(&source, &output, &["menu".into()], true, true, false)?;
        assert!(output.join("menu.rs").exists());
        remove(&source, &output, &["menu".into()], true, false, false)?;
        assert!(!output.join("menu.rs").exists());
        assert!(output.join("button.rs").exists() && output.join("theme.rs").exists());
        assert_eq!(
            fs::read_to_string(output.join("mod.rs"))?,
            "// mine\npub mod button;\npub mod theme;\n"
        );
        assert!(remove(&source, &output, &["menu".into()], false, false, false).is_err());
        remove(
            &source,
            &output,
            &["button".into(), "theme".into()],
            false,
            false,
            false,
        )?;
        assert_eq!(fs::read_to_string(output.join("mod.rs"))?, "// mine\n");
        Ok(())
    }
    #[test]
    fn dependency_order_is_shared_once_and_cycles_fail() -> Result<()> {
        let manifest: Manifest = toml::from_str(
            "name='test'\nhomepage='local'\n[components.menu]\ndependencies=['button']\n[components.context_menu]\ndependencies=['menu']",
        )?;
        let available = ["theme", "button", "menu", "context_menu"]
            .map(String::from)
            .into_iter()
            .collect();
        let mut result = Vec::new();
        dependencies(
            "context_menu",
            &manifest,
            &available,
            &mut BTreeSet::new(),
            &mut result,
        )?;
        assert_eq!(result, ["theme", "button", "menu", "context_menu"]);
        let cyclic: Manifest = toml::from_str(
            "name='test'\nhomepage='local'\n[components.theme]\ndependencies=['button']",
        )?;
        assert!(
            dependencies(
                "button",
                &cyclic,
                &available,
                &mut BTreeSet::new(),
                &mut Vec::new()
            )
            .is_err()
        );
        Ok(())
    }
}
