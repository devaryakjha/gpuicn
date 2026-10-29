//! Nova rows for team-chat transcripts: messages, reactions and dividers.
//!
//! The layout follows left-aligned team chat (Slack, Mattermost): a 32px avatar
//! gutter, a name and time header, and body content. Consecutive messages from
//! one sender collapse into continuation rows.

use std::rc::Rc;

use gpui_icons::{LucideIcon, lucide};
use gpui_kit::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement, RenderOnce, Role, SharedString, StatefulInteractiveElement as _, Styled, Window,
    div, prelude::FluentBuilder as _, px, relative,
};

use super::button::{Button, ButtonSize, ButtonVariant};
use super::theme::{UiTheme, apply_style};

type Handler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
type ReactionHandler = Rc<dyn Fn(&SharedString, &mut Window, &mut App) + 'static>;

/// How a message row is emphasized.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MessageHighlight {
    #[default]
    /// No emphasis.
    None,
    /// Mentions the reader: a tinted row and a primary bar in the gutter.
    Mention,
    /// Selected for keyboard actions; also keeps the action bar visible.
    Selected,
}

/// Delivery state of the reader's own message.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MessageStatus {
    #[default]
    /// Delivered.
    Sent,
    /// Still sending; shown at reduced opacity.
    Pending,
    /// Failed to send; shows Retry and Delete when handlers are set.
    Failed,
}

/// One reaction chip: an emoji (or short text), its count and whether the reader reacted.
#[derive(Clone, Debug, PartialEq)]
pub struct Reaction {
    /// The emoji or short code shown in the chip; also the value toggled.
    pub emoji: SharedString,
    /// How many people reacted.
    pub count: usize,
    /// Whether the reader is one of them.
    pub reacted: bool,
    /// The accessible name, such as "Arya and 2 others reacted with :tada:".
    pub label: Option<SharedString>,
}

impl Reaction {
    /// Creates a reaction chip.
    pub fn new(emoji: impl Into<SharedString>, count: usize, reacted: bool) -> Self {
        Self {
            emoji: emoji.into(),
            count,
            reacted,
            label: None,
        }
    }
    /// Sets the accessible name, for example who reacted.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// A transcript row with avatar, header, body, reactions and a hover action bar.
#[derive(IntoElement)]
pub struct Message {
    style: gpui_kit::StyleRefinement,
    id: ElementId,
    avatar: Option<AnyElement>,
    author: Option<SharedString>,
    timestamp: Option<SharedString>,
    gutter_time: Option<SharedString>,
    header: Vec<AnyElement>,
    children: Vec<AnyElement>,
    reactions: Vec<Reaction>,
    on_reaction: Option<ReactionHandler>,
    on_add_reaction: Option<Handler>,
    actions: Option<AnyElement>,
    continuation: bool,
    highlight: MessageHighlight,
    status: MessageStatus,
    on_retry: Option<Handler>,
    on_delete: Option<Handler>,
}

impl Message {
    /// Creates a message row with a stable caller-owned ID, such as the post ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            style: gpui_kit::StyleRefinement::default(),
            id: id.into(),
            avatar: None,
            author: None,
            timestamp: None,
            gutter_time: None,
            header: Vec::new(),
            children: Vec::new(),
            reactions: Vec::new(),
            on_reaction: None,
            on_add_reaction: None,
            actions: None,
            continuation: false,
            highlight: MessageHighlight::None,
            status: MessageStatus::Sent,
            on_retry: None,
            on_delete: None,
        }
    }
    /// Sets the sender's avatar, usually an `Avatar`.
    pub fn avatar(mut self, avatar: impl IntoElement) -> Self {
        self.avatar = Some(avatar.into_any_element());
        self
    }
    /// Sets the sender's display name.
    pub fn author(mut self, author: impl Into<SharedString>) -> Self {
        self.author = Some(author.into());
        self
    }
    /// Sets the time shown beside the name, such as "10:42 AM".
    pub fn timestamp(mut self, timestamp: impl Into<SharedString>) -> Self {
        self.timestamp = Some(timestamp.into());
        self
    }
    /// Adds an element after the time in the header, such as a "Bot" tag.
    pub fn header_child(mut self, child: impl IntoElement) -> Self {
        self.header.push(child.into_any_element());
        self
    }
    /// Collapses the row under the previous message from the same sender. The
    /// avatar and header are hidden and `gutter_time` shows on hover instead.
    pub fn continuation(mut self, continuation: bool) -> Self {
        self.continuation = continuation;
        self
    }
    /// Sets the short time shown in the gutter of a hovered continuation row.
    pub fn gutter_time(mut self, time: impl Into<SharedString>) -> Self {
        self.gutter_time = Some(time.into());
        self
    }
    /// Emphasizes the row as a mention of the reader or as keyboard-selected.
    pub fn highlight(mut self, highlight: MessageHighlight) -> Self {
        self.highlight = highlight;
        self
    }
    /// Sets the delivery state of the reader's own message.
    pub fn status(mut self, status: MessageStatus) -> Self {
        self.status = status;
        self
    }
    /// Shows a Retry action on a failed message.
    pub fn on_retry(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_retry = Some(Rc::new(handler));
        self
    }
    /// Shows a Delete action on a failed message.
    pub fn on_delete(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_delete = Some(Rc::new(handler));
        self
    }
    /// Sets the reaction chips shown under the body.
    pub fn reactions(mut self, reactions: impl IntoIterator<Item = Reaction>) -> Self {
        self.reactions = reactions.into_iter().collect();
        self
    }
    /// Runs with a chip's emoji when the reader toggles that reaction.
    pub fn on_reaction(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_reaction = Some(Rc::new(handler));
        self
    }
    /// Adds an "Add reaction" button after the chips, for opening an emoji picker.
    pub fn on_add_reaction(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_add_reaction = Some(Rc::new(handler));
        self
    }
    /// Sets the floating action bar shown while the row is hovered or selected,
    /// usually `message_actions` holding `message_action` buttons.
    pub fn actions(mut self, actions: impl IntoElement) -> Self {
        self.actions = Some(actions.into_any_element());
        self
    }
}

