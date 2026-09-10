//! Theme global-state integration checks.

use gpui_kit::{TestAppContext, px};
use gpuicn::theme::UiRadius;
use gpuicn::{ThemeMode, UiTheme};

#[gpui_kit::test]
fn switches_the_app_owned_theme(cx: &mut TestAppContext) {
    cx.update(|cx| {
        UiTheme::switch(cx, ThemeMode::Dark);
        assert_eq!(UiTheme::read(cx).mode, ThemeMode::Dark);
        let mut theme = UiTheme::neutral_light();
        theme.spacing.unit = px(5.);
        theme.radius = UiRadius::new(px(4.));
        theme.text_scale = 1.1;
        theme.motion.reduced = true;
        theme.fonts.body = "Custom font".into();
        UiTheme::set(cx, theme);
    });
    cx.update(|cx| {
        assert_eq!(UiTheme::read(cx).mode, ThemeMode::Light);
        UiTheme::switch(cx, ThemeMode::Dark);
        let theme = UiTheme::read(cx);
        assert_eq!(theme.mode, ThemeMode::Dark);
        assert_eq!(theme.space(2.), px(10.));
        assert_eq!(theme.radius.lg, px(4.));
        assert_eq!(theme.text(10.), px(11.));
        assert!(theme.motion.reduced);
        assert_eq!(theme.fonts.body.as_ref(), "Custom font");
        assert_eq!(
            UiTheme::read(cx).colors.background,
            UiTheme::neutral_dark().colors.background
        );
    });
}
