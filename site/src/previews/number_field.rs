//! The number field catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn number_field_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = window.use_keyed_state("quantity", cx, |window, cx| {
            InputState::new(window, cx)
                .default_value("3")
                .min(0.)
                .max(20.)
                .step(1.)
        });
        NumberField::new(&state).aria_label("Quantity").w(px(200.))
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.number_field_preview(window, cx).into_any_element()
}
