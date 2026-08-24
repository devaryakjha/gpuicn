//! The shadcn Nova Tooltip visual port.
//!
//! Base GPUI owns the provider delay group, hover/focus lifecycle and collision behavior.

pub use base_gpui::tooltip::{
    TooltipAlign, TooltipPopup, TooltipPortal, TooltipPositioner, TooltipProvider, TooltipRoot,
    TooltipSide, TooltipTrigger, TooltipViewport,
};
use gpui::{App, BoxShadow, ElementId, Styled, px};

use super::theme::UiTheme;

/// Creates a Tooltip provider with a caller-owned stable ID.
pub fn tooltip_provider(id: impl Into<ElementId>) -> TooltipProvider<()> {
    TooltipProvider::new().id(id)
}

/// Creates a Tooltip root with a caller-owned stable ID.
pub fn tooltip_root(id: impl Into<ElementId>) -> TooltipRoot<()> {
    TooltipRoot::new().id(id)
}

/// Creates a Tooltip trigger. Keep its host control's own visual treatment.
pub fn tooltip_trigger(id: impl Into<ElementId>) -> TooltipTrigger<()> {
    TooltipTrigger::new().id(id)
}

/// Creates the Tooltip portal.
pub fn tooltip_portal() -> TooltipPortal<()> {
    TooltipPortal::new()
}

/// Creates an anchored Tooltip positioner with Nova's 4px side offset.
pub fn tooltip_positioner() -> TooltipPositioner<()> {
    TooltipPositioner::new().side_offset(px(4.0))
}

/// Creates Nova's compact inverse Tooltip surface.
pub fn tooltip_popup(id: impl Into<ElementId>, cx: &App) -> TooltipPopup<()> {
    let theme = UiTheme::read(cx).clone();
    TooltipPopup::new()
        .id(id)
        .style_with_state(move |_state, base| {
            base.flex()
                .items_center()
                .gap(px(6.0))
                .rounded(px(6.0))
                .px(px(12.0))
                .py(px(6.0))
                .bg(theme.colors.foreground)
                .text_color(theme.colors.background)
                .font_family(theme.fonts.body.clone())
                .text_size(px(12.0))
                .shadow(vec![
                    BoxShadow::new(px(0.0), px(4.0), theme.colors.foreground.alpha(0.16).into())
                        .blur_radius(px(8.0)),
                ])
        })
}
