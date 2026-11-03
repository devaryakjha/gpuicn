//! The combobox catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn combobox_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = window.use_keyed_state("combobox_preview", cx, |window, cx| {
            let mut state = SelectState::new(
                [
                    SelectItem::new("apple", "Apple"),
                    SelectItem::new("banana", "Banana"),
                    SelectItem::new("orange", "Orange").disabled(true),
                    SelectItem::new("pear", "Pear"),
                ],
                window,
                cx,
            );
            state.set_value(Some("pear".into()), window, cx);
            state
        });
        gpuicn::combobox::combobox(&state)
            .aria_label("Fruit")
            .placeholder("Search a fruit…")
            .w(px(240.))
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.combobox_preview(window, cx).into_any_element()
}
