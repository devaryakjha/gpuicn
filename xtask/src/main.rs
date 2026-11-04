//! Repository tasks. `cargo xtask new-component <name>` adds a component
//! everywhere the catalog expects one, so the gates pass from the first build.

use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

const USAGE: &str = "\
Usage:
  cargo xtask new-component <snake_name> [--title \"Title\"] [--description \"One sentence.\"]

Creates crates/ui/registry/<name>/<name>.rs, declares it in crates/ui/src/lib.rs
and registry.toml, writes docs/parity/<name>.md, a Usage example in
crates/ui/examples/usage.rs, a catalog preview in site/src/previews/<name>.rs
(listed in site/src/previews/mod.rs and site/src/gallery.rs), a website page
(listed in web/src/lib/catalog.ts), then prints the commands that refresh the
registry snapshot.";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("new-component") => match parse_new_component(&args[1..]) {
            Ok(component) => match new_component(&repo_root(), &component) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("error: {error}");
                    ExitCode::FAILURE
                }
            },
            Err(error) => {
                eprintln!("error: {error}\n\n{USAGE}");
                ExitCode::FAILURE
            }
        },
        _ => {
            eprintln!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives one level below the repository root")
        .to_path_buf()
}

/// A component to scaffold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    /// Module name, such as `date_picker`.
    pub module: String,
    /// Type name, such as `DatePicker`.
    pub type_name: String,
    /// Catalog title, such as `Date Picker`.
    pub title: String,
    /// One-sentence catalog description.
    pub description: String,
}

impl Component {
    pub fn new(
        module: &str,
        title: Option<&str>,
        description: Option<&str>,
    ) -> Result<Self, String> {
        let valid = !module.is_empty()
            && module
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            && module.starts_with(|c: char| c.is_ascii_lowercase())
            && !module.contains("__")
            && !module.ends_with('_');
        if !valid {
            return Err(format!(
                "{module:?} is not a snake_case module name such as date_picker"
            ));
        }
        let words: Vec<String> = module
            .split('_')
            .map(|part| {
                let mut chars = part.chars();
                let first = chars.next().unwrap().to_ascii_uppercase();
                format!("{first}{}", chars.as_str())
            })
            .collect();
        let title = title.map_or_else(|| words.join(" "), str::to_owned);
        Ok(Self {
            module: module.to_owned(),
            type_name: words.concat(),
            title,
            description: description.map_or_else(
                || "Describe the component in one sentence.".to_owned(),
                str::to_owned,
            ),
        })
    }

    /// The registry name, such as `date-picker`.
    pub fn slug(&self) -> String {
        self.module.replace('_', "-")
    }
}

fn parse_new_component(args: &[String]) -> Result<Component, String> {
    let mut module = None;
    let mut title = None;
    let mut description = None;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--title" => title = Some(rest.next().ok_or("--title needs a value")?.as_str()),
            "--description" => {
                description = Some(rest.next().ok_or("--description needs a value")?.as_str())
            }
            flag if flag.starts_with("--") => return Err(format!("unknown flag {flag}")),
            name if module.is_none() => module = Some(name),
            extra => return Err(format!("unexpected argument {extra:?}")),
        }
    }
    Component::new(module.ok_or("name the component")?, title, description)
}

type Edit = fn(&str, &Component) -> Result<String, String>;

