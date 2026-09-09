//! Nova presentation for GPUI Kit's controlled collapsible region.
use super::theme::UiTheme;
use gpui_kit::base::Button;
pub use gpui_kit::base::Collapsible;
use gpui_kit::{
    App, Div, ElementId, InteractiveElement as _, StatefulInteractiveElement as _, Styled, div,
};

/// Creates a region whose caller sets `.open(...)` and supplies `.content(...)`.
pub fn collapsible(cx: &App) -> Collapsible {
    Collapsible::new()
        .flex()
        .flex_col()
        .font_family(UiTheme::read(cx).fonts.body.clone())
}
/// Creates an accessible trigger. Update the caller's open state in its click handler.
pub fn collapsible_trigger(id: impl Into<ElementId>, open: bool, cx: &App) -> Button {
    let theme = UiTheme::read(cx);
    let colors = theme.colors;
    let ring = theme.focus_ring();
    Button::new(id)
        .aria_expanded(open)
        .px(theme.space(2.5))
        .py(theme.space(1.5))
        .rounded(theme.radius.lg)
        .text_size(theme.text(14.))
        .text_color(colors.foreground)
        .cursor_pointer()
        .hover(move |s| s.bg(colors.muted))
        .focus_visible(move |s| {
            s.bg(colors.background)
                .border_1()
                .border_color(colors.ring)
                .shadow(ring.clone())
        })
}
/// Creates the disclosure content's text treatment.
pub fn collapsible_content(cx: &App) -> Div {
    let theme = UiTheme::read(cx);
    div()
        .pt(theme.space(2.))
        .overflow_hidden()
        .font_family(theme.fonts.body.clone())
        .text_size(theme.text(14.))
        .text_color(theme.colors.foreground)
}
