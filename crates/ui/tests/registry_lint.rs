//! Source lint for the component registry.
//!
//! Every file under `crates/ui/registry` is open code that applications copy,
//! so the rules that keep the catalog visually consistent are checked here
//! rather than remembered. Each rule names the shared primitive that replaces
//! the pattern it bans. `registry_lint_baseline.txt` lists the violations that
//! existed when a rule was added; a file may only move toward zero. Run with
//! `REGISTRY_LINT_WRITE_BASELINE=1` to rewrite the baseline after adding a rule.

use std::{collections::BTreeMap, fs, path::Path};

struct Rule {
    name: &'static str,
    /// A file matches when every needle occurs in it (not necessarily on one line).
    needles_in_file: &'static [&'static str],
    /// Counted per line; when empty, the file-level match counts once.
    needles_per_line: &'static [&'static str],
    /// Lines containing any of these are not counted (color alpha is not element opacity).
    skip_lines_with: &'static [&'static str],
    /// `(opener, follower, lines)`: counts an `opener` line followed by a
    /// `follower` line within `lines`, for builder chains such as `img(..).rounded(..)`.
    needles_within: Option<(&'static str, &'static str, usize)>,
    /// Files exempt from the rule, by `module/file.rs`.
    allow: &'static [&'static str],
    fix: &'static str,
}

const PRIMITIVES: &str = "theme/primitives.rs";

const RULES: &[Rule] = &[
    Rule {
        name: "raw-color",
        needles_in_file: &[],
        needles_per_line: &[
            "hsla(",
            "rgb(0x",
            "rgba(0x",
            "::white()",
            "::black()",
            " white()",
            " black()",
        ],
        skip_lines_with: &[],
        needles_within: None,
        allow: &["theme/theme.rs", "lightbox/lightbox.rs"],
        fix: "read colors from `UiTheme`; add a semantic token when none fits",
    },
    Rule {
        name: "raw-text-size",
        needles_in_file: &[],
        needles_per_line: &[
            "text_size(px(",
            "* text_scale)",
            "* theme.text_scale)",
            "* t.text_scale)",
        ],
        skip_lines_with: &[],
        needles_within: None,
        allow: &["theme/theme.rs"],
        fix: "use `text::xs/sm/base/lg` from the primitives or `theme.text(..)`",
    },
    Rule {
        name: "disabled-literal",
        needles_in_file: &[],
        needles_per_line: &["opacity(0.5)", "opacity(0.50)"],
        skip_lines_with: &["colors."],
        needles_within: None,
        allow: &["theme/theme.rs", PRIMITIVES],
        fix: "apply `disabled::style(..)` or `disabled::dim(..)` from the primitives",
    },
    Rule {
        name: "mode-branch",
        needles_in_file: &[],
        needles_per_line: &["ThemeMode::Dark"],
        skip_lines_with: &[],
        needles_within: None,
        allow: &["theme/theme.rs", PRIMITIVES],
        fix: "use the derived control tokens (`colors.control_*`) computed once in theme.rs",
    },
    Rule {
        name: "popover-surface",
        needles_in_file: &[],
        needles_per_line: &[
            ".bg(t.colors.popover)",
            ".bg(theme.colors.popover)",
            ".bg(colors.popover)",
        ],
        skip_lines_with: &[],
        needles_within: None,
        allow: &[PRIMITIVES],
        fix: "use `surface::popover(..)` or `surface::modal(..)` from the primitives",
    },
    Rule {
        name: "image-clip",
        needles_in_file: &[],
        needles_per_line: &[],
        skip_lines_with: &[],
        needles_within: Some(("img(", ".rounded", 6)),
        // Markdown pictures size their frame from the picture, so they round
        // themselves inside a plain frame.
        allow: &[PRIMITIVES, "markdown/markdown.rs"],
        fix: "use `clipped_image(..)` from the primitives; GPUI clips to rectangles",
    },
    Rule {
        name: "hover-state",
        needles_in_file: &[],
        needles_per_line: &["\"hovered\""],
        skip_lines_with: &[],
        needles_within: None,
        allow: &[],
        fix: "style hover with `.hover(..)`; reserve space for revealed actions instead of re-rendering",
    },
    Rule {
        name: "key-glyph",
        needles_in_file: &[],
        needles_per_line: &["⌘", "⇧", "⌥", "⌃"],
        skip_lines_with: &[],
        needles_within: None,
        allow: &["kbd/kbd.rs"],
        fix: "draw key symbols through `kbd::key_label`; the bundled fonts have no glyphs for them",
    },
    Rule {
        name: "crate-private",
        needles_in_file: &[],
        needles_per_line: &["pub(crate)", "pub(super)"],
        skip_lines_with: &[],
        needles_within: None,
        allow: &[],
        fix: "ejected files reach the crate through `super::`; use `#[doc(hidden)] pub`",
    },
    Rule {
        name: "manual-focus-style",
        needles_in_file: &[],
        needles_per_line: &[".when(focused"],
        skip_lines_with: &[],
        needles_within: None,
        allow: &[PRIMITIVES],
        fix: "use `focus::ring` through `focus_visible`, not a border swap on `is_focused`",
    },
];

fn registry_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("registry")
}

