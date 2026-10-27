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
    },
    /// List the components available in the configured registry.
    List,
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
    /// Build an open, shadcn-compatible source registry from registry.toml and registry/.
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

fn main() {
    if let Err(error) = run(Cli::parse()) {
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
        } => {
            let created = current_file(&cli.config)?.is_none();
            if created {
                let config = Config {
                    version: 1,
                    registry: cli.registry.clone().unwrap_or_else(default_registry),
                    output,
                };
                let text = toml::to_string_pretty(&config)?;
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&cli.config)
                    .context("create config (an existing config is never overwritten)")?
                    .write_all(text.as_bytes())?;
                println!("Write {}", cli.config.display());
            } else {
                ensure!(
                    !config_only,
                    "{} already exists; an existing config is never overwritten",
                    cli.config.display()
                );
                println!("Keep {}", cli.config.display());
            }
            if config_only {
                println!(
                    "Run `gpuicn add button` next. Add `mod ui;` to your crate root and call `ui::theme::init(cx)` at application startup."
                );
                return Ok(());
            }
            let config = read_config(&cli.config, cli.registry)?;
            setup(&config_root(&cli.config)?, &config)?;
        }
        Command::Build { source, output } => build(&source, &output)?,
        command => {
            let config = read_config(&cli.config, cli.registry)?;
            let root = config_root(&cli.config)?;
            let source = Source::new(&config.registry, &root)?;
            match command {
                Command::List => {
                    for item in source.index()?.items {
                        println!("{}\t{}", item.name, item.description);
                    }
                }
                Command::Add {
                    mut components,
                    all,
                    overwrite,
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
                            .collect();
                    }
                    ensure!(!components.is_empty(), "name a component or use --all");
                    let components: Vec<_> =
                        components.iter().map(|name| normalize(name)).collect();
                    let files = collect_files(&source, &components)?;
                    let required = required_dependencies(&files);
                    install(&root.join(config.output), files, overwrite, dry_run)?;
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

fn collect_files(source: &Source, components: &[String]) -> Result<BTreeMap<String, String>> {
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
                        modules.remove(&module_name(&path.value())?);
                    }
                }
            }
        }
    }
    Ok(modules)
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
    let manifest_path = root.join("Cargo.toml");
    match current_file(&manifest_path)? {
        Some(bytes) => {
            let merged = merge_cargo_dependencies(std::str::from_utf8(&bytes)?)?;
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
            "No Cargo.toml beside gpuicn.toml. Run `cargo new my-app` first, or add:\n\n{DEPENDENCIES}"
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
        let client = match &mut client {
            Some(client) => client,
            None => client.insert(http_client()?),
        };
        let url = format!("{RELEASE}/site/assets/fonts/{file}");
        let bytes =
            download(client, &url, MAX_FONT_BYTES).with_context(|| format!("fetch {url}"))?;
        ensure!(
            hex_sha256(&bytes) == sha256,
            "{url} does not match its pinned SHA-256"
        );
        write_new(&path, &bytes)?;
    }
    let license = fonts.join("OFL.txt");
    if current_file(&license)?.is_none() {
        let client = match &mut client {
            Some(client) => client,
            None => client.insert(http_client()?),
        };
        let url = format!("{RELEASE}/LICENSES/Geist-OFL-1.1");
        let bytes =
            download(client, &url, MAX_ITEM_BYTES).with_context(|| format!("fetch {url}"))?;
        write_new(&license, &bytes)?;
    }

    let main_path = root.join("src/main.rs");
    let main = current_file(&main_path)?;
    let starter = match &main {
        None => true,
        Some(text) => text == CARGO_NEW_MAIN.as_bytes(),
    };
    if starter {
        let source = Source::new(&config.registry, root)?;
        let files = collect_files(&source, &["button".into()])?;
        install(&root.join(&config.output), files, false, false)?;
        fs::create_dir_all(main_path.parent().context("src directory")?)?;
        fs::write(&main_path, STARTER)?;
        println!("Write {} (starter window)", main_path.display());
        println!(
            "\nReady. Run `cargo run`, then `gpuicn add <component>`; `gpuicn list` shows them all."
        );
    } else if main.as_deref() == Some(STARTER.as_bytes()) {
        println!("\nReady. Run `cargo run`, then `gpuicn add <component>`.");
    } else {
        let text = String::from_utf8_lossy(main.as_deref().unwrap_or_default());
        println!("\nKeep {} (your code).", main_path.display());
        if !text.contains("mod ui;") {
            println!("Add `mod ui;` to your crate root.");
        }
        if !text.contains("theme::init") {
            println!("Call `ui::theme::init(cx)` once at application startup.");
        }
        println!("Then run `gpuicn add button`; `gpuicn list` shows every component.");
    }
    Ok(())
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

/// Adds this release's pinned dependencies without replacing any the app already declares.
fn merge_cargo_dependencies(manifest: &str) -> Result<MergedManifest> {
    let template: toml_edit::DocumentMut = DEPENDENCIES.parse()?;
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
    let available: BTreeSet<_> = fs::read_dir(source.join("registry"))?
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
            let mut paths = fs::read_dir(source.join("registry").join(&dependency))?
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
        let files = collect_files(&source, &["button".into()])?;
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
        assert!(collect_files(&source, &["button".into(), "missing".into()]).is_err());
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
            collect_files(&source, &["alert-dialg".into()])
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
        let merged = merge_cargo_dependencies(fresh)?;
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
        let again = merge_cargo_dependencies(&merged.text)?;
        assert!(again.added.is_empty() && again.kept.is_empty());

        let custom = "[dependencies]\nweb-time = \"1.1.0\"\n\n[dependencies.gpui-kit]\nversion = \"=0.6.4\"\ndefault-features = false\nfeatures = [\"component\"]\n";
        let merged = merge_cargo_dependencies(custom)?;
        assert_eq!(merged.kept, [("dependencies".into(), "web-time".into())]);
        assert!(!merged.added.contains(&"gpui-kit".to_owned()));
        assert!(merged.text.contains("web-time = \"1.1.0\""));
        assert!(merge_cargo_dependencies("not = [valid").is_err());
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
        assert_eq!(deps["gpui-icons"]["rev"], pins["gpui_icons"]["commit"]);
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
