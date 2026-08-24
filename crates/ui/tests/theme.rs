//! Theme global-state integration checks.

use gpui::TestAppContext;
use gpuicn::{ThemeMode, UiTheme};

#[gpui::test]
fn switches_the_app_owned_theme(cx: &mut TestAppContext) {
    cx.update(|cx| UiTheme::set(cx, UiTheme::neutral_light()));
    cx.update(|cx| {
        assert_eq!(UiTheme::read(cx).mode, ThemeMode::Light);
        UiTheme::switch(cx, ThemeMode::Dark);
        assert_eq!(UiTheme::read(cx).mode, ThemeMode::Dark);
        assert_eq!(
            UiTheme::read(cx).colors.background,
            UiTheme::neutral_dark().colors.background
        );
    });
}
