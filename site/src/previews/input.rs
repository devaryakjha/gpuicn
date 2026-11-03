//! The input catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn input_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let email = demo_input("input.email", "", "Email", window, cx);
        let readonly = demo_input("input.readonly", "read-only@example.com", "", window, cx);
        let disabled = demo_input("input.disabled", "disabled@example.com", "", window, cx);
        div()
            .w(px(320.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(Input::new(&email).aria_label("Email address"))
            .child(
                Input::new(&readonly)
                    .aria_label("Read-only email")
                    .read_only(true),
            )
            .child(
                Input::new(&disabled)
                    .aria_label("Disabled email")
                    .disabled(true),
            )
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.input_preview(window, cx).into_any_element()
}