fn new_component(root: &Path, component: &Component) -> Result<(), String> {
    let module = &component.module;
    let registry_dir = root.join("crates/ui/registry").join(module);
    if registry_dir.exists() {
        return Err(format!("{} already exists", registry_dir.display()));
    }
    let edits: Vec<(PathBuf, Edit)> = vec![
        (root.join("crates/ui/src/lib.rs"), declare_in_lib),
        (root.join("registry.toml"), declare_in_registry),
        (root.join("crates/ui/examples/usage.rs"), add_usage_example),
        (root.join("site/src/previews/mod.rs"), list_preview),
        (root.join("site/src/gallery.rs"), list_in_gallery),
        (root.join("web/src/lib/catalog.ts"), list_in_site_catalog),
    ];
    // Compute every edit before writing anything, so a failure changes no file.
    let mut writes = Vec::new();
    for (path, edit) in &edits {
        let text = fs::read_to_string(path)
            .map_err(|error| format!("read {}: {error}", path.display()))?;
        let updated =
            edit(&text, component).map_err(|error| format!("{}: {error}", path.display()))?;
        writes.push((path.clone(), updated));
    }
    let new_files = [
        (
            registry_dir.join(format!("{module}.rs")),
            component_source(component),
        ),
        (
            root.join("docs/parity").join(format!("{module}.md")),
            parity_doc(component),
        ),
        (
            root.join("site/src/previews").join(format!("{module}.rs")),
            preview_source(component),
        ),
    ];
    for (path, _) in &new_files {
        if path.exists() {
            return Err(format!("{} already exists", path.display()));
        }
    }
    for (path, text) in writes {
        fs::write(&path, text).map_err(|error| format!("write {}: {error}", path.display()))?;
        println!(
            "Edit  {}",
            path.strip_prefix(root).unwrap_or(&path).display()
        );
    }
    for (path, text) in new_files {
        fs::create_dir_all(path.parent().unwrap()).map_err(|error| error.to_string())?;
        fs::write(&path, text).map_err(|error| format!("write {}: {error}", path.display()))?;
        println!(
            "Write {}",
            path.strip_prefix(root).unwrap_or(&path).display()
        );
    }
    println!(
        "\nNext:\n  1. Build the component in crates/ui/registry/{module}/{module}.rs and its preview in site/src/previews/{module}.rs.\n  2. cargo run -p gpuicn-cli -- build && bun scripts/sync-catalog.mjs\n  3. cargo test --workspace (the registry lint and catalog checks cover the new files)\n  4. Fill docs/parity/{module}.md and the Usage example in crates/ui/examples/usage.rs."
    );
    Ok(())
}

/// Inserts `#[path] pub mod <module>;` in alphabetical order among the registry modules.
pub fn declare_in_lib(text: &str, component: &Component) -> Result<String, String> {
    let module = &component.module;
    let declaration =
        format!("#[path = \"../registry/{module}/{module}.rs\"]\npub mod {module};\n");
    if text.contains(&format!("pub mod {module};")) {
        return Err(format!("already declares {module}"));
    }
    let mut lines: Vec<&str> = text.split_inclusive('\n').collect();
    let mut insert_at = None;
    let mut last_path = None;
    for (index, line) in lines.iter().enumerate() {
        if let Some(rest) = line.strip_prefix("#[path = \"../registry/") {
            let existing = rest.split('/').next().unwrap_or("");
            last_path = Some(index + 2);
            if insert_at.is_none() && existing > module.as_str() {
                insert_at = Some(index);
            }
        }
    }
    let at = insert_at
        .or(last_path)
        .ok_or("found no `#[path = \"../registry/…\"]` module declarations")?;
    lines.insert(at, &declaration);
    Ok(lines.concat())
}

/// Appends `[components.<module>]` with the description.
pub fn declare_in_registry(text: &str, component: &Component) -> Result<String, String> {
    let header = format!("[components.{}]", component.module);
    if text.contains(&header) {
        return Err(format!("already has {header}"));
    }
    let mut text = text.trim_end().to_owned();
    text.push_str(&format!(
        "\n{header}\ndescription = \"{}\"\n",
        component.description.replace('"', "\\\"")
    ));
    Ok(text)
}

/// Appends a compile-checked Usage example module.
pub fn add_usage_example(text: &str, component: &Component) -> Result<String, String> {
    let module = &component.module;
    if text.contains(&format!("\nmod {module} {{")) {
        return Err(format!("already has `mod {module}`"));
    }
    let mut text = text.trim_end().to_owned();
    text.push_str(&format!(
        "\n\nmod {module} {{\n    use crate::ui::{module}::*;\n    use gpui_kit::IntoElement;\n\n    fn example() -> impl IntoElement {{\n        {type_name}::new()\n    }}\n}}\n",
        type_name = component.type_name
    ));
    Ok(text)
}

