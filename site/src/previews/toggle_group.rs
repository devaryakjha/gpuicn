//! The toggle group catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn toggle_group_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let value =
            window.use_keyed_state("format-values", cx, |_, _| vec![SharedString::from("bold")]);
        ToggleGroup::new("format")
            .aria_label("Text styles")
            .multiple(true)
            .value(value.read(cx).clone())
            .item(
                ToggleGroupItem::new("bold", "bold")
                    .aria_label("Bold")
                    .child("Bold"),
            )
            .item(
                ToggleGroupItem::new("italic", "italic")
                    .aria_label("Italic")
                    .child("Italic"),
            )
            .item(
                ToggleGroupItem::new("underline", "underline")
                    .aria_label("Underline")
                    .child("Underline"),
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
    view.toggle_group_preview(window, cx).into_any_element()
}