fn registry_files() -> BTreeMap<String, String> {
    let mut files = BTreeMap::new();
    for module in fs::read_dir(registry_dir()).expect("registry directory") {
        let module = module.unwrap().path();
        if !module.is_dir() {
            continue;
        }
        for file in fs::read_dir(&module).unwrap() {
            let file = file.unwrap().path();
            if file.extension().is_some_and(|ext| ext == "rs") {
                let key = format!(
                    "{}/{}",
                    module.file_name().unwrap().to_string_lossy(),
                    file.file_name().unwrap().to_string_lossy()
                );
                files.insert(key, fs::read_to_string(&file).unwrap());
            }
        }
    }
    files
}

/// Code before `#[cfg(test)]`; tests may use literals freely.
fn non_test(source: &str) -> &str {
    source.split("#[cfg(test)]").next().unwrap_or(source)
}

fn count(rule: &Rule, source: &str) -> usize {
    let source = non_test(source);
    if !rule.needles_in_file.iter().all(|n| source.contains(n)) {
        return 0;
    }
    if let Some((opener, follower, within)) = rule.needles_within {
        let lines: Vec<&str> = source.lines().collect();
        return lines
            .iter()
            .enumerate()
            .filter(|(index, line)| {
                line.contains(opener)
                    && lines[*index..(*index + within).min(lines.len())]
                        .iter()
                        .any(|next| next.contains(follower))
            })
            .count();
    }
    if rule.needles_per_line.is_empty() {
        return 1;
    }
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .filter(|line| !rule.skip_lines_with.iter().any(|n| line.contains(n)))
        .map(|line| {
            rule.needles_per_line
                .iter()
                .filter(|n| line.contains(*n))
                .count()
        })
        .sum()
}

fn baseline_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/registry_lint_baseline.txt")
}

fn baseline() -> BTreeMap<(String, String), usize> {
    let text = fs::read_to_string(baseline_path()).unwrap_or_default();
    text.lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|line| {
            let mut parts = line.split_whitespace();
            let rule = parts.next().unwrap().to_owned();
            let file = parts.next().unwrap().to_owned();
            let count: usize = parts.next().unwrap().parse().unwrap();
            ((rule, file), count)
        })
        .collect()
}

/// A component counts as tested when its own file has a test module or an
/// integration test under `tests/` names its module.
fn untested_files(files: &BTreeMap<String, String>) -> Vec<String> {
    let tests_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let integration: String = fs::read_dir(&tests_dir)
        .unwrap()
        .filter_map(|e| fs::read_to_string(e.unwrap().path()).ok())
        .collect();
    files
        .iter()
        .filter(|(file, source)| {
            let module = file.split('/').next().unwrap();
            !source.contains("#[cfg(test)]") && !names_module(&integration, module)
        })
        .map(|(file, _)| file.clone())
        .collect()
}

/// Whether `text` uses `module::` or `module;` as a path segment, as in
/// `gpuicn::module::Item` or the `module::{..}` entries of a `use` list.
fn names_module(text: &str, module: &str) -> bool {
    text.match_indices(module).any(|(start, _)| {
        let before = text[..start].chars().next_back();
        let after = &text[start + module.len()..];
        !before.is_some_and(|c| c.is_alphanumeric() || c == '_')
            && (after.starts_with("::") || after.starts_with(';'))
    })
}

#[test]
fn registry_source_follows_the_design_rules() {
    let files = registry_files();
    assert!(
        files.len() > 50,
        "expected the component registry, found {}",
        files.len()
    );
    if std::env::var_os("REGISTRY_LINT_WRITE_BASELINE").is_some() {
        let mut lines =
            vec!["# rule file count. Counts may only go down; delete a line at zero.".to_owned()];
        for rule in RULES {
            for (file, source) in &files {
                if rule.allow.contains(&file.as_str()) {
                    continue;
                }
                let found = count(rule, source);
                if found > 0 {
                    lines.push(format!("{} {file} {found}", rule.name));
                }
            }
        }
        for file in untested_files(&files) {
            lines.push(format!("untested {file} 1"));
        }
        fs::write(baseline_path(), lines.join("\n") + "\n").unwrap();
        return;
    }
    let baseline = baseline();
    let mut failures = Vec::new();
    let mut improvements = Vec::new();
    for rule in RULES {
        for (file, source) in &files {
            if rule.allow.contains(&file.as_str()) {
                continue;
            }
            let found = count(rule, source);
            let allowed = baseline
                .get(&(rule.name.to_owned(), file.clone()))
                .copied()
                .unwrap_or(0);
            if found > allowed {
                failures.push(format!(
                    "{file}: {found} `{}` violation(s) (baseline {allowed}). Fix: {}",
                    rule.name, rule.fix
                ));
            } else if found < allowed {
                improvements.push(format!("{} {file} {found}", rule.name));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "registry lint failed:\n  {}",
        failures.join("\n  ")
    );
    assert!(
        improvements.is_empty(),
        "registry lint baseline is stale; lower these lines in tests/registry_lint_baseline.txt:\n  {}",
        improvements.join("\n  ")
    );
}

#[test]
fn every_component_file_is_named_in_a_test() {
    let files = registry_files();
    let untested = untested_files(&files);
    let allowed: Vec<_> = baseline()
        .into_keys()
        .filter(|(rule, _)| rule == "untested")
        .map(|(_, file)| file)
        .collect();
    let new: Vec<_> = untested.iter().filter(|f| !allowed.contains(f)).collect();
    assert!(new.is_empty(), "components without any test: {new:?}");
    let fixed: Vec<_> = allowed.iter().filter(|f| !untested.contains(f)).collect();
    assert!(
        fixed.is_empty(),
        "remove these `untested` baseline lines: {fixed:?}"
    );
}
