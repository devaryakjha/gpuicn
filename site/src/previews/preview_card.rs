//! The preview card catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn preview_card_preview(&self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        preview_card("profile-card")
            .trigger(
                Button::new("profile-card.trigger")
                    .variant(ButtonVariant::Link)
                    .label("@gpuicn"),
            )
            .content(|_, _, cx| {
                preview_card_popup("profile-card.popup", cx).child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(8.))
                        .child("gpuicn")
                        .child("Open-code Nova components for native GPUI applications."),
                )
            })
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.preview_card_preview(window, cx).into_any_element()
}
