//! Every GPUI Kit component color follows a gpuicn token. This lives outside the
//! registry because it needs `serde_json`, which installed apps do not have.

use gpuicn::{ThemeMode, UiTheme};

/// Kit colors that gpuicn leaves at Kit's value for the mode, each with the reason.
const KIT_INHERITED: &[(&str, &str)] = &[
    ("tab", "transparent in both palettes"),
    ("red", "syntax and terminal palette, not a shadcn token"),
    (
        "red_light",
        "syntax and terminal palette, not a shadcn token",
    ),
    ("green", "syntax and terminal palette, not a shadcn token"),
    (
        "green_light",
        "syntax and terminal palette, not a shadcn token",
    ),
    ("blue", "syntax and terminal palette, not a shadcn token"),
    (
        "blue_light",
        "syntax and terminal palette, not a shadcn token",
    ),
    ("yellow", "syntax and terminal palette, not a shadcn token"),
    (
        "yellow_light",
        "syntax and terminal palette, not a shadcn token",
    ),
    ("magenta", "syntax and terminal palette, not a shadcn token"),
    (
        "magenta_light",
        "syntax and terminal palette, not a shadcn token",
    ),
    ("cyan", "syntax and terminal palette, not a shadcn token"),
    (
        "cyan_light",
        "syntax and terminal palette, not a shadcn token",
    ),
];

/// Every `UiColors` slot, so a theme can be built where each one is distinct.
const SLOTS: &[&str] = &[
    "background",
    "foreground",
    "card",
    "card-foreground",
    "popover",
    "popover-foreground",
    "primary",
    "primary-foreground",
    "secondary",
    "secondary-foreground",
    "muted",
    "muted-foreground",
    "accent",
    "accent-foreground",
    "destructive",
    "destructive-foreground",
    "success",
    "success-foreground",
    "warning",
    "warning-foreground",
    "info",
    "info-foreground",
    "border",
    "input",
    "ring",
    "chart-1",
    "chart-2",
    "chart-3",
    "chart-4",
    "chart-5",
    "sidebar",
    "sidebar-foreground",
    "sidebar-primary",
    "sidebar-primary-foreground",
    "sidebar-accent",
    "sidebar-accent-foreground",
    "sidebar-border",
    "sidebar-ring",
];

/// Every Kit color must follow a gpuicn token: a Kit widget that keeps Kit's
/// default palette in a themed app is a visible seam. A theme where every
/// slot holds an unlikely color is synced, and any Kit field still equal to
/// Kit's default for the mode is unmapped, unless `KIT_INHERITED` says why.
#[gpui_kit::test]
fn maps_every_component_color(cx: &mut gpui_kit::TestAppContext) {
    cx.update(|cx| {
        gpuicn::init(cx);
        for (mode, base, kit_default) in [
            (
                ThemeMode::Light,
                UiTheme::neutral_light(),
                gpui_kit::component::ThemeColor::light(),
            ),
            (
                ThemeMode::Dark,
                UiTheme::neutral_dark(),
                gpui_kit::component::ThemeColor::dark(),
            ),
        ] {
            let mut theme = base;
            for (index, slot) in SLOTS.iter().enumerate() {
                let hue = (index as f32 * 37.) % 360.;
                let color = gpui_kit::hsla(hue / 360., 0.73, 0.41, 0.97);
                *theme.colors.slot(slot).unwrap() = color.into();
            }
            theme.colors.overlay = gpui_kit::hsla(0.5, 0.5, 0.5, 0.42).into();
            UiTheme::set(cx, theme);
            let synced =
                serde_json::to_value(gpui_kit::component::Theme::global(cx).colors).unwrap();
            let defaults = serde_json::to_value(*kit_default).unwrap();
            let unmapped: Vec<&String> = synced
                .as_object()
                .unwrap()
                .iter()
                .filter(|(field, value)| {
                    defaults[field.as_str()] == **value
                        && !KIT_INHERITED.iter().any(|(name, _)| name == field)
                })
                .map(|(field, _)| field)
                .collect();
            assert!(
                unmapped.is_empty(),
                "{mode:?}: Kit colors left at Kit's default by sync_component_theme: {unmapped:?}"
            );
            let stale: Vec<&&str> = KIT_INHERITED
                .iter()
                .map(|(name, _)| name)
                .filter(|name| synced[**name] != defaults[**name])
                .collect();
            assert!(
                stale.is_empty(),
                "{mode:?}: now mapped, drop from KIT_INHERITED: {stale:?}"
            );
        }
    });
}
