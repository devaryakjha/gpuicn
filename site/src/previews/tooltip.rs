//! The tooltip catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn tooltip_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        tooltip(
            "preview.tooltip",
            "Add to library",
            gpui_kit::base::Button::new("tooltip-trigger")
                .accessibility_label("Add to library")
                .px(px(12.))
                .py(px(8.))
                .border_1()
                .border_color(UiTheme::read(cx).colors.border)
                .rounded(px(8.))
                .child("Hover or focus"),
            window,
            cx,
        )
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.tooltip_preview(window, cx).into_any_element()
}
