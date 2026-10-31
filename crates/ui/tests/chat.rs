//! Keyboard and rendering checks for the chat components.
use std::{cell::RefCell, rc::Rc};

use gpui_kit::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, Styled as _,
    TestAppContext, Window, div, px,
};
use gpuicn::{
    attachment::{Attachment, AttachmentStatus, attachment_group},
    command::{Command, CommandEvent, CommandItem, CommandState},
    markdown::{markdown, selection_root},
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

struct Selectable {
    list: Entity<MessageListState>,
    focus: gpui_kit::FocusHandle,
    selected: Rc<RefCell<Option<usize>>>,
    claimed: Rc<RefCell<Vec<String>>>,
    bodies: Rc<RefCell<std::collections::HashMap<usize, gpui_kit::Bounds<gpui_kit::Pixels>>>>,
}
impl Render for Selectable {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        use gpui_kit::base::ElementExt as _;
        let selected = *self.selected.borrow();
        let sink = self.selected.clone();
        let claimed = self.claimed.clone();
        let bodies = self.bodies.clone();
        selection_root("root").child(
            div().w(px(600.)).h(px(400.)).child(
                MessageList::new("selectable", &self.list, move |index, _, cx| {
                    if index == 0 {
                        return day_divider("day", "Today", cx).into_any_element();
                    }
                    let bodies = bodies.clone();
                    Message::new(("post", index))
                        .author("Arya")
                        .highlight(if selected == Some(index) {
                            MessageHighlight::Selected
                        } else {
                            MessageHighlight::None
                        })
                        .child(
                            div()
                                .child(markdown(
                                    ("body", index),
                                    format!("Message number {index}"),
                                    cx,
                                ))
                                .on_prepaint(move |bounds, _, _| {
                                    bodies.borrow_mut().insert(index, bounds);
                                }),
                        )
                        .into_any_element()
                })
                .track_focus(&self.focus)
                .selected(selected)
                .selectable(|index, _| index != 0)
                .on_select(move |index, _, _| *sink.borrow_mut() = index)
                .on_key(move |event, selected, _, _| {
                    if event.keystroke.key == "e" {
                        claimed.borrow_mut().push(format!("edit {selected:?}"));
                        return true;
                    }
                    false
                }),
            ),
        )
    }
}

#[gpui_kit::test]
fn transcript_keyboard_selection_and_copy_across_rows(cx: &mut TestAppContext) {
    cx.update(gpuicn::init);
    let selected = Rc::new(RefCell::new(None));
    let claimed = Rc::new(RefCell::new(Vec::new()));
    let (selected_sink, claimed_sink) = (selected.clone(), claimed.clone());
    let (view, cx) = cx.add_window_view(move |_, cx| Selectable {
        list: cx.new(|cx| MessageListState::new(4, cx)),
        focus: cx.focus_handle(),
        selected: selected_sink,
        claimed: claimed_sink,
        bodies: Default::default(),
    });
    cx.update(|window, cx| {
        window.activate_window();
        window.focus(&view.read(cx).focus.clone(), cx);
        window.draw(cx).clear(cx);
    });
    cx.run_until_parked();

    // Up starts at the newest row; the divider at 0 is skipped.
    cx.simulate_keystrokes("up");
    assert_eq!(*selected.borrow(), Some(3));
    cx.simulate_keystrokes("up up up");
    assert_eq!(*selected.borrow(), Some(1));
    cx.simulate_keystrokes("down");
    assert_eq!(*selected.borrow(), Some(2));
    cx.simulate_keystrokes("e");
    assert_eq!(claimed.borrow().as_slice(), ["edit Some(2)"]);
    cx.simulate_keystrokes("escape");
    assert_eq!(*selected.borrow(), None);

    // A drag from the first body into the last selects across messages.
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let (first, last) = cx.update(|_, cx| {
        let bodies = view.read(cx).bodies.borrow();
        (bodies[&1], bodies[&3])
    });
    let start = gpui_kit::point(first.left() + px(2.), first.center().y);
    let end = gpui_kit::point(last.right() - px(2.), last.center().y);
    cx.simulate_mouse_down(
        start,
        gpui_kit::MouseButton::Left,
        gpui_kit::Modifiers::default(),
    );
    cx.simulate_mouse_move(
        end,
        Some(gpui_kit::MouseButton::Left),
        gpui_kit::Modifiers::default(),
    );
    cx.simulate_mouse_up(
        end,
        gpui_kit::MouseButton::Left,
        gpui_kit::Modifiers::default(),
    );
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let text = cx.update(gpui_kit::base::TextSelection::selected_text);
    assert!(text.contains("Message number 1"), "{text:?}");
    assert!(text.contains("Message number 2"), "{text:?}");
    assert!(text.contains("Message number 3"), "{text:?}");

    cx.simulate_keystrokes("secondary-c");
    let copied = cx.update(|_, cx| cx.read_from_clipboard().and_then(|item| item.text()));
    assert_eq!(copied.as_deref(), Some(text.trim()));
}
