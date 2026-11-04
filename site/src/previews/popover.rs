//! The popover catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn popover_preview(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap(px(24.))
            .child(
                popover("dimensions")
                    .trigger(popover_trigger("dimensions.trigger", cx).child("Dimensions"))
                    .content(|_, window, cx| {
                        let width = demo_input("dimensions.width", "100%", "Width", window, cx);
                        let height = demo_input("dimensions.height", "25px", "Height", window, cx);
                        popover_popup("dimensions.popup", "Dimensions", cx)
                            .child(popover_title(cx).child("Dimensions"))
                            .child(popover_description(cx).child("Set the dimensions for the layer."))
                            .child(Field::new("width", &width).label("Width"))
                            .child(Field::new("height", &height).label("Height"))
                    }),
            )
            // A profile card: rest the pointer on the avatar, or focus it and
            // press Enter, to open it.
            .child(
                hover_popover("profile", "Priya Nair, online")
                    .trigger(
                        Avatar::new("profile.avatar")
                            .fallback("PN")
                            .status(AvatarStatus::Online),
                    )
                    .content(|_, cx| {
                        popover_popup("profile.card", "Priya Nair", cx)
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(12.))
                                    .child(
                                        Avatar::new("profile.card.avatar")
                                            .size(AvatarSize::Lg)
                                            .fallback("PN")
                                            .status(AvatarStatus::Online),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .child(popover_title(cx).child("Priya Nair"))
                                            .child(popover_description(cx).child("@priya · Design")),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap(px(8.))
                                    .child(Button::new("profile.message").label("Message"))
                                    .child(
                                        Button::new("profile.mention")
                                            .variant(ButtonVariant::Outline)
                                            .label("Mention"),
                                    ),
                            )
                    }),
            )
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.popover_preview(window, cx).into_any_element()
}
