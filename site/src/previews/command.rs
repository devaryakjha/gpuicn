//! The command catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn command_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.entity().downgrade();
        let handle = window
            .use_keyed_state("command.handle", cx, |_, _| DialogHandle::new(false))
            .read(cx)
            .clone();
        let state = window
            .use_keyed_state("command.state", cx, |window, cx| {
                let state = cx.new(|cx| {
                    let mut state = CommandState::new(command_items(), window, cx);
                    // Direct messages lead with an avatar; channels with unread
                    // messages end with a count.
                    state.set_leading(
                        |item, _, _| {
                            (item.group.as_deref() == Some("Direct messages")).then(|| {
                                Avatar::new(gpui_kit::ElementId::Name(
                                    format!("command.avatar.{}", item.value).into(),
                                ))
                                .size(AvatarSize::Sm)
                                .w(px(20.))
                                .h(px(20.))
                                .fallback(item.label.chars().take(1).collect::<String>())
                                .into_any_element()
                            })
                        },
                        cx,
                    );
                    state.set_trailing(
                        |item, _, _| {
                            let unread = match item.value.as_ref() {
                                "town-square" => 3,
                                "gpuicn" => 128,
                                _ => return None,
                            };
                            Some(Badge::count(unread, 99).into_any_element())
                        },
                        cx,
                    );
                    state
                });
                let handle = handle.clone();
                let subscription =
                    cx.subscribe_in(&state, window, move |_, _, event, window, cx| {
                        let action = match event {
                            CommandEvent::Confirm(value) => format!("Opened {value}"),
                            CommandEvent::Cancel => "Closed".into(),
                            CommandEvent::Query(_) => return,
                        };
                        handle.close(window, cx);
                        let _ = owner.update(cx, |this, cx| {
                            this.demo_action = action;
                            cx.notify();
                        });
                    });
                (state, subscription)
            })
            .read(cx)
            .0
            .clone();
        let open = handle.clone();
        let focus = state.clone();
        div()
            .w_full()
            .max_w(px(420.))
            .flex()
            .flex_col()
            .items_center()
            .gap(px(12.))
            .child(
                Button::new("command.open")
                    .variant(ButtonVariant::Outline)
                    .on_click(move |_, window, cx| {
                        open.open(window, cx);
                        focus.update(cx, |state, cx| state.focus(window, cx));
                    })
                    .child("Quick switcher")
                    .child(Kbd::shortcut("secondary-k")),
            )
            .child(dialog(
                "command.dialog",
                &handle,
                command_dialog_popup("command.popup", &state, cx),
                window,
                cx,
            ))
    }
}

fn command_items() -> Vec<gpuicn::command::CommandItem> {
    use gpuicn::command::CommandItem;
    vec![
        CommandItem::new("town-square", "Town Square")
            .group("Channels")
            .icon(LucideIcon::Hash),
        CommandItem::new("off-topic", "Off-Topic")
            .group("Channels")
            .icon(LucideIcon::Hash),
        CommandItem::new("gpuicn", "gpuicn")
            .group("Channels")
            .icon(LucideIcon::Lock),
        CommandItem::new("arya", "Arya")
            .group("Direct messages")
            .icon(LucideIcon::User),
        CommandItem::new("sam", "Sam")
            .group("Direct messages")
            .icon(LucideIcon::User),
        CommandItem::new("threads", "Threads")
            .group("Views")
            .icon(LucideIcon::MessagesSquare)
            .shortcut("secondary-shift-t"),
        CommandItem::new("preferences", "Preferences")
            .group("Commands")
            .icon(LucideIcon::Settings)
            .shortcut("secondary-,"),
        CommandItem::new("theme", "Toggle dark mode")
            .group("Commands")
            .icon(LucideIcon::Moon)
            .keywords(["theme", "appearance"]),
        CommandItem::new("archive", "Browse archived channels")
            .group("Commands")
            .icon(LucideIcon::Archive)
            .disabled(true),
    ]
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.command_preview(window, cx).into_any_element()
}
