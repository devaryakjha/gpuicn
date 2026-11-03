//! The field catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn field_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let username = demo_input("field.username", "", "e.g. ada", window, cx);
        let email = demo_input("field.email", "", "you@example.com", window, cx);
        div()
            .w(px(280.))
            .flex()
            .flex_col()
            .gap(px(20.))
            .child(
                Field::new("username", &username)
                    .label("Username")
                    .description("Visible on your public profile."),
            )
            .child(
                Field::new("email", &email)
                    .label("Email")
                    .error("Enter a valid email address."),
            )
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.field_preview(window, cx).into_any_element()
}
