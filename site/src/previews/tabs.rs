//! The tabs catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn tabs_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let value = window.use_keyed_state("tabs", cx, |_, _| SharedString::from("account"));
        let selected = value.read(cx).clone();
        div()
            .w(px(320.))
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(
                Tabs::new("settings-tabs")
                    .aria_label("Settings")
                    .selected(selected.clone())
                    .item(Tab::new("account", "account", "Account"))
                    .item(Tab::new("password", "password", "Password"))
                    .item(Tab::new("billing", "billing", "Billing").disabled(true))
                    .on_change(move |next, _, cx| {
                        value.update(cx, |value, cx| {
                            *value = next;
                            cx.notify();
                        })
                    }),
            )
            .child(tabs_content("settings-panel", selected.clone(), cx).child(
                if selected.as_ref() == "account" {
                    "Manage your account details."
                } else {
                    "Change your account password."
                },
            ))
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.tabs_preview(window, cx).into_any_element()
}
