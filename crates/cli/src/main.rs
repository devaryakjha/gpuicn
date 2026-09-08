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

const DEFAULT_REGISTRY: &str = "https://ui.imajha.com/r";
const MAX_ITEM_BYTES: u64 = 4 * 1024 * 1024;

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
    /// Create gpuicn.toml. Does not edit Cargo.toml or your application.
    Init {
        #[arg(long, default_value = "src/ui")]
        output: PathBuf,
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

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Init { output } => {
            let config = Config {
                version: 1,
                registry: cli.registry.unwrap_or_else(|| DEFAULT_REGISTRY.into()),
                output,
            };
            let text = toml::to_string_pretty(&config)?;
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&cli.config)
                .context("create config (an existing config is never overwritten)")?
                .write_all(text.as_bytes())?;
            println!(
                "Created {}. Run `gpuicn add button` next.\nAdd `mod ui;` to your crate root and initialize Base GPUI and UiTheme.\nCargo dependencies, fonts, and application code remain yours; see docs/registry.md.",
                cli.config.display()
            );
        }
        Command::Build { source, output } => build(&source, &output)?,
        command => {
            let mut config: Config = toml::from_str(
                &fs::read_to_string(&cli.config)
                    .context("read gpuicn.toml; run `gpuicn init` first")?,
            )?;
            ensure!(
                config.version == 1,
                "unsupported config version {}",
                config.version
            );
            if let Some(registry) = cli.registry {
                config.registry = registry;
            }
            let root = cli
                .config
                .parent()
                .filter(|path| !path.as_os_str().is_empty())
                .unwrap_or(Path::new("."))
                .canonicalize()?;
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
                    let files = collect_files(&source, &components)?;
                    install(&root.join(config.output), files, overwrite, dry_run)?;
                }
                _ => unreachable!(),
            }
        }
    }
    Ok(())
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
            client: if remote {
                Some(
                    reqwest::blocking::Client::builder()
                        .timeout(Duration::from_secs(30))
                        .redirect(reqwest::redirect::Policy::limited(5))
                        .build()?,
                )
            } else {
                None
            },
        })
    }
    fn read(&self, name: &str) -> Result<Vec<u8>> {
        ensure!(valid_name(name), "invalid component name {name:?}");
        let mut reader: Box<dyn Read> = if let Some(client) = &self.client {
            let response = client
                .get(format!("{}/{name}.json", self.location))
                .send()
                .map_err(reqwest::Error::without_url)?
                .error_for_status()
                .map_err(reqwest::Error::without_url)?;
            Box::new(response)
        } else {
            Box::new(fs::File::open(
                self.root.join(&self.location).join(format!("{name}.json")),
            )?)
        };
        let mut bytes = Vec::new();
        reader
            .by_ref()
            .take(MAX_ITEM_BYTES + 1)
            .read_to_end(&mut bytes)?;
        ensure!(
            bytes.len() as u64 <= MAX_ITEM_BYTES,
            "registry item exceeds 4 MiB"
        );
        Ok(bytes)
    }
    fn index(&self) -> Result<Registry> {
        serde_json::from_slice(&self.read("registry")?).context("parse registry index")
    }
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
        let item: Item = serde_json::from_slice(
            &source
                .read(name)
                .with_context(|| format!("load component {name}"))?,
        )?;
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
                    format!("The shadcn default {title} visual port backed by Base GPUI behavior.")
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