/// Adds the preview to the `previews!` list in alphabetical order.
pub fn list_preview(text: &str, component: &Component) -> Result<String, String> {
    let module = &component.module;
    let entry = format!("    {module} => {},\n", component.type_name);
    if text.contains(&format!("    {module} => ")) {
        return Err(format!("already lists {module}"));
    }
    let start = text
        .find("previews! {\n")
        .ok_or("found no `previews! {` list")?
        + "previews! {\n".len();
    let end = start
        + text[start..]
            .find("\n}")
            .ok_or("unterminated `previews!` list")?;
    let mut entries: Vec<&str> = text[start..=end].split_inclusive('\n').collect();
    let at = entries
        .iter()
        .position(|line| line.trim_start().split(' ').next().unwrap_or("") > module.as_str())
        .unwrap_or(entries.len());
    entries.insert(at, &entry);
    Ok(format!(
        "{}{}{}",
        &text[..start],
        entries.concat(),
        &text[end + 1..]
    ))
}

/// Adds the catalog entry to `COMPONENTS` in alphabetical slug order.
pub fn list_in_gallery(text: &str, component: &Component) -> Result<String, String> {
    let slug = component.slug();
    let entry = format!(
        "    (\n        \"{slug}\",\n        \"{}\",\n        \"{}\",\n    ),\n",
        component.title.replace('"', "\\\""),
        component.description.replace('"', "\\\"")
    );
    if text.contains(&format!("\"{slug}\",\n")) {
        return Err(format!("already lists {slug}"));
    }
    let start = text
        .find("const COMPONENTS: &[(&str, &str, &str)] = &[\n")
        .ok_or("found no COMPONENTS list")?;
    let start = start + text[start..].find('\n').unwrap() + 1;
    let end = start
        + text[start..]
            .find("\n];")
            .ok_or("unterminated COMPONENTS list")?;
    let body = &text[start..=end];
    // Entries are `(\n "slug",\n …\n ),\n` blocks.
    let mut blocks = Vec::new();
    let mut rest = body;
    while let Some(close) = rest.find("    ),\n") {
        let (block, tail) = rest.split_at(close + "    ),\n".len());
        blocks.push(block);
        rest = tail;
    }
    let slug_of = |block: &str| {
        block
            .lines()
            .nth(1)
            .map(|line| line.trim().trim_matches(|c| c == '"' || c == ','))
            .unwrap_or("")
            .to_owned()
    };
    let at = blocks
        .iter()
        .position(|block| slug_of(block) > slug)
        .unwrap_or(blocks.len());
    blocks.insert(at, &entry);
    Ok(format!(
        "{}{}{}{}",
        &text[..start],
        blocks.concat(),
        rest,
        &text[end + 1..]
    ))
}

/// Adds the website's component page to `components` in web/src/lib/catalog.ts,
/// in alphabetical slug order. Without it the catalog preview exists but the
/// website has no page for it.
pub fn list_in_site_catalog(text: &str, component: &Component) -> Result<String, String> {
    let slug = component.slug();
    if text.contains(&format!("slug: \"{slug}\",")) {
        return Err(format!("already lists {slug}"));
    }
    let start = text
        .find("export const components: CatalogComponent[] = [\n")
        .ok_or("found no `components` list")?;
    let start = start + text[start..].find('\n').unwrap() + 1;
    let end = start
        + text[start..]
            .find("\n]")
            .ok_or("unterminated `components` list")?;
    let entry = format!(
        "  {{\n    slug: \"{slug}\",\n    name: \"{}\",\n    description:\n      \"{}\",\n  }},\n",
        component.title.replace('"', "\\\""),
        component.description.replace('"', "\\\"")
    );
    // Insert before the first entry whose slug sorts after this one.
    let mut at = end + 1;
    let mut offset = start;
    for line in text[start..=end].split_inclusive('\n') {
        if let Some(other) = line.trim().strip_prefix("slug: \"") {
            let other = other.split('"').next().unwrap_or("");
            if other > slug.as_str() {
                // Back up to the `  {` line that opens this entry.
                at = text[..offset].rfind("  {\n").ok_or("malformed entry")?;
                break;
            }
        }
        offset += line.len();
    }
    Ok(format!("{}{}{}", &text[..at], entry, &text[at..]))
}

