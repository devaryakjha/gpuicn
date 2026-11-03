//! The checkbox group catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn checkbox_group_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let value = window.use_keyed_state("checkbox-group", cx, |_, _| {
            vec![SharedString::from("email")]
        });
        CheckboxGroup::new("notifications")
            .aria_label("Notifications")
            .value(value.read(cx).clone())
            .item(CheckboxGroupItem::new("email", "email").label("Email notifications"))
            .item(CheckboxGroupItem::new("push", "push").label("Push notifications"))
            .item(
                CheckboxGroupItem::new("sms", "sms")
                    .label("SMS notifications")
                    .disabled(true),
            )
            .on_change(move |next, _, cx| {
                value.update(cx, |value, cx| {
                    *value = next;
                    cx.notify();
                })
            })
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.checkbox_group_preview(window, cx).into_any_element()
}
