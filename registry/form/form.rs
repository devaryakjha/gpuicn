//! shadcn-style form composition backed by Base GPUI Form primitives.

use base_gpui::form::Form;
use gpui::{App, ElementId, Styled, px};

use super::theme::UiTheme;

pub use base_gpui::form::{
    FormErrors, FormSubmitAction, FormSubmitDetails, FormSubmitReason, FormValidateAction,
    FormValue, FormValues,
};

/// Creates a styled Form with Base GPUI submit, validation, and focus behavior.
pub fn form(id: impl Into<ElementId>, cx: &App) -> Form {
    let theme = UiTheme::read(cx).clone();
    Form::new().id(id).style_with_state(move |_state, base| {
        base.flex()
            .flex_col()
            .w_full()
            .gap(px(24.0))
            .font_family(theme.fonts.body.clone())
            .text_size(px(14.0))
            .text_color(theme.colors.foreground)
    })
}
