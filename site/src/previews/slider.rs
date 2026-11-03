//! The slider catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn slider_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = window.use_keyed_state("volume", cx, |_, _| {
            gpuicn::slider::SliderState::new()
                .min(0.)
                .max(100.)
                .step(1.)
                .default_value(50.)
        });
        div()
            .w(px(280.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .child("Volume")
            .child(Slider::new(&state).aria_label("Volume"))
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.slider_preview(window, cx).into_any_element()
}
