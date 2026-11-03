//! The scroll area catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn scroll_area_preview(&self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        ScrollArea::new("tags")
            .aria_label("Tags")
            .w(px(240.))
            .h(px(240.))
            .children((1..=50).map(|i| {
                div()
                    .px(px(12.))
                    .py(px(8.))
                    .child(format!("Version 1.0.{i}"))
            }))
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.scroll_area_preview(window, cx).into_any_element()
}