impl ParentElement for Message {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Styled for Message {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Message {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let hovered = window.use_keyed_state((self.id.clone(), "hovered"), cx, |_, _| false);
        let is_hovered = *hovered.read(cx);
        let selected = self.highlight == MessageHighlight::Selected;
        let show_actions = is_hovered || selected;
        let gutter = theme.space(8.);

        let gutter_content = if self.continuation {
            div()
                .w(gutter)
                .flex_shrink_0()
                .pt(px(2.))
                .text_size(theme.text(11.))
                .line_height(theme.text(20.))
                .text_color(colors.muted_foreground)
                .text_right()
                .whitespace_nowrap()
                .when(is_hovered, |gutter| {
                    gutter.children(self.gutter_time.clone().or(self.timestamp.clone()))
                })
        } else {
            div()
                .w(gutter)
                .flex_shrink_0()
                .pt(px(2.))
                .children(self.avatar)
        };

        let header = (!self.continuation).then(|| {
            div()
                .flex()
                .items_baseline()
                .gap(theme.space(2.))
                .min_w_0()
                .children(self.author.clone().map(|author| {
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_size(theme.text(14.))
                        .text_color(colors.foreground)
                        .overflow_hidden()
                        .text_ellipsis()
                        .whitespace_nowrap()
                        .child(author)
                }))
                .children(self.timestamp.clone().map(|time| {
                    div()
                        .flex_shrink_0()
                        .text_size(theme.text(12.))
                        .text_color(colors.muted_foreground)
                        .child(time)
                }))
                .children(self.header)
        });

        let failed = (self.status == MessageStatus::Failed).then(|| {
            let link = |id: &'static str, label: &'static str, handler: Handler| {
                Button::new((self.id.clone(), id))
                    .variant(ButtonVariant::Link)
                    .size(ButtonSize::Xs)
                    .h(theme.space(5.))
                    .px_0()
                    .label(label)
                    .on_click(move |event, window, cx| handler(event, window, cx))
            };
            div()
                .flex()
                .items_center()
                .gap(theme.space(2.))
                .text_size(theme.text(12.))
                .text_color(colors.destructive)
                .child(
                    lucide(LucideIcon::CircleAlert)
                        .size(theme.space(3.5))
                        .text_color(colors.destructive),
                )
                .child("Message failed to send.")
                .when_some(self.on_retry.clone(), |row, handler| {
                    row.child(link("retry", "Retry", handler))
                })
                .when_some(self.on_delete.clone(), |row, handler| {
                    row.child(link("delete", "Delete", handler))
                })
        });

        let reactions =
            (!self.reactions.is_empty() || self.on_add_reaction.is_some()).then(|| {
                let on_reaction = self.on_reaction.clone();
                div()
                    .flex()
                    .flex_wrap()
                    .gap(theme.space(1.))
                    .pt(theme.space(1.))
                    .children(self.reactions.iter().map(|reaction| {
                        reaction_chip(&self.id, reaction, on_reaction.clone(), &theme)
                    }))
                    .when_some(self.on_add_reaction.clone(), |row, handler| {
                        row.child(
                            Button::new((self.id.clone(), "add-reaction"))
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::IconXs)
                                .rounded_full()
                                .aria_label("Add reaction")
                                .on_click(move |event, window, cx| handler(event, window, cx))
                                .child(
                                    lucide(LucideIcon::FaceSlightlySmilingPlus)
                                        .size(theme.space(3.5))
                                        .text_color(colors.muted_foreground),
                                ),
                        )
                    })
            });

        let body = div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .gap(theme.space(0.5))
            .children(header)
            .child(
                div()
                    .text_size(theme.text(14.))
                    .line_height(relative(1.45))
                    .text_color(colors.foreground)
                    .children(self.children),
            )
            .children(failed)
            .children(reactions);

