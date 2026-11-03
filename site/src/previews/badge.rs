//! The badge catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn badge_preview(&self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(16.0))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .justify_center()
                    .gap(px(8.0))
                    .child(Badge::new("Default"))
                    .child(Badge::new("Secondary").variant(BadgeVariant::Secondary))
                    .child(Badge::new("Destructive").variant(BadgeVariant::Destructive))
                    .child(Badge::new("Outline").variant(BadgeVariant::Outline))
                    .child(Badge::new("Ghost").variant(BadgeVariant::Ghost)),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .justify_center()
                    .gap(px(8.0))
                    .child(Badge::count(3, 99))
                    .child(Badge::count(42, 99).variant(BadgeVariant::Secondary))
                    .child(Badge::count(250, 99).variant(BadgeVariant::Destructive))
                    .child(
                        Badge::new("Verified")
                            .variant(BadgeVariant::Outline)
                            .icon(LucideIcon::BadgeCheck),
                    ),
            )
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.badge_preview(window, cx).into_any_element()
}
