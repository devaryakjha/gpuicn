//! The separator catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn separator_preview(&self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .max_w(px(320.0))
            .flex()
            .flex_col()
            .gap(px(12.0))
            .child("gpuicn")
            .child(Separator::new("preview.separator"))
            .child("Open-code components for GPUI")
            .child(
                div()
                    .flex()
                    .items_center()
                    .h(px(20.))
                    .gap(px(12.))
                    .child("Docs")
                    .child(Separator::new("preview.separator.vertical").vertical())
                    .child("Components"),
            )
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.separator_preview(window, cx).into_any_element()
}
