//! The message catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn message_preview(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let image = "https://raw.githubusercontent.com/devaryakjha/devaryakjha/6526e3d7415b2fb573ba3da4523b5c6948aa5d08/avatar.png";
        let reacted = self.checked;
        let toggle = cx.listener(|this, emoji: &SharedString, _, cx| {
            this.checked = !this.checked;
            this.demo_action = format!("Toggled {emoji}");
            cx.notify();
        });
        let actions = |id: &'static str, cx: &App| {
            message_actions(cx)
                .child(message_action(
                    (id, 0usize),
                    LucideIcon::FaceSlightlySmilingPlus,
                    "Add reaction",
                    cx,
                ))
                .child(message_action(
                    (id, 1usize),
                    LucideIcon::Reply,
                    "Reply in thread",
                    cx,
                ))
                .child(message_action(
                    (id, 2usize),
                    LucideIcon::Ellipsis,
                    "More actions",
                    cx,
                ))
        };
        let view = cx.entity().downgrade();
        let open_profile = |name: &'static str| {
            let view = view.clone();
            move |_: &gpui_kit::ClickEvent, _: gpui_kit::Bounds<gpui_kit::Pixels>, _: &mut Window, cx: &mut App| {
                view.update(cx, |this, cx| {
                    this.demo_action = format!("Opened {name}'s profile");
                    cx.notify();
                })
                .ok();
            }
        };
        div()
            .size_full()
            .flex()
            .flex_col()
            .pt(px(12.))
            .child(day_divider("message.day", "Today", cx))
            .child(
                Message::new("message.first")
                    .avatar(
                        Avatar::new("message.first.avatar")
                            .image(image)
                            .fallback("AJ"),
                    )
                    .author("Arya")
                    .on_author_click(open_profile("Arya"))
                    .timestamp("10:42 AM")
                    .timestamp_tooltip("Tuesday, 4 November 2026 at 10:42")
                    .on_gutter_click(cx.listener(|this, _, _, cx| {
                        this.demo_action = "Selected the first message".into();
                        cx.notify();
                    }))
                    .actions(actions("message.first", cx))
                    .reactions([
                        Reaction::new("🎉", if reacted { 4 } else { 3 }, reacted).tooltip(
                            if reacted {
                                "You, Sam, Lee and Kai"
                            } else {
                                "Sam, Lee and Kai"
                            },
                        ),
                        Reaction::new("👀", 1, false).tooltip("Sam"),
                        Reaction::new(":arya:", 2, false)
                            .image(image)
                            .tooltip("Sam and Lee reacted with :arya:"),
                    ])
                    .aria_description("Arya, 10:42 AM, 3 reactions")
                    .on_reaction(toggle)
                    .on_add_reaction(|_, _, _| {})
                    .child("gpuicn 0.5.0 is out. Chat components are next."),
            )
            .child(
                Message::new("message.second")
                    .continuation(true)
                    .gutter_time("10:43")
                    .timestamp_tooltip("Tuesday, 4 November 2026 at 10:43")
                    .on_gutter_click(cx.listener(|this, _, _, cx| {
                        this.demo_action = "Selected the second message".into();
                        cx.notify();
                    }))
                    .actions(actions("message.second", cx))
                    .child("Hover a row to see its actions and time."),
            )
            .child(new_messages_divider("message.new", "New messages", cx))
            .child(
                Message::new("message.mention")
                    .avatar(Avatar::new("message.mention.avatar").fallback("SK"))
                    .author("Sam")
                    .timestamp("11:05 AM")
                    .header_child(
                        Tag::new("message.mention.bot", "Bot").variant(TagVariant::Outline),
                    )
                    .highlight(MessageHighlight::Mention)
                    .actions(actions("message.mention", cx))
                    .child("@arya can you review the Matty plan?"),
            )
            .child(
                Message::new("message.permalink")
                    .continuation(true)
                    .gutter_time("11:05")
                    .highlight(MessageHighlight::Permalink)
                    .child("Opened from a link: the permalink target stays tinted."),
            )
            .child(
                Message::new("message.failed")
                    .avatar(
                        Avatar::new("message.failed.avatar")
                            .image(image)
                            .fallback("AJ"),
                    )
                    .author("Arya")
                    .timestamp("11:06 AM")
                    .status(MessageStatus::Failed)
                    .on_retry(cx.listener(|this, _, _, cx| {
                        this.demo_action = "Retried".into();
                        cx.notify();
                    }))
                    .on_delete(|_, _, _| {})
                    .child("On it."),
            )
            .child(day_divider("message.compact-day", "Compact", cx))
            .child(
                Message::new("message.compact")
                    .density(MessageDensity::Compact)
                    .author("Sam")
                    .on_author_click(open_profile("Sam"))
                    .timestamp("11:07")
                    .actions(actions("message.compact", cx))
                    .child("Compact density keeps one line per post."),
            )
            .child(
                Message::new("message.compact-follow")
                    .density(MessageDensity::Compact)
                    .continuation(true)
                    .gutter_time("11:08")
                    .child("The time stays in the gutter."),
            )
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.message_preview(window, cx).into_any_element()
}
