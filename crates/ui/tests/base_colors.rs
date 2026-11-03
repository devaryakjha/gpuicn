//! Every base color the CLI ships builds a light and a dark theme.

use gpuicn::{ThemeMode, UiTheme};
use std::{fs, path::Path};

#[test]
fn every_base_color_parses_in_both_modes() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../cli/templates/base-colors");
    let mut names = Vec::new();
    for entry in fs::read_dir(&dir).expect("base-colors directory") {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|ext| ext != "css") {
            continue;
        }
        let name = path.file_stem().unwrap().to_string_lossy().into_owned();
        let css = fs::read_to_string(&path).unwrap();
        let light = UiTheme::from_shadcn_css(&css, ThemeMode::Light)
            .unwrap_or_else(|error| panic!("{name} light: {error}"));
        let dark = UiTheme::from_shadcn_css(&css, ThemeMode::Dark)
            .unwrap_or_else(|error| panic!("{name} dark: {error}"));
        assert_eq!(light.mode, ThemeMode::Light);
        assert_eq!(dark.mode, ThemeMode::Dark);
        assert_ne!(light.colors.background, dark.colors.background, "{name}");
        assert_eq!(
            light.radius,
            UiTheme::neutral_light().radius,
            "{name} radius"
        );
        names.push(name);
    }
    names.sort();
    assert_eq!(
        names,
        [
            "mauve", "mist", "neutral", "olive", "stone", "taupe", "zinc"
        ]
    );
}
