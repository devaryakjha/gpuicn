//! Nova surfaces for GPUI Kit's anchored popover.
use super::{
    button::{ButtonSize, ButtonVariant, style_button},
    theme::UiTheme,
};
use gpui_kit::base::Button;
pub use gpui_kit::base::{Popover, PopoverState};
use gpui_kit::{
    App, Div, ElementId, FontWeight, InteractiveElement as _, SharedString, Stateful,
    StatefulInteractiveElement as _, Styled, div,
};

/// Create a popover and supply its trigger and content builder directly.
pub fn popover(id: impl Into<ElementId>) -> Popover {
    Popover::new(id)
}
/// Outline trigger compatible with Kit's selected-state projection.
pub fn popover_trigger(id: impl Into<ElementId>, cx: &App) -> Button {
    style_button(
        Button::new(id),
        false,
        ButtonVariant::Outline,
        ButtonSize::Default,
        UiTheme::read(cx),
    )
}
/// Named Nova content surface. Dismiss through the content builder's `PopoverState`.
pub fn popover_popup(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    cx: &App,
) -> Stateful<Div> {
    let t = UiTheme::read(cx);
    div()
        .id(id)
        .aria_label(label)
        .w(t.space(72.))
        .flex()
        .flex_col()
        .gap(t.space(2.5))
        .rounded(t.radius.lg)
        .border_1()
        .border_color(t.colors.foreground.opacity(0.1))
        .p(t.space(2.5))
        .bg(t.colors.popover)
        .text_color(t.colors.popover_foreground)
        .font_family(t.fonts.body.clone())
        .text_size(t.text(14.))
        .shadow(t.shadows.md.clone())
}
/// Medium-weight popover title.
pub fn popover_title(cx: &App) -> Div {
    let t = UiTheme::read(cx);
    div()
        .font_family(t.fonts.body.clone())
        .font_weight(FontWeight::MEDIUM)
        .text_size(t.text(14.))
        .text_color(t.colors.popover_foreground)
}
/// Muted popover description.
pub fn popover_description(cx: &App) -> Div {
    let t = UiTheme::read(cx);
    div()
        .font_family(t.fonts.body.clone())
        .text_size(t.text(14.))
        .text_color(t.colors.muted_foreground)
}
