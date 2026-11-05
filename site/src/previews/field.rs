//! The field catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn field_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let username = demo_input("field.username", "", "e.g. ada", window, cx);
        let email = demo_input("field.email", "", "you@example.com", window, cx);
        let notify = window.use_keyed_state("field.notify.checked", cx, |_, _| true);
        let toggled = notify.clone();
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
            .child(
                Field::from_control(
                    "notify",
                    Checkbox::new("field.notify")
                        .checked(*notify.read(cx))
                        .on_change(move |checked, _, _, cx| {
                            toggled.update(cx, |notify, cx| {
                                *notify = checked;
                                cx.notify();
                            })
                        }),
                )
                .label("Notify the channel")
                .description("Posts a message when the dialog is submitted."),
            )
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.field_preview(window, cx).into_any_element()
}
