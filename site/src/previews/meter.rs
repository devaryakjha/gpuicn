//! The meter catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn meter_preview(&self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .max_w(px(320.0))
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child("Storage used · 68%")
            .child(
                Meter::new("preview.meter")
                    .value(68.0)
                    .aria_label("Storage used: 68 percent"),
            )
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.meter_preview(window, cx).into_any_element()
}
