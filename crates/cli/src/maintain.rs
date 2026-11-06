//! Keeping an app current: `update`, `doctor` and `remove --unused`.

use super::*;

const REPOSITORY: &str = "https://raw.githubusercontent.com/karyalabs/gpuicn";
/// Where the repository lived before it moved to karyalabs; apps installed
/// then still name it in `gpuicn.toml`.
const OLD_REPOSITORY: &str = "https://raw.githubusercontent.com/devaryakjha/gpuicn";
const LATEST_RELEASE: &str = "https://api.github.com/repos/karyalabs/gpuicn/releases/latest";
const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The release tag a default-shaped registry URL points at.
pub(super) fn registry_version(registry: &str) -> Option<&str> {
    let rest = registry
        .strip_prefix(REPOSITORY)
        .or_else(|| registry.strip_prefix(OLD_REPOSITORY))?
        .strip_prefix("/v")?;
    let version = rest.split('/').next()?;
    (!version.is_empty()).then_some(version)
}

/// A release file for `version`: under `GPUICN_RELEASE` (a checkout) when set.
fn release_location_for(version: &str, path: &str) -> String {
    match std::env::var("GPUICN_RELEASE") {
        Ok(base) => format!("{}/{path}", base.trim_end_matches('/')),
        Err(_) => format!("{REPOSITORY}/v{version}/{path}"),
    }
}

fn latest_version(client: &mut Option<reqwest::blocking::Client>) -> Result<String> {
    if std::env::var_os("GPUICN_RELEASE").is_some() {
        return Ok(CLI_VERSION.to_owned());
    }
    let client = match client {
        Some(client) => client,
        None => client.insert(http_client()?),
    };
    let response = client
        .get(LATEST_RELEASE)
        .header(
            "User-Agent",
            concat!("gpuicn-cli/", env!("CARGO_PKG_VERSION")),
        )
        .header("Accept", "application/vnd.github+json")
        .send()
        .map_err(reqwest::Error::without_url)?
        .error_for_status()
        .map_err(reqwest::Error::without_url)?;
    let release: serde_json::Value =
        serde_json::from_slice(&read_limited(response, MAX_ITEM_BYTES)?)?;
    let tag = release["tag_name"]
        .as_str()
        .context("latest release has no tag_name")?;
    Ok(tag.trim_start_matches('v').to_owned())
}

/// Replaces the template's dependencies in the manifest (unlike `init`, which
/// keeps existing choices) and reports what changed.
fn replace_cargo_dependencies(manifest: &str, template: &str) -> Result<(String, Vec<String>)> {
    let template: toml_edit::DocumentMut = template.parse()?;
    let mut document: toml_edit::DocumentMut = manifest
        .parse()
        .context("parse Cargo.toml; it was not changed")?;
    let mut changed = Vec::new();
    for (table_name, wanted) in template.iter() {
        let wanted = wanted.as_table().context("dependency template table")?;
        let table = document
            .entry(table_name)
            .or_insert_with(toml_edit::table)
            .as_table_like_mut()
            .with_context(|| format!("Cargo.toml [{table_name}] is not a table"))?;
        for (key, item) in wanted.iter() {
            if table
                .get(key)
                .is_none_or(|existing| dependency_value(existing) != dependency_value(item))
            {
                table.insert(key, item.clone());
                changed.push(if table_name == "dependencies" {
                    key.to_owned()
                } else {
                    format!("{key} ({table_name})")
                });
            }
        }
    }
    Ok((document.to_string(), changed))
}

