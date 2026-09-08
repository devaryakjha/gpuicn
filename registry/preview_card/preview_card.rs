//! The shadcn Nova Hover Card visual port, named Preview Card in Base GPUI.
//!
//! Base GPUI keeps hover/focus delay, the safe polygon and anchor collision behavior.

pub use base_gpui::preview_card::{
    PreviewCardAlign, PreviewCardArrow, PreviewCardBackdrop, PreviewCardPopup, PreviewCardPortal,
    PreviewCardPositioner, PreviewCardRoot, PreviewCardSide, PreviewCardTrigger,
    PreviewCardViewport,
};
use gpui::{App, ElementId, Styled, px};

use super::theme::UiTheme;

/// Creates a Preview Card root with a caller-owned stable ID.
pub fn preview_card_root(id: impl Into<ElementId>) -> PreviewCardRoot<()> {
    PreviewCardRoot::new().id(id)
}

/// Creates a Preview Card trigger. Callers may add link-like styling as needed.
pub fn preview_card_trigger(id: impl Into<ElementId>) -> PreviewCardTrigger<()> {
    PreviewCardTrigger::new().id(id)
}

/// Creates the Preview Card portal.
pub fn preview_card_portal() -> PreviewCardPortal<()> {
    PreviewCardPortal::new()
}

/// Creates an anchored Preview Card positioner with Nova's 4px side offset.
pub fn preview_card_positioner(cx: &App) -> PreviewCardPositioner<()> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    PreviewCardPositioner::new()
        .side_offset(spacing * 1_f32)
        .align_offset(spacing * 1_f32)
}

/// Creates the 256px Nova Hover Card surface.
pub fn preview_card_popup(id: impl Into<ElementId>, cx: &App) -> PreviewCardPopup<()> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    PreviewCardPopup::new()
        .id(id)
        .style_with_state(move |_state, base| {
            base.w(spacing * 64_f32)
                .rounded(theme.radius.lg)
                .border_1()
                .border_color(theme.colors.foreground.opacity(0.10))
                .p(spacing * 2.5_f32)
                .bg(theme.colors.popover)
                .text_color(theme.colors.popover_foreground)
                .font_family(theme.fonts.body.clone())
                .text_size(px(14.0) * text_scale)
                .shadow(theme.shadows.md.clone())
        })
}

/// Creates the Preview Card arrow surface.
pub fn preview_card_arrow(cx: &App) -> PreviewCardArrow<()> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    PreviewCardArrow::new()
        .size(spacing * 2.5_f32)
        .bg(theme.colors.popover)
}
