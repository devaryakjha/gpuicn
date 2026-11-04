//! The message list catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn message_list_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let total = window.use_keyed_state("message-list.total", cx, |_, _| 400usize);
        let unseen = window.use_keyed_state("message-list.unseen", cx, |_, _| 0usize);
        let selected = window.use_keyed_state("message-list.selected", cx, |_, _| None::<usize>);
        let focus = window
            .use_keyed_state("message-list.focus", cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let state = window
            .use_keyed_state("message-list.state", cx, |_, cx| {
                cx.new(|cx| MessageListState::new(400, cx))
            })
            .read(cx)
            .clone();
        if !state.read(cx).is_scrolled_up() && *unseen.read(cx) > 0 {
            unseen.update(cx, |unseen, _| *unseen = 0);
        }
        let theme = UiTheme::read(cx).clone();
        let add = {
            let state = state.clone();
            let total = total.clone();
            let unseen = unseen.clone();
            move |_: &gpui_kit::ClickEvent, _: &mut Window, cx: &mut App| {
                total.update(cx, |total, _| *total += 1);
                if state.read(cx).is_scrolled_up() {
                    unseen.update(cx, |unseen, _| *unseen += 1);
                }
                state.update(cx, |state, cx| {
                    state.append(1, cx);
                });
            }
        };
        let older = {
            let state = state.clone();
            let total = total.clone();
            move |_: &gpui_kit::ClickEvent, _: &mut Window, cx: &mut App| {
                total.update(cx, |total, _| *total += 50);
                state.update(cx, |state, cx| {
                    state.prepend(50, cx);
                });
            }
        };
        let count = *total.read(cx);
        let current = *selected.read(cx);
        let select = selected.clone();
        let owner = cx.entity().downgrade();
        selection_root("message-list.selection")
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(8.))
                    .h(px(48.))
                    .px(px(16.))
                    .border_b_1()
                    .border_color(theme.colors.border)
                    .child(
                        div()
                            .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                            .child(format!("# general · {count} messages")),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .child(
                                Button::new("message-list.older")
                                    .variant(ButtonVariant::Outline)
                                    .size(ButtonSize::Sm)
                                    .on_click(older)
                                    .label("Load older"),
                            )
                            .child(
                                Button::new("message-list.add")
                                    .size(ButtonSize::Sm)
                                    .on_click(add)
                                    .label("New message"),
                            ),
                    ),
            )
            .child(
                div().flex_1().min_h_0().child(
                    MessageList::new("message-list", &state, move |index, _, cx| {
                        transcript_row(index, current, cx)
                    })
                    .unseen(*unseen.read(cx))
                    .on_jump({
                        let state = state.clone();
                        move |_, cx| state.update(cx, |state, cx| state.scroll_to_end(cx))
                    })
                    .track_focus(&focus)
                    .selected(current)
                    .selectable(|index, _| !index.is_multiple_of(60))
                    .on_select(move |index, _, cx| {
                        select.update(cx, |selected, _| *selected = index);
                        let _ = owner.update(cx, |this, cx| {
                            this.demo_action = match index {
                                Some(index) => format!("Selected message {index}"),
                                None => "Cleared the selection".into(),
                            };
                            cx.notify();
                        });
                    }),
                ),
            )
    }
}

/// One synthetic transcript row: day dividers every 60 rows and new sender
/// groups every 4, with varied heights to exercise variable-height layout.
fn transcript_row(index: usize, selected: Option<usize>, cx: &App) -> gpui_kit::AnyElement {
    use gpuicn::markdown::selectable_text;
    use gpuicn::message::{Message, MessageHighlight, Reaction, day_divider};
    const AUTHORS: [(&str, &str); 3] = [("Arya", "AJ"), ("Sam", "SK"), ("Lee", "LW")];
    const LINES: [&str; 5] = [
        "Pushed the fix.",
        "Variable-height rows keep their place when older history loads above them.",
        "Scroll up, then press New message: the pill counts what arrived while you were away.",
        "Looks good to me.",
        "Message lists only render the rows on screen, so long channels stay fast. This one wraps onto a second line at narrow widths.",
    ];
    if index.is_multiple_of(60) {
        return day_divider(
            ("transcript.day", index),
            format!("Day {}", index / 60 + 1),
            cx,
        )
        .into_any_element();
    }
    let (author, initials) = AUTHORS[(index / 4) % AUTHORS.len()];
    let first = index % 4 == 1 || index % 60 == 1;
    Message::new(("transcript", index))
        .continuation(!first)
        .avatar(Avatar::new(("transcript.avatar", index)).fallback(initials))
        .author(author)
        .timestamp(format!("{}:{:02}", 9 + index / 60 % 8, index % 60))
        .gutter_time(format!("{}:{:02}", 9 + index / 60 % 8, index % 60))
        .when(selected == Some(index), |message| {
            message.highlight(MessageHighlight::Selected)
        })
        .when(index.is_multiple_of(7), |message| {
            message.reactions([Reaction::new("👍", index % 5 + 1, false)])
        })
        .child(selectable_text(
            ("transcript.body", index),
            LINES[index % LINES.len()],
            cx,
        ))
        .into_any_element()
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.message_list_preview(window, cx).into_any_element()
}
