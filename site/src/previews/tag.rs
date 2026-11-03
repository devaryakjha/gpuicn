//! The tag catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn tag_preview(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
                    .child(Tag::new("preview.tag.online", "Online").dot(gpui_kit::rgb(0x22c55e)))
                    .child(Tag::new("preview.tag.away", "Away").dot(gpui_kit::rgb(0xf59e0b)))
                    .child(
                        Tag::new("preview.tag.bot", "Bot")
                            .variant(TagVariant::Outline)
                            .icon(LucideIcon::Bot),
                    )
                    .child(
                        Tag::new("preview.tag.deactivated", "Deactivated")
                            .variant(TagVariant::Destructive),
                    ),
            )
            .when(!self.checked, |root| {
                root.child(
                    Tag::new("preview.tag.filter", "from: @arya")
                        .variant(TagVariant::Outline)
                        .on_remove(cx.listener(|this, _, _, cx| {
                            this.checked = true;
                            this.demo_action = "Removed filter".into();
                            cx.notify();
                        })),
                )
            })
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.tag_preview(window, cx).into_any_element()
}