/// `gpuicn update`: move gpuicn.toml, the Cargo pins and the fonts to a release.
pub(super) fn update(
    root: &Path,
    config_path: &Path,
    mut config: Config,
    to: Option<String>,
    overwrite: bool,
    dry_run: bool,
) -> Result<()> {
    let mut client = None;
    let explicit = to.is_some();
    let target = match to {
        Some(version) => version.trim_start_matches('v').to_owned(),
        None => latest_version(&mut client).context("look up the latest release")?,
    };
    let current = registry_version(&config.registry).map(str::to_owned);
    if !explicit && current.as_deref() == Some(target.as_str()) {
        println!("Already on gpuicn v{target}.");
        return Ok(());
    }
    let verb = if dry_run { "Would write" } else { "Write" };
    match &current {
        Some(_) => {
            config.registry = format!("{REPOSITORY}/v{target}/site/pages/r");
            println!(
                "{verb} {} (registry → v{target}{})",
                config_path.display(),
                current
                    .as_deref()
                    .map_or(String::new(), |c| format!(", was v{c}"))
            );
        }
        None => println!(
            "Keep registry {} in {} (not a release URL)",
            config.registry,
            config_path.display()
        ),
    }

    let template_text = String::from_utf8(release_file(
        &mut client,
        &release_location_for(&target, "crates/cli/templates/dependencies.toml"),
        MAX_ITEM_BYTES,
    )?)?;
    let mut template: toml_edit::DocumentMut = template_text.parse()?;
    if config.mode == Mode::Crate {
        template["dependencies"]["gpuicn"] = toml_edit::value(format!("={target}"));
    }
    let manifest_path = root.join("Cargo.toml");
    if let Some(bytes) = current_file(&manifest_path)? {
        let (text, changed) =
            replace_cargo_dependencies(std::str::from_utf8(&bytes)?, &template.to_string())?;
        if changed.is_empty() {
            println!("Cargo.toml pins already match v{target}.");
        } else {
            println!(
                "{verb} {} (pin {})",
                manifest_path.display(),
                changed.join(", ")
            );
            if !dry_run {
                fs::write(&manifest_path, text)?;
            }
        }
    }

    if !dry_run {
        write_config(config_path, &config)?;
        let fonts = assets_dir(root, &config.output).join("fonts");
        for (file, _) in FONTS {
            let path = fonts.join(file);
            if current_file(&path)?.is_none() {
                let bytes = release_file(
                    &mut client,
                    &release_location_for(&target, &format!("site/assets/fonts/{file}")),
                    MAX_FONT_BYTES,
                )?;
                write_new(&path, &bytes)?;
            }
        }
        write_theme_files(root, &config, false)?;
    }

    let source = Source::new(&config.registry, root)?;
    let output = root.join(&config.output);
    let own = config.mode == Mode::Crate;
    let installed = installed_components(&source, &output)?;
    if !installed.is_empty() {
        println!();
        if overwrite {
            let files = collect_files(&source, &installed, own)?;
            install(&output, files, true, dry_run)?;
        } else {
            status(&source, &output, own)?;
            println!("`gpuicn update --overwrite` replaces installed files with v{target}'s.");
        }
    }
    if target != CLI_VERSION {
        println!(
            "\nThen install the matching CLI: cargo install gpuicn-cli --version {target} --locked --force"
        );
    }
    if !dry_run {
        println!("Run `cargo build`, then `gpuicn doctor`.");
    }
    Ok(())
}

struct Finding {
    text: String,
    fix: Option<String>,
}

struct Report {
    findings: Vec<Finding>,
}

impl Report {
    fn ok(&mut self, text: impl Into<String>) {
        self.findings.push(Finding {
            text: text.into(),
            fix: None,
        });
    }
    fn fix(&mut self, text: impl Into<String>, fix: impl Into<String>) {
        self.findings.push(Finding {
            text: text.into(),
            fix: Some(fix.into()),
        });
    }
}

