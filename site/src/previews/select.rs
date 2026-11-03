//! The select catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn select_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = window.use_keyed_state("select_preview", cx, |window, cx| {
            SelectState::new(
                [
                    SelectItem::new("apple", "Apple"),
                    SelectItem::new("banana", "Banana"),
                    SelectItem::new("orange", "Orange").disabled(true),
                    SelectItem::new("pear", "Pear"),
                ],
                window,
                cx,
            )
        });
        Select::new(&state)
            .aria_label("Fruit")
            .placeholder("Select a fruit…")
            .w(px(240.))
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.select_preview(window, cx).into_any_element()
}
