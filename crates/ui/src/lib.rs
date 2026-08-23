//! Open-code shadcn visual ports for GPUI.
//!
//! Every installed component root takes a caller-owned [`gpui::ElementId`].
//! Keep IDs stable across renders and unique among siblings. Use semantic paths
//! such as `settings.save`; never derive an ID from visible text, list position,
//! a pointer, or a random value. Changing an ID resets GPUI's keyed state.

#[path = "../../../registry/theme/theme.rs"]
pub mod theme;

pub use theme::{ThemeMode, UiColors, UiFonts, UiRadius, UiSpacing, UiTheme};