/// `gpuicn doctor`: checks the app against the CLI's release. Exits 1 on findings.
pub(super) fn doctor(root: &Path, config_path: &Path, config: &Config) -> Result<()> {
    let mut report = Report {
        findings: Vec::new(),
    };
    let output = root.join(&config.output);
    let own = config.mode == Mode::Crate;

    if config.version == 2 {
        report.ok(format!("{} is version 2", config_path.display()));
    } else {
        report.fix(
            format!("{} is version {}", config_path.display(), config.version),
            "run `gpuicn theme set --appearance system` or `gpuicn update` to migrate it",
        );
    }
    match registry_version(&config.registry) {
        Some(version) if version == CLI_VERSION => {
            report.ok(format!("registry is release v{version}, same as this CLI"));
        }
        Some(version) => report.fix(
            format!("registry is release v{version}; this CLI is v{CLI_VERSION}"),
            format!(
                "run `gpuicn update --to {CLI_VERSION}`, or `cargo install gpuicn-cli --version {version} --locked --force`"
            ),
        ),
        None => report.ok(format!("registry is {} (not a release URL)", config.registry)),
    }

    let manifest_path = root.join("Cargo.toml");
    match current_file(&manifest_path)? {
        None => report.fix("no Cargo.toml beside gpuicn.toml", "run `cargo init` here"),
        Some(bytes) => {
            let merged = merge_cargo_dependencies(
                std::str::from_utf8(&bytes)?,
                &dependency_template(config.mode)?,
            )?;
            if merged.added.is_empty() && merged.kept.is_empty() {
                report.ok("Cargo.toml pins match this release");
            } else {
                for name in &merged.added {
                    report.fix(
                        format!("Cargo.toml lacks {name}"),
                        "run `gpuicn init` to add the pinned dependency",
                    );
                }
                for (table, key) in &merged.kept {
                    report.fix(
                        format!("Cargo.toml {table}.{key} differs from this release's pin"),
                        format!("run `gpuicn update --to {CLI_VERSION}` to replace the pins"),
                    );
                }
            }
        }
    }

    let assets = assets_dir(root, &config.output);
    let fonts = assets.join("fonts");
    for (file, sha256) in FONTS {
        match current_file(&fonts.join(file))? {
            None => report.fix(
                format!("{} is missing", fonts.join(file).display()),
                "run `gpuicn init` to download the Geist fonts",
            ),
            Some(bytes) if hex_sha256(&bytes) != sha256 => report.fix(
                format!(
                    "{} is not the pinned Geist file",
                    fonts.join(file).display()
                ),
                "delete it and run `gpuicn init`, or keep your font and set the theme's font names",
            ),
            Some(_) => report.ok(format!(
                "{} is the pinned Geist file",
                fonts.join(file).display()
            )),
        }
    }
    if current_file(&fonts.join("OFL.txt"))?.is_none() {
        report.fix(
            format!("{} is missing", fonts.join("OFL.txt").display()),
            "run `gpuicn init` to add the Geist license",
        );
    }

    let css_path = assets.join("theme.css");
    match current_file(&css_path)? {
        None => report.fix(
            format!("{} is missing", css_path.display()),
            "run `gpuicn theme set --base-color neutral` to generate it",
        ),
        Some(bytes) => match check_theme_css(std::str::from_utf8(&bytes)?) {
            Ok(()) => report.ok(format!("{} is a complete shadcn theme", css_path.display())),
            Err(error) => report.fix(
                format!("{}: {error:#}", css_path.display()),
                "fix the CSS or run `gpuicn theme set --base-color neutral`",
            ),
        },
    }
    let app_theme = output.join("app_theme.rs");
    match current_file(&app_theme)? {
        None => report.fix(
            format!("{} is missing", app_theme.display()),
            "run `gpuicn theme set --appearance system` to generate it",
        ),
        Some(bytes) => {
            let text = String::from_utf8_lossy(&bytes);
            let include = text
                .split("include_str!(\"")
                .nth(1)
                .and_then(|rest| rest.split('"').next());
            match include {
                Some(relative) if output.join(relative).is_file() => {
                    report.ok(format!("{} includes {relative}", app_theme.display()))
                }
                Some(relative) => report.fix(
                    format!(
                        "{} includes {relative}, which does not exist",
                        app_theme.display()
                    ),
                    "run `gpuicn theme set --appearance system` to regenerate it",
                ),
                None => report.ok(format!("{} is yours", app_theme.display())),
            }
        }
    }

    let main_path = root.join("src/main.rs");
    match current_file(&main_path)? {
        None => report.ok("no src/main.rs (library crate)"),
        Some(bytes) => {
            let text = String::from_utf8_lossy(&bytes);
            if !text.contains("mod ui;") {
                report.fix(
                    "src/main.rs does not declare `mod ui;`",
                    "add `mod ui;` so the installed components compile",
                );
            } else if text.contains("AppConfig::new(") {
                report.ok("src/main.rs starts through `ui::app::run`");
            } else if text.contains("theme::init(") {
                report.ok("src/main.rs calls `ui::theme::init`");
                if !text.contains("LucideAssetSource") {
                    report.fix(
                        "src/main.rs does not register `LucideAssetSource`",
                        "pass `.with_assets(LucideAssetSource)` to the application, or start through `ui::app::run`",
                    );
                }
            } else {
                report.fix(
                    "src/main.rs neither calls `ui::app::run` nor `ui::theme::init`",
                    "start through `ui::app::run(AppConfig::new(..), |_, _| Root)` or call `ui::theme::init(cx)` once",
                );
            }
        }
    }

    let source = Source::new(&config.registry, root)?;
    if let Ok(installed) = installed_components(&source, &output) {
        let mut behind = Vec::new();
        let mut missing = Vec::new();
        for name in &installed {
            for (module, content) in collect_files(&source, std::slice::from_ref(name), own)? {
                match current_file(&output.join(format!("{module}.rs")))? {
                    Some(local) if local == content.as_bytes() => {}
                    Some(_) => behind.push(name.clone()),
                    None => missing.push(name.clone()),
                }
            }
        }
        behind.dedup();
        missing.dedup();
        if installed.is_empty() {
            report.ok("no components installed yet");
        } else if behind.is_empty() && missing.is_empty() {
            report.ok(format!(
                "{} installed component(s) match the registry",
                installed.len()
            ));
        }
        if !behind.is_empty() {
            report.fix(
                format!(
                    "{} differ from the registry: {}",
                    if own { "ejected components" } else { "components" },
                    behind.join(", ")
                ),
                "`gpuicn diff <component>` shows the changes; `gpuicn add <component> --overwrite` takes the registry's",
            );
        }
        if !missing.is_empty() {
            report.fix(
                format!(
                    "components are missing shared files: {}",
                    missing.join(", ")
                ),
                format!(
                    "run `gpuicn {} <component>` again",
                    if own { "eject" } else { "add" }
                ),
            );
        }
    }

    let problems = report.findings.iter().filter(|f| f.fix.is_some()).count();
    for finding in &report.findings {
        match &finding.fix {
            None => println!("ok   {}", finding.text),
            Some(fix) => println!("fix  {}\n     → {fix}", finding.text),
        }
    }
    ensure!(problems == 0, "{problems} finding(s) need attention");
    println!("\nEverything matches gpuicn v{CLI_VERSION}.");
    Ok(())
}