pub fn component_source(component: &Component) -> String {
    let Component {
        module,
        type_name,
        title,
        description,
    } = component;
    format!(
        r#"//! {title}: {description}
//!
//! Visual reference: shadcn/ui {title}, Nova. Name the GPUI Kit behavior this
//! builds on here, and read every color, size and radius from the theme or
//! `theme::primitives` (the registry lint in crates/ui/tests enforces it).

use gpui_kit::{{App, IntoElement, ParentElement as _, RenderOnce, SharedString, Styled as _, Window, div}};

use super::theme::{{UiTheme, primitives::{{surface, text}}}};

/// {description}
#[derive(IntoElement)]
pub struct {type_name} {{
    label: SharedString,
}}

impl {type_name} {{
    /// Creates a `{type_name}`.
    pub fn new() -> Self {{
        Self {{
            label: "{title}".into(),
        }}
    }}

    /// The text shown inside.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {{
        self.label = label.into();
        self
    }}
}}

impl Default for {type_name} {{
    fn default() -> Self {{
        Self::new()
    }}
}}

impl RenderOnce for {type_name} {{
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {{
        let theme = UiTheme::read(cx).clone();
        text::sm(surface::card(div(), &theme), &theme)
            .px(theme.space(3.))
            .py(theme.space(2.))
            .text_color(theme.colors.card_foreground)
            .child(self.label)
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;
    use gpui_kit::{{Context, Render, TestAppContext, VisualTestContext}};

    struct Probe;

    impl Render for Probe {{
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {{
            {type_name}::new().label("Probe")
        }}
    }}

    #[gpui_kit::test]
    fn draws_in_both_themes(cx: &mut TestAppContext) {{
        cx.update(super::super::theme::init);
        let window = cx.add_window(|_, _| Probe);
        let mut cx = VisualTestContext::from_window(window.into(), cx);
        for mode in [super::super::theme::ThemeMode::Light, super::super::theme::ThemeMode::Dark] {{
            cx.update(|_, cx| UiTheme::switch(cx, mode));
            cx.update(|window, cx| window.draw(cx).clear(cx));
        }}
    }}
}}
"#
    )
    .replace("{module}", module)
}

pub fn parity_doc(component: &Component) -> String {
    let Component { module, title, .. } = component;
    format!(
        "# {title} parity\n\nVisual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:\n`crates/ui/registry/{module}/{module}.rs`, using GPUI Kit 0.9.0.\n\nDescribe which parts GPUI Kit supplies, what gpuicn adds (Nova borders,\nspacing, typography) and what the application owns.\n\nThe Rust source and Usage example define the supported API.\n"
    )
}

