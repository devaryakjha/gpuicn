//! The shadcn Nova Tooltip visual port.
//!
//! Base GPUI owns the provider delay group, hover/focus lifecycle and collision behavior.

pub use base_gpui::tooltip::{
    TooltipAlign, TooltipPopup, TooltipPortal, TooltipPositioner, TooltipProvider, TooltipRoot,
    TooltipSide, TooltipTrigger, TooltipViewport,
};
use gpui::{App, ElementId, Styled, px};

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
pub fn tooltip_positioner(cx: &App) -> TooltipPositioner<()> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    TooltipPositioner::new().side_offset(spacing * 1_f32)
}

/// Creates Nova's compact inverse Tooltip surface.
pub fn tooltip_popup(id: impl Into<ElementId>, cx: &App) -> TooltipPopup<()> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    TooltipPopup::new()
        .id(id)
        .style_with_state(move |_state, base| {
            base.flex()
                .items_center()
                .gap(spacing * 1.5_f32)
                .rounded(theme.radius.sm)
                .px(spacing * 3_f32)
                .py(spacing * 1.5_f32)
                .bg(theme.colors.foreground)
                .text_color(theme.colors.background)
                .font_family(theme.fonts.body.clone())
                .text_size(px(12.0) * text_scale)
        })
}

/// Creates a tooltip view for GPUI's native `.tooltip(...)` attachment point.
/// Use this on an existing interactive control to avoid nesting another button
/// and tab stop. The host control must retain its own full accessible name.
pub fn text_tooltip(label: gpui::SharedString, cx: &mut App) -> gpui::AnyView {
    use gpui::AppContext as _;
    cx.new(|_| TextTooltip(label)).into()
}
struct TextTooltip(gpui::SharedString);
impl gpui::Render for TextTooltip {
    fn render(
        &mut self,
        _: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        use gpui::ParentElement as _;
        let theme = UiTheme::read(cx);
        let spacing = theme.spacing.unit;
        let text_scale = theme.text_scale;
        gpui::div()
            .rounded(theme.radius.sm)
            .px(spacing * 3_f32)
            .py(spacing * 1.5_f32)
            .bg(theme.colors.foreground)
            .text_color(theme.colors.background)
            .font_family(theme.fonts.body.clone())
            .text_size(px(12.) * text_scale)
            .child(self.0.clone())
    }
}
