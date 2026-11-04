//! The avatar catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn avatar_preview(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = UiTheme::read(cx);
        // Pinned to Arya's supplied revision; AJ remains the loading/error fallback.
        let image = "https://raw.githubusercontent.com/devaryakjha/devaryakjha/6526e3d7415b2fb573ba3da4523b5c6948aa5d08/avatar.png";
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(28.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(28.))
                    .child(
                        Avatar::new("preview.avatar.image")
                            .image(image)
                            .aria_label("Arya")
                            .fallback("AJ"),
                    )
                    .child(
                        Avatar::new("preview.avatar.fallback")
                            .aria_label("Initials fallback")
                            .fallback("AJ"),
                    )
                    .child(
                        div().flex().items_center().children(
                            [
                                ("preview.avatar.group.arya", "AJ", true),
                                ("preview.avatar.group.member", "SK", false),
                                ("preview.avatar.group.count", "+3", false),
                            ]
                            .into_iter()
                            .enumerate()
                            .map(
                                |(index, (id, initials, has_image))| {
                                    div()
                                        .rounded_full()
                                        .border_2()
                                        .border_color(theme.colors.background)
                                        .when(index > 0, |item| item.ml(px(-10.)))
                                        .child(
                                            Avatar::new(id)
                                                .when(has_image, |avatar| avatar.image(image))
                                                .aria_label(if initials == "+3" {
                                                    "3 more members"
                                                } else {
                                                    initials
                                                })
                                                .fallback(initials),
                                        )
                                },
                            ),
                        ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(12.))
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(theme.colors.muted_foreground)
                            .child("Presence"),
                    )
                    .child(
                        div().flex().items_center().gap(px(16.)).children(
                            [
                                ("preview.avatar.online", "Ana Lopez", "AL", AvatarStatus::Online),
                                ("preview.avatar.away", "Sam Kim", "SK", AvatarStatus::Away),
                                ("preview.avatar.dnd", "Priya Nair", "PN", AvatarStatus::Dnd),
                                ("preview.avatar.offline", "Lee Chen", "LC", AvatarStatus::Offline),
                            ]
                            .into_iter()
                            .map(|(id, name, initials, status)| {
                                Avatar::new(id)
                                    .aria_label(name)
                                    .fallback(initials)
                                    .status(status)
                            }),
                        ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(12.))
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(theme.colors.muted_foreground)
                            .child("Sizes"),
                    )
                    .child(
                        div().flex().items_center().gap(px(12.)).children(
                            [
                                ("preview.avatar.small", AvatarSize::Sm),
                                ("preview.avatar.default", AvatarSize::Default),
                                ("preview.avatar.large", AvatarSize::Lg),
                            ]
                            .into_iter()
                            .map(|(id, size)| {
                                Avatar::new(id)
                                    .size(size)
                                    .image(image)
                                    .aria_label("Arya")
                                    .fallback("AJ")
                                    .status(AvatarStatus::Online)
                            }),
                        ),
                    ),
            )
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.avatar_preview(window, cx).into_any_element()
}
