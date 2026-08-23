//! Minimal native proof that an application can install and switch `UiTheme`.

use imajha_ui::{ThemeMode, UiTheme};

fn main() {
    gpui_platform::headless().run(|cx| {
        UiTheme::set(cx, UiTheme::neutral_light());
        assert_eq!(UiTheme::read(cx).mode, ThemeMode::Light);
        UiTheme::switch(cx, ThemeMode::Dark);
        assert_eq!(UiTheme::read(cx).mode, ThemeMode::Dark);
        cx.quit();
    });
}
