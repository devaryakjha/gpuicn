//! Keyboard and rendering checks for the chat components.
use std::{cell::RefCell, rc::Rc};

use gpui_kit::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, Styled as _,
    TestAppContext, Window, div, px,
};
use gpuicn::{
    attachment::{Attachment, AttachmentStatus, attachment_group},
    command::{Command, CommandEvent, CommandItem, CommandState},
    message::{
        Message, MessageHighlight, MessageStatus, Reaction, day_divider, new_messages_divider,
    },
    message_list::{MessageList, MessageListState},
    title_bar::TitleBar,
};

struct Palette {
    state: Entity<CommandState>,
}
impl Render for Palette {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(480.)).child(Command::new(&self.state))
    }
}

#[gpui_kit::test]
fn command_filters_navigates_confirms_and_cancels(cx: &mut TestAppContext) {
    cx.update(gpuicn::init);
    let events = Rc::new(RefCell::new(Vec::new()));
    let (view, cx) = cx.add_window_view(|window, cx| {
        let state = cx.new(|cx| {
            CommandState::new(
                [
                    CommandItem::new("general", "General").group("Channels"),
                    CommandItem::new("random", "Random").group("Channels"),
                    CommandItem::new("archived", "Archived")
                        .group("Channels")
                        .disabled(true),
                    CommandItem::new("reload", "Reload window")
                        .group("Commands")
                        .keywords(["refresh"]),
                ],
                window,
                cx,
            )
        });
        let sink = events.clone();
        cx.subscribe(&state, move |_, _, event: &CommandEvent, _| {
            sink.borrow_mut().push(event.clone())
        })
        .detach();
        Palette { state }
    });
    cx.update(|window, cx| {
        window.activate_window();
        let state = view.read(cx).state.clone();
        state.update(cx, |state, cx| state.focus(window, cx));
        window.draw(cx).clear(cx);
    });
    cx.simulate_keystrokes("down enter");
    assert_eq!(
        events.borrow().last(),
        Some(&CommandEvent::Confirm("random".into()))
    );
    // Down wraps past the disabled item to the next group.
    cx.simulate_keystrokes("down enter");
    assert_eq!(
        events.borrow().last(),
        Some(&CommandEvent::Confirm("reload".into()))
    );

    cx.simulate_input("refr");
    cx.update(|_, cx| assert_eq!(view.read(cx).state.read(cx).matched_count(), 1));
    cx.simulate_keystrokes("enter");
    assert_eq!(
        events.borrow().last(),
        Some(&CommandEvent::Confirm("reload".into()))
    );

    cx.simulate_keystrokes("escape");
    cx.update(|_, cx| assert!(view.read(cx).state.read(cx).query(cx).is_empty()));
    assert_ne!(events.borrow().last(), Some(&CommandEvent::Cancel));
    cx.simulate_keystrokes("escape");
    assert_eq!(events.borrow().last(), Some(&CommandEvent::Cancel));
}

struct Transcript {
    list: Entity<MessageListState>,
    toggled: Rc<RefCell<Vec<String>>>,
}
impl Render for Transcript {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let toggled = self.toggled.clone();
        div()
            .w(px(600.))
            .h(px(400.))
            .flex()
            .flex_col()
            .child(TitleBar::new().child("Matty"))
            .child(
                MessageList::new("transcript", &self.list, move |index, _, cx| {
                    let toggled = toggled.clone();
                    match index {
                        0 => day_divider("day", "Today", cx).into_any_element(),
                        1 => new_messages_divider("new", "New messages", cx).into_any_element(),
                        index => Message::new(("post", index))
                            .author("Arya")
                            .timestamp("10:42 AM")
                            .continuation(index % 3 == 0)
                            .highlight(if index == 4 {
                                MessageHighlight::Mention
                            } else {
                                MessageHighlight::None
                            })
                            .status(if index == 5 {
                                MessageStatus::Failed
                            } else {
                                MessageStatus::Sent
                            })
                            .on_retry(|_, _, _| {})
                            .reactions([Reaction::new("🎉", 3, true)])
                            .on_reaction(move |emoji, _, _| {
                                toggled.borrow_mut().push(emoji.to_string())
                            })
                            .child(format!("Message {index}"))
                            .child(attachment_group(
                                ("files", index),
                                [Attachment::new(("file", index), "plan.pdf")
                                    .meta("PDF · 2.4 MB")
                                    .status(AttachmentStatus::Uploading(0.4))
                                    .on_remove(|_, _, _| {})],
                                cx,
                            ))
                            .into_any_element(),
                    }
                })
                .unseen(2),
            )
    }
}

#[gpui_kit::test]
fn transcript_renders_rows_and_toggles_reactions(cx: &mut TestAppContext) {
    cx.update(gpuicn::init);
    let toggled = Rc::new(RefCell::new(Vec::new()));
    let sink = toggled.clone();
    let (view, cx) = cx.add_window_view(move |_, cx| Transcript {
        list: cx.new(|cx| MessageListState::new(200, cx)),
        toggled: sink,
    });
    for _ in 0..2 {
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.run_until_parked();
    }
    view.update(cx, |view, cx| {
        view.list.update(cx, |list, cx| {
            list.prepend(50, cx);
            list.append(3, cx);
        })
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.update(|_, cx| assert_eq!(view.read(cx).list.read(cx).item_count(), 253));
    assert!(toggled.borrow().is_empty());
}