/// Installed components no Rust file outside the output directory refers to.
/// `theme` and `app` are never unused: `app_theme.rs` and the starter need them.
pub(super) fn unused_components(
    root: &Path,
    output: &Path,
    installed: &[String],
) -> Result<Vec<String>> {
    let mut code = String::new();
    let mut stack = vec![root.join("src")];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries {
            let path = entry?.path();
            if path.starts_with(output) {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                code.push_str(&fs::read_to_string(&path)?);
                code.push('\n');
            }
        }
    }
    Ok(installed
        .iter()
        .filter(|name| *name != "theme" && *name != "app")
        .filter(|name| !names_module(&code, &name.replace('-', "_")))
        .cloned()
        .collect())
}

/// Whether `text` uses `module::` or `module;` as a path segment.
fn names_module(text: &str, module: &str) -> bool {
    text.match_indices(module).any(|(start, _)| {
        let before = text[..start].chars().next_back();
        let after = &text[start + module.len()..];
        !before.is_some_and(|c| c.is_alphanumeric() || c == '_')
            && (after.starts_with("::") || after.starts_with(';') || after.starts_with(','))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_urls_name_their_release() {
        assert_eq!(
            registry_version(
                "https://raw.githubusercontent.com/karyalabs/gpuicn/v0.8.0/site/pages/r"
            ),
            Some("0.8.0")
        );
        assert_eq!(
            registry_version(
                "https://raw.githubusercontent.com/karyalabs/gpuicn/v0.9.0-beta.1/site/pages/r"
            ),
            Some("0.9.0-beta.1")
        );
        assert_eq!(
            registry_version(
                "https://raw.githubusercontent.com/devaryakjha/gpuicn/v0.9.7/site/pages/r"
            ),
            Some("0.9.7")
        );
        assert_eq!(registry_version("../../site/pages/r"), None);
    }

    #[test]
    fn pins_are_replaced_and_unused_components_are_found() -> Result<()> {
        let (text, changed) = replace_cargo_dependencies(
            "[package]\nname = \"a\"\n\n[dependencies]\ngpui-icons = \"=0.5.0\"\nserde = \"1\"\n",
            "[dependencies]\ngpui-icons = \"=0.6.0\"\nweb-time = \"1.1\"\n",
        )?;
        assert_eq!(changed, ["gpui-icons", "web-time"]);
        let manifest: toml::Table = toml::from_str(&text)?;
        assert_eq!(
            manifest["dependencies"]["gpui-icons"].as_str(),
            Some("=0.6.0")
        );
        assert_eq!(manifest["dependencies"]["serde"].as_str(), Some("1"));
        let (_, unchanged) =
            replace_cargo_dependencies(&text, "[dependencies]\nweb-time = \"1.1\"\n")?;
        assert!(unchanged.is_empty());

        let temp = tempfile::tempdir()?;
        let root = temp.path();
        fs::create_dir_all(root.join("src/ui"))?;
        fs::create_dir_all(root.join("src/screens"))?;
        fs::write(
            root.join("src/main.rs"),
            "mod ui;\nuse ui::{app::run, button::Button, dialog::*};\n",
        )?;
        fs::write(
            root.join("src/screens/home.rs"),
            "use crate::ui::tabs;\nfn x() { tabs::tabs(); }",
        )?;
        fs::write(root.join("src/ui/select.rs"), "use super::button::Button;")?;
        let installed: Vec<String> = [
            "app", "button", "dialog", "select", "tabs", "theme", "toast",
        ]
        .map(String::from)
        .into();
        assert_eq!(
            unused_components(root, &root.join("src/ui"), &installed)?,
            ["select", "toast"]
        );
        Ok(())
    }
}
