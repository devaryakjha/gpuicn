//! The radio group catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn radio_group_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let value = window.use_keyed_state("radio", cx, |_, _| SharedString::from("comfortable"));
        RadioGroup::new("density")
            .aria_label("Density")
            .value(value.read(cx).clone())
            .item(RadioItem::new("compact", "compact").label("Compact"))
            .item(RadioItem::new("comfortable", "comfortable").label("Comfortable"))
            .item(
                RadioItem::new("spacious", "spacious")
                    .label("Spacious")
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
    view.radio_group_preview(window, cx).into_any_element()
}
