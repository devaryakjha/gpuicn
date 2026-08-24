//! The shadcn Nova Hover Card visual port, named Preview Card in Base GPUI.
//!
//! Base GPUI keeps hover/focus delay, the safe polygon and anchor collision behavior.

pub use base_gpui::preview_card::{
    PreviewCardAlign, PreviewCardArrow, PreviewCardBackdrop, PreviewCardPopup, PreviewCardPortal,
    PreviewCardPositioner, PreviewCardRoot, PreviewCardSide, PreviewCardTrigger,
    PreviewCardViewport,
};
use gpui::{App, BoxShadow, ElementId, Styled, px};

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
pub fn preview_card_positioner() -> PreviewCardPositioner<()> {
    PreviewCardPositioner::new()
        .side_offset(px(4.0))
        .align_offset(px(4.0))
}

/// Creates the 256px Nova Hover Card surface.
pub fn preview_card_popup(id: impl Into<ElementId>, cx: &App) -> PreviewCardPopup<()> {
    let theme = UiTheme::read(cx).clone();
    PreviewCardPopup::new()
        .id(id)
        .style_with_state(move |_state, base| {
            base.w(px(256.0))
                .rounded(px(8.0))
                .border_1()
                .border_color(theme.colors.foreground.alpha(0.10))
                .p(px(10.0))
                .bg(theme.colors.popover)
                .text_color(theme.colors.popover_foreground)
                .font_family(theme.fonts.body.clone())
                .text_size(px(14.0))
                .shadow(vec![
                    BoxShadow::new(px(0.0), px(4.0), theme.colors.foreground.alpha(0.12).into())
                        .blur_radius(px(8.0)),
                ])
        })
}

/// Creates the Preview Card arrow surface.
pub fn preview_card_arrow(cx: &App) -> PreviewCardArrow<()> {
    let theme = UiTheme::read(cx).clone();
    PreviewCardArrow::new()
        .size(px(10.0))
        .bg(theme.colors.popover)
}
