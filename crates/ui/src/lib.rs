//! Open-code shadcn visual ports for GPUI.
//!
//! Every installed component root takes a caller-owned [`gpui::ElementId`].
//! Keep IDs stable across renders and unique among siblings. Use semantic paths
//! such as `settings.save`; never derive an ID from visible text, list position,
//! a pointer, or a random value. Changing an ID resets GPUI's keyed state.

#[path = "../../../registry/button/button.rs"]
pub mod button;
#[path = "../../../registry/checkbox/checkbox.rs"]
pub mod checkbox;
#[path = "../../../registry/dialog/dialog.rs"]
pub mod dialog;
#[path = "../../../registry/theme/theme.rs"]
pub mod theme;

pub use button::{Button, ButtonSize, ButtonVariant};
pub use checkbox::Checkbox;
pub use theme::{ThemeMode, UiColors, UiFonts, UiRadius, UiSpacing, UiTheme};

/// Registers the Base GPUI actions used by the v0.1 component slice.
pub fn init(cx: &mut gpui::App) {
    base_gpui::button::init(cx);
    base_gpui::checkbox::init(cx);
    base_gpui::dialog::init(cx);
}
