//! Minimal native proof that an application can install and switch `UiTheme`
//! through `AppConfig`, the way `app::run` does.

use gpuicn::{Appearance, ThemeMode, UiTheme, app::AppConfig};

fn main() {
    gpui_kit::platform::headless().run(|cx| {
        let options = AppConfig::new("Theme check")
            .appearance(Appearance::Light)
            .install(cx);
        assert_eq!(
            options.titlebar.unwrap().title.as_deref(),
            Some("Theme check")
        );
        assert_eq!(UiTheme::read(cx).mode, ThemeMode::Light);
        UiTheme::set_appearance(cx, Appearance::Dark);
        assert_eq!(UiTheme::read(cx).mode, ThemeMode::Dark);
        UiTheme::switch(cx, ThemeMode::Light);
        assert_eq!(UiTheme::read(cx).mode, ThemeMode::Light);
        cx.quit();
    });
}