        let hover_state = hovered.clone();
        let root = div()
            .id(self.id.clone())
            .role(Role::ListItem)
            .relative()
            .w_full()
            .flex()
            .gap(theme.space(2.))
            .px(theme.space(4.))
            .py(theme.space(0.5))
            .when(!self.continuation, |row| row.pt(theme.space(2.)))
            .border_l_2()
            .border_color(colors.primary.opacity(0.))
            .when(is_hovered, |row| row.bg(colors.muted.opacity(0.5)))
            .when(self.highlight == MessageHighlight::Mention, |row| {
                row.bg(colors.primary.opacity(0.06))
                    .border_color(colors.primary)
            })
            .when(selected, |row| row.bg(colors.muted))
            .when(self.status == MessageStatus::Pending, |row| {
                row.opacity(0.6)
            })
            .on_hover(move |hovered, _, cx| {
                hover_state.update(cx, |state, cx| {
                    if *state != *hovered {
                        *state = *hovered;
                        cx.notify();
                    }
                })
            })
            .child(gutter_content)
            .child(body)
            .when_some(self.actions.filter(|_| show_actions), |row, actions| {
                row.child(
                    div()
                        .absolute()
                        .top(-theme.space(3.))
                        .right(theme.space(4.))
                        .child(actions),
                )
            });
        apply_style(root, &self.style)
    }
}

fn reaction_chip(
    message: &ElementId,
    reaction: &Reaction,
    on_reaction: Option<ReactionHandler>,
    theme: &UiTheme,
) -> impl IntoElement {
    let colors = theme.colors;
    let emoji = reaction.emoji.clone();
    let label = reaction
        .label
        .clone()
        .unwrap_or_else(|| format!("{} {}", reaction.emoji, reaction.count).into());
    Button::new((
        message.clone(),
        SharedString::from(format!("reaction-{}", reaction.emoji)),
    ))
    .variant(ButtonVariant::Outline)
    .size(ButtonSize::Xs)
    .h(theme.space(6.))
    .px(theme.space(2.))
    .gap(theme.space(1.5))
    .rounded_full()
    .when(reaction.reacted, |chip| {
        chip.bg(colors.primary.opacity(0.1))
            .border_color(colors.primary.opacity(0.5))
    })
    .aria_label(label)
    .on_click(move |_, window, cx| {
        if let Some(handler) = &on_reaction {
            handler(&emoji, window, cx);
        }
    })
    .child(
        div()
            .text_size(theme.text(14.))
            .child(reaction.emoji.clone()),
    )
    .child(
        div()
            .text_size(theme.text(12.))
            .font_weight(FontWeight::MEDIUM)
            .text_color(if reaction.reacted {
                colors.primary
            } else {
                colors.muted_foreground
            })
            .child(reaction.count.to_string()),
    )
}

/// The floating bar that holds a message's hover actions.
pub fn message_actions(cx: &App) -> gpui_kit::Div {
    let theme = UiTheme::read(cx);
    div()
        .flex()
        .items_center()
        .gap(theme.space(0.5))
        .p(theme.space(0.5))
        .rounded(theme.radius.lg)
        .border_1()
        .border_color(theme.colors.border)
        .bg(theme.colors.background)
        .shadow(theme.shadows.sm.clone())
}

/// An icon button for `message_actions`, named for assistive technology.
pub fn message_action(
    id: impl Into<ElementId>,
    icon: LucideIcon,
    label: impl Into<SharedString>,
    cx: &App,
) -> Button {
    let theme = UiTheme::read(cx);
    Button::new(id)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::IconSm)
        .aria_label(label)
        .child(
            lucide(icon)
                .size(theme.space(4.))
                .text_color(theme.colors.muted_foreground),
        )
}

/// A centered date label between two rules, such as "Today" or "Monday, October 27".
pub fn day_divider(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    cx: &App,
) -> impl IntoElement {
    let theme = UiTheme::read(cx);
    let rule = || div().flex_1().h(px(1.)).bg(theme.colors.border);
    div()
        .id(id)
        .role(Role::Splitter)
        .w_full()
        .flex()
        .items_center()
        .gap(theme.space(3.))
        .px(theme.space(4.))
        .py(theme.space(2.))
        .child(rule())
        .child(
            div()
                .px(theme.space(3.))
                .h(theme.space(6.))
                .flex()
                .items_center()
                .rounded_full()
                .border_1()
                .border_color(theme.colors.border)
                .bg(theme.colors.background)
                .text_size(theme.text(12.))
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.colors.foreground)
                .child(label.into()),
        )
        .child(rule())
}

/// The destructive rule that marks where unread messages start.
pub fn new_messages_divider(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    cx: &App,
) -> impl IntoElement {
    let theme = UiTheme::read(cx);
    div()
        .id(id)
        .role(Role::Splitter)
        .w_full()
        .flex()
        .items_center()
        .gap(theme.space(2.))
        .px(theme.space(4.))
        .py(theme.space(1.))
        .child(div().flex_1().h(px(1.)).bg(theme.colors.destructive))
        .child(
            div()
                .text_size(theme.text(12.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.colors.destructive)
                .child(label.into()),
        )
}