pub fn preview_source(component: &Component) -> String {
    let Component {
        module, type_name, ..
    } = component;
    format!(
        "//! The {name} catalog preview.\n\nuse crate::*;\n\nimpl Showcase {{\n    pub(crate) fn {module}_preview(\n        &self,\n        _window: &mut Window,\n        _cx: &mut Context<Self>,\n    ) -> impl IntoElement {{\n        gpuicn::{module}::{type_name}::new()\n    }}\n}}\n\npub(crate) fn preview(\n    view: &Showcase,\n    window: &mut Window,\n    cx: &mut Context<Showcase>,\n) -> gpui_kit::AnyElement {{\n    view.{module}_preview(window, cx).into_any_element()\n}}\n",
        name = module.replace('_', " ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn component() -> Component {
        Component::new("date_picker", None, Some("Picks a date.")).unwrap()
    }

    #[test]
    fn names_derive_from_the_module() {
        let component = component();
        assert_eq!(component.type_name, "DatePicker");
        assert_eq!(component.title, "Date Picker");
        assert_eq!(component.slug(), "date-picker");
        assert!(Component::new("DatePicker", None, None).is_err());
        assert!(Component::new("date-picker", None, None).is_err());
        assert!(Component::new("_x", None, None).is_err());
        assert_eq!(
            parse_new_component(&["x".into(), "--title".into(), "X marks".into()])
                .unwrap()
                .title,
            "X marks"
        );
        assert!(parse_new_component(&["--bogus".into()]).is_err());
    }

    #[test]
    fn edits_insert_in_order_and_refuse_duplicates() {
        let component = component();
        let lib = "//! crate\n#[path = \"../registry/button/button.rs\"]\npub mod button;\n#[path = \"../registry/dialog/dialog.rs\"]\npub mod dialog;\n";
        let edited = declare_in_lib(lib, &component).unwrap();
        assert_eq!(
            edited,
            "//! crate\n#[path = \"../registry/button/button.rs\"]\npub mod button;\n#[path = \"../registry/date_picker/date_picker.rs\"]\npub mod date_picker;\n#[path = \"../registry/dialog/dialog.rs\"]\npub mod dialog;\n"
        );
        assert!(declare_in_lib(&edited, &component).is_err());
        let late = Component::new("zoom", None, None).unwrap();
        assert!(
            declare_in_lib(lib, &late)
                .unwrap()
                .ends_with("pub mod zoom;\n")
        );

        let registry = "[components.button]\ndescription = \"A button.\"\n";
        let edited = declare_in_registry(registry, &component).unwrap();
        assert!(edited.ends_with("[components.date_picker]\ndescription = \"Picks a date.\"\n"));
        assert!(declare_in_registry(&edited, &component).is_err());

        let usage = "use gpuicn as ui;\n\nmod button {\n    fn example() {}\n}\n";
        let edited = add_usage_example(usage, &component).unwrap();
        assert!(edited.contains("\nmod date_picker {\n    use crate::ui::date_picker::*;"));
        assert!(edited.contains("fn example("));
        assert!(add_usage_example(&edited, &component).is_err());

        let previews = "previews! {\n    button => Button,\n    dialog => Dialog,\n}\n";
        let edited = list_preview(previews, &component).unwrap();
        assert_eq!(
            edited,
            "previews! {\n    button => Button,\n    date_picker => DatePicker,\n    dialog => Dialog,\n}\n"
        );
        assert!(list_preview(&edited, &component).is_err());

        let gallery = "const COMPONENTS: &[(&str, &str, &str)] = &[\n    (\n        \"button\",\n        \"Button\",\n        \"A button.\",\n    ),\n    (\n        \"dialog\",\n        \"Dialog\",\n        \"A dialog.\",\n    ),\n];\n";
        let edited = list_in_gallery(gallery, &component).unwrap();
        let slugs: Vec<&str> = edited
            .lines()
            .filter(|line| line.starts_with("        \"") && line.ends_with("\","))
            .step_by(3)
            .collect();
        assert_eq!(
            slugs,
            [
                "        \"button\",",
                "        \"date-picker\",",
                "        \"dialog\","
            ]
        );
        assert!(edited.ends_with("    ),\n];\n"));
        assert!(list_in_gallery(&edited, &component).is_err());

        let catalog = "export const components: CatalogComponent[] = [\n  {\n    slug: \"button\",\n    name: \"Button\",\n    description: \"A button.\",\n  },\n  {\n    slug: \"dialog\",\n    name: \"Dialog\",\n    description: \"A dialog.\",\n  },\n]\n";
        let edited = list_in_site_catalog(catalog, &component).unwrap();
        let slugs: Vec<&str> = edited
            .lines()
            .filter_map(|line| line.trim().strip_prefix("slug: "))
            .collect();
        assert_eq!(slugs, ["\"button\",", "\"date-picker\",", "\"dialog\","]);
        assert!(edited.contains("    name: \"Date Picker\",\n"));
        assert!(list_in_site_catalog(&edited, &component).is_err());
        let last = list_in_site_catalog(catalog, &late).unwrap();
        assert!(last.ends_with("    slug: \"zoom\",\n    name: \"Zoom\",\n    description:\n      \"Describe the component in one sentence.\",\n  },\n]\n"));

        assert!(component_source(&component).contains("pub struct DatePicker"));
        assert!(component_source(&component).contains("#[cfg(test)]"));
        assert!(preview_source(&component).contains("fn date_picker_preview("));
        assert!(parity_doc(&component).starts_with("# Date Picker parity"));
    }
}
