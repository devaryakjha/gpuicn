//! The progress catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn progress_preview(&self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .max_w(px(320.0))
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child("Uploading · 64%")
            .child(
                Progress::new("preview.progress")
                    .value(64.0)
                    .label("Uploading…"),
            )
            .child("Waiting · indeterminate")
            .child(
                Progress::new("preview.progress.indeterminate")
                    .indeterminate()
                    .label("Waiting…"),
            )
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.progress_preview(window, cx).into_any_element()
}
