//! Nova form layout. The application owns values, validation and submission.

use super::theme::UiTheme;
use gpui_kit::{
    App, Div, ElementId, InteractiveElement as _, Role, Stateful, StatefulInteractiveElement as _,
    Styled, div,
};

/// Groups fields semantically without an implicit global form context.
/// Subscribe to each input's `InputEvent` and submit through the application's state.
pub fn form(id: impl Into<ElementId>, cx: &App) -> Stateful<Div> {
    let theme = UiTheme::read(cx);
    div()
        .id(id)
        .role(Role::Form)
        .flex()
        .flex_col()
        .w_full()
        .gap(theme.space(6.))
        .font_family(theme.fonts.body.clone())
        .text_size(theme.text(14.))
        .text_color(theme.colors.foreground)
}
