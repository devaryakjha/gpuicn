//! Nova rows for team-chat transcripts: messages, reactions and dividers.
//!
//! The layout follows left-aligned team chat (Slack, Mattermost): a 32px avatar
//! gutter, a name and time header, and body content. Consecutive messages from
//! one sender collapse into continuation rows.

use std::rc::Rc;

use gpui_icons::{LucideIcon, lucide};
use gpui_kit::{
    AnyElement, AnyView, App, Axis, Bounds, ClickEvent, ElementId, FontWeight, ImageSource,
    InteractiveElement as _, IntoElement, ParentElement, Pixels, RenderOnce, Role, SharedString,
    StatefulInteractiveElement as _, Styled, Window, div, prelude::FluentBuilder as _, px,
    relative,
};

use gpui_kit::base::ElementExt as _;

use super::button::{Button, ButtonSize, ButtonVariant};
use super::theme::{
    UiText, UiTheme, apply_style,
    locale::{self, DirectionalStyled as _},
    primitives::{focus, hover_reveal_in_group, surface, text},
};
use super::tooltip::text_tooltip;

type Handler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
type AuthorHandler = Rc<dyn Fn(&ClickEvent, Bounds<Pixels>, &mut Window, &mut App) + 'static>;
type ReactionHandler = Rc<dyn Fn(&SharedString, &mut Window, &mut App) + 'static>;
type ReactionTooltip = Rc<dyn Fn(&Reaction, &mut Window, &mut App) -> AnyView + 'static>;

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
    /// The target of a permalink the reader followed: an amber tint and bar.
    Permalink,
}

/// How much room a row takes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MessageDensity {
    #[default]
    /// Avatar gutter, a header line with the name and time, then the body.
    Standard,
    /// One line per post, as Mattermost's compact display: the time always
    /// sits in the gutter, the name starts the body line, and rows are tighter.
    Compact,
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
#[derive(Clone)]
pub struct Reaction {
    /// The emoji or short code shown in the chip; also the value toggled.
    pub emoji: SharedString,
    /// How many people reacted.
    pub count: usize,
    /// Whether the reader is one of them.
    pub reacted: bool,
    /// The accessible name, such as "Arya and 2 others reacted with :tada:".
    pub label: Option<SharedString>,
    /// A custom emoji image shown instead of `emoji`, supplied by the application.
    pub image: Option<ImageSource>,
    /// Text shown in a tooltip on hover, such as who reacted.
    pub tooltip: Option<SharedString>,
}

impl Reaction {
    /// Creates a reaction chip.
    pub fn new(emoji: impl Into<SharedString>, count: usize, reacted: bool) -> Self {
        Self {
            emoji: emoji.into(),
            count,
            reacted,
            label: None,
            image: None,
            tooltip: None,
        }
    }
    /// Sets the accessible name, for example who reacted.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
    /// Shows a custom emoji image, such as a decoded image from the application's
    /// media cache. `emoji` stays the toggled value and the fallback name.
    pub fn image(mut self, image: impl Into<ImageSource>) -> Self {
        self.image = Some(image.into());
        self
    }
    /// Shows text in a tooltip on hover, for example "Arya, Sam and 3 others".
    /// `Message::reaction_tooltip` replaces it with a custom view.
    pub fn tooltip(mut self, text: impl Into<SharedString>) -> Self {
        self.tooltip = Some(text.into());
        self
    }
}

type BuildActions = Box<dyn FnOnce(&mut Window, &mut App) -> AnyElement>;

/// A row's hover action bar, built up front or only when shown.
enum Actions {
    Built(AnyElement),
    Lazy(BuildActions),
}

/// Whether the pointer is on a row or on its action bar, which floats partly
/// outside the row; the bar shows while either is hovered.
#[derive(Default)]
struct HoverState {
    row: bool,
    bar: bool,
}

/// A transcript row with avatar, header, body, reactions and a hover action bar.
#[derive(IntoElement)]
pub struct Message {
    style: gpui_kit::StyleRefinement,
    id: ElementId,
    avatar: Option<AnyElement>,
    author: Option<SharedString>,
    author_element: Option<AnyElement>,
    on_author_click: Option<AuthorHandler>,
    timestamp: Option<SharedString>,
    timestamp_tooltip: Option<SharedString>,
    gutter_time: Option<SharedString>,
    on_gutter_click: Option<Handler>,
    header: Vec<AnyElement>,
    children: Vec<AnyElement>,
    reactions: Vec<Reaction>,
    on_reaction: Option<ReactionHandler>,
    reaction_tooltip: Option<ReactionTooltip>,
    on_add_reaction: Option<Handler>,
    actions: Option<Actions>,
    description: Option<SharedString>,
    continuation: bool,
    density: MessageDensity,
    highlight: MessageHighlight,
    status: MessageStatus,
    on_retry: Option<Handler>,
    on_delete: Option<Handler>,
    copy_prefix: Option<SharedString>,
}

impl Message {
    /// Creates a message row with a stable caller-owned ID, such as the post ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            style: gpui_kit::StyleRefinement::default(),
            id: id.into(),
            avatar: None,
            author: None,
            author_element: None,
            on_author_click: None,
            timestamp: None,
            timestamp_tooltip: None,
            gutter_time: None,
            on_gutter_click: None,
            header: Vec::new(),
            children: Vec::new(),
            reactions: Vec::new(),
            on_reaction: None,
            reaction_tooltip: None,
            on_add_reaction: None,
            actions: None,
            description: None,
            continuation: false,
            density: MessageDensity::Standard,
            highlight: MessageHighlight::None,
            status: MessageStatus::Sent,
            on_retry: None,
            on_delete: None,
            copy_prefix: None,
        }
    }
    /// Chooses the standard or compact row layout.
    pub fn density(mut self, density: MessageDensity) -> Self {
        self.density = density;
        self
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
    /// Makes the name a link-styled button that the reader can click, or
    /// reach with Tab and press with Enter or Space, such as to open the
    /// sender's profile card. The handler gets the click and the name's
    /// bounds in window coordinates, to anchor a popover.
    pub fn on_author_click(
        mut self,
        handler: impl Fn(&ClickEvent, Bounds<Pixels>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_author_click = Some(Rc::new(handler));
        self
    }
    /// Draws this element in place of the name, such as the app's own popover
    /// trigger around the name, the way `avatar` takes any element. It
    /// inherits the name's weight, size and color, and replaces both `author`
    /// and `on_author_click`.
    pub fn author_element(mut self, element: impl IntoElement) -> Self {
        self.author_element = Some(element.into_any_element());
        self
    }
    /// Sets the time shown beside the name, such as "10:42 AM".
    pub fn timestamp(mut self, timestamp: impl Into<SharedString>) -> Self {
        self.timestamp = Some(timestamp.into());
        self
    }
    /// Shows the full date and time in a tooltip when the reader hovers the
    /// time, such as "Tuesday, 3 November 2026 at 16:40". It also serves the
    /// `gutter_time` of continuation and compact rows.
    pub fn timestamp_tooltip(mut self, text: impl Into<SharedString>) -> Self {
        self.timestamp_tooltip = Some(text.into());
        self
    }
    /// Runs when the reader clicks the gutter column (the avatar on the first
    /// row of a run, the empty gutter or hover time on continuation rows), for
    /// selecting the row with the pointer while clicks on the body stay free
    /// for text selection and links.
    pub fn on_gutter_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_gutter_click = Some(Rc::new(handler));
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
    /// Sets the text written before this message's selected text when a
    /// copy spans several messages, such as "Ada [12:24]: ", so the copy
    /// reads one line per message. A selection inside one message copies
    /// as before, with no prefix. Wrap the window in `selection_root`.
    pub fn copy_prefix(mut self, prefix: impl Into<SharedString>) -> Self {
        self.copy_prefix = Some(prefix.into());
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
    /// Builds the tooltip shown when a reaction chip is hovered, for example a
    /// list of who reacted with avatars. Without it, a chip with
    /// `Reaction::tooltip` text shows that text.
    pub fn reaction_tooltip(
        mut self,
        builder: impl Fn(&Reaction, &mut Window, &mut App) -> AnyView + 'static,
    ) -> Self {
        self.reaction_tooltip = Some(Rc::new(builder));
        self
    }
    /// Sets the description assistive technology reads for the row, such as
    /// "Arya, 10:42 AM, 2 reactions, edited".
    pub fn aria_description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
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
    ///
    /// Prefer [`Message::actions_with`] in long transcripts: this one asks the
    /// caller to build the bar for every row, though only the hovered or
    /// selected row shows it.
    pub fn actions(mut self, actions: impl IntoElement) -> Self {
        self.actions = Some(Actions::Built(actions.into_any_element()));
        self
    }
    /// Like [`Message::actions`], but builds the bar only while the row is
    /// hovered or selected, so rows the pointer is not on cost nothing for it.
    pub fn actions_with(
        mut self,
        actions: impl FnOnce(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.actions = Some(Actions::Lazy(Box::new(actions)));
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
        let dir = theme.direction;
        let group: SharedString = format!("message-{}", self.id).into();
        let selected = self.highlight == MessageHighlight::Selected;
        // Only the hovered or selected row lays out its action bar; a hidden
        // bar in every row cost each frame its layout, hitboxes and paint.
        let hover = self.actions.is_some().then(|| {
            window.use_keyed_state((self.id.clone(), "hover"), cx, |_, _| HoverState::default())
        });
        let show_actions = selected
            || hover
                .as_ref()
                .is_some_and(|hover| hover.read(cx).row || hover.read(cx).bar);
        let actions = match self.actions {
            Some(Actions::Built(actions)) if show_actions => Some(actions),
            Some(Actions::Lazy(build)) if show_actions => Some(build(window, cx)),
            _ => None,
        };
        let compact = self.density == MessageDensity::Compact;
        let gutter = if compact {
            theme.space(11.)
        } else {
            theme.space(8.)
        };

        let gutter_content = if compact {
            // Compact rows always show the time, so the gutter has a fixed
            // width and the body lines up from row to row.
            div().w(gutter).flex_shrink_0().pt(px(2.)).children(
                self.gutter_time
                    .clone()
                    .or(self.timestamp.clone())
                    .map(|time| {
                        time_tooltip(
                            text::xs(div(), &theme).id((self.id.clone(), "gutter-time")),
                            self.timestamp_tooltip.clone(),
                        )
                        .line_height(theme.text(20.))
                        .text_color(colors.muted_foreground)
                        .whitespace_nowrap()
                        .child(time)
                    }),
            )
        } else if self.continuation {
            // The hover time floats in the gutter so showing it never changes
            // the row's height.
            div().w(gutter).flex_shrink_0().relative().children(
                self.gutter_time
                    .clone()
                    .or(self.timestamp.clone())
                    .map(|time| {
                        hover_reveal_in_group(
                            time_tooltip(
                                text::xs(div(), &theme).id((self.id.clone(), "gutter-time")),
                                self.timestamp_tooltip.clone(),
                            )
                            .absolute()
                            .top(px(2.))
                            .inset_end(dir, px(0.))
                            .line_height(theme.text(20.))
                            .text_color(colors.muted_foreground)
                            .whitespace_nowrap()
                            .child(time),
                            group.clone(),
                            false,
                        )
                    }),
            )
        } else {
            div()
                .w(gutter)
                .flex_shrink_0()
                .pt(px(2.))
                .children(self.avatar)
        };
        let gutter_content = gutter_content.id((self.id.clone(), "gutter")).when_some(
            self.on_gutter_click.clone(),
            |gutter, handler| {
                gutter
                    .cursor_pointer()
                    .on_click(move |event, window, cx| handler(event, window, cx))
            },
        );

        // The name, as plain text, a link-styled button or the caller's element.
        let shows_author = !self.continuation;
        // Clipping would cut off the hover underline of a clickable name.
        let clip = self.author_element.is_none() && self.on_author_click.is_none();
        let mut author = shows_author
            .then(|| match (self.author_element, self.author.clone()) {
                (Some(element), _) => Some(element),
                (None, Some(name)) => Some(match self.on_author_click.clone() {
                    Some(handler) => author_button(&self.id, name, handler, &theme),
                    None => name.into_any_element(),
                }),
                (None, None) => None,
            })
            .flatten();

        let mut header_children = self.header;
        let header = (!self.continuation && !compact).then(|| {
            div()
                .flex_row_in(dir)
                .items_baseline()
                .gap(theme.space(2.))
                .min_w_0()
                .children(author.take().map(|author| {
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_size(theme.text(14.))
                        .text_color(colors.foreground)
                        .min_w_0()
                        .when(clip, |name| name.overflow_hidden())
                        .text_ellipsis()
                        .whitespace_nowrap()
                        .child(author)
                }))
                .children(self.timestamp.clone().map(|time| {
                    time_tooltip(
                        div().id((self.id.clone(), "timestamp")),
                        self.timestamp_tooltip.clone(),
                    )
                    .flex_shrink_0()
                    .text_size(theme.text(12.))
                    .text_color(colors.muted_foreground)
                    .child(time)
                }))
                .children(std::mem::take(&mut header_children))
        });

        let failed = (self.status == MessageStatus::Failed).then(|| {
            let link = |id: &'static str, label: SharedString, handler: Handler| {
                Button::new((self.id.clone(), id))
                    .variant(ButtonVariant::Link)
                    .size(ButtonSize::Xs)
                    .h(theme.space(5.))
                    .px_0()
                    .label(label)
                    .on_click(move |event, window, cx| handler(event, window, cx))
            };
            div()
                .flex_row_in(dir)
                .items_center()
                .gap(theme.space(2.))
                .text_size(theme.text(12.))
                .text_color(colors.destructive)
                .child(
                    lucide(LucideIcon::CircleAlert)
                        .size(theme.space(3.5))
                        .text_color(colors.destructive),
                )
                .child(locale::text(cx, UiText::MessageFailed))
                .when_some(self.on_retry.clone(), |row, handler| {
                    row.child(link("retry", locale::text(cx, UiText::Retry), handler))
                })
                .when_some(self.on_delete.clone(), |row, handler| {
                    row.child(link("delete", locale::text(cx, UiText::Delete), handler))
                })
        });

        let reactions = (!self.reactions.is_empty() || self.on_add_reaction.is_some()).then(|| {
            let on_reaction = self.on_reaction.clone();
            let tooltip = self.reaction_tooltip.clone();
            div()
                .flex_row_in(dir)
                .flex_wrap()
                .gap(theme.space(1.))
                .pt(theme.space(1.))
                .children(self.reactions.iter().map(|reaction| {
                    reaction_chip(
                        &self.id,
                        reaction,
                        on_reaction.clone(),
                        tooltip.clone(),
                        &theme,
                    )
                }))
                .when_some(self.on_add_reaction.clone(), |row, handler| {
                    row.child(
                        Button::new((self.id.clone(), "add-reaction"))
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconXs)
                            .rounded_full()
                            .aria_label(locale::text(cx, UiText::AddReaction))
                            .on_click(move |event, window, cx| handler(event, window, cx))
                            .child(
                                lucide(LucideIcon::FaceSlightlySmilingPlus)
                                    .size(theme.space(3.5))
                                    .text_color(colors.muted_foreground),
                            ),
                    )
                })
        });

        // Compact rows start the body line with the name; continuation rows
        // keep only the time in the gutter.
        let inline_author = (compact && !self.continuation)
            .then(|| author.take())
            .flatten()
            .map(|author| {
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors.foreground)
                    .whitespace_nowrap()
                    .child(author)
            });
        let inline_header =
            (compact && !self.continuation && !header_children.is_empty()).then(|| {
                div()
                    .flex_row_in(dir)
                    .items_center()
                    .gap(theme.space(1.))
                    .children(header_children)
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
                    // Wide content (an image, a table) shrinks with the
                    // column instead of widening the row past it.
                    .min_w_0()
                    .text_size(theme.text(14.))
                    .line_height(relative(1.45))
                    .text_color(colors.foreground)
                    .when(compact, |content| {
                        content
                            .flex_row_in(dir)
                            .flex_wrap()
                            .items_baseline()
                            .gap(theme.space(1.5))
                    })
                    .children(inline_author)
                    .children(inline_header)
                    .children(self.children),
            )
            .children(failed)
            .children(reactions);

        let permalink = colors.warning;
        let copy_prefix = self.copy_prefix.clone();
        let marker_id = self.id.clone();
        let root = div()
            .id(self.id.clone())
            .group(group.clone())
            .role(Role::ListItem)
            .relative()
            .w_full()
            .flex_row_in(dir)
            .text_start(dir)
            .gap(theme.space(2.))
            .px(theme.space(4.))
            .py(theme.space(if compact { 0.25 } else { 0.5 }))
            .when(!self.continuation && !compact, |row| {
                row.pt(theme.space(2.))
            })
            .border_start_2(dir)
            .border_color(colors.primary.opacity(0.))
            .when(!selected, |row| {
                row.hover(move |style| style.bg(colors.muted.opacity(0.5)))
            })
            .when(self.highlight == MessageHighlight::Mention, |row| {
                row.bg(colors.mention_highlight().opacity(0.06))
                    .border_color(colors.mention_highlight())
            })
            .when(selected, |row| row.bg(colors.muted))
            .when(self.highlight == MessageHighlight::Permalink, |row| {
                row.bg(permalink.opacity(0.14)).border_color(permalink)
            })
            .when_some(self.description, |row, description| {
                row.aria_description(description)
            })
            .when(self.status == MessageStatus::Pending, |row| {
                row.opacity(0.6)
            })
            .when_some(copy_prefix.clone(), |row, prefix| {
                row.child(copy_marker(&marker_id, &prefix, false, &theme))
            })
            .child(gutter_content)
            .child(body)
            .when_some(hover.clone(), |row, hover| {
                row.on_hover(move |hovered, _, cx| {
                    hover.update(cx, |hover, cx| {
                        if hover.row != *hovered {
                            hover.row = *hovered;
                            cx.notify();
                        }
                    })
                })
            })
            .when_some(actions, |row, actions| {
                let bar_hover = hover.clone();
                row.child(
                    div()
                        .id((self.id.clone(), "actions"))
                        .absolute()
                        .top(-theme.space(3.))
                        .inset_end(dir, theme.space(4.))
                        .when_some(bar_hover, |bar, hover| {
                            bar.on_hover(move |hovered, _, cx| {
                                hover.update(cx, |hover, cx| {
                                    if hover.bar != *hovered {
                                        hover.bar = *hovered;
                                        cx.notify();
                                    }
                                })
                            })
                        })
                        .child(actions),
                )
            })
            .when_some(copy_prefix, |row, prefix| {
                row.child(copy_marker(&marker_id, &prefix, true, &theme))
            });
        apply_style(root, &self.style)
    }
}

const MARK_OPEN: char = '\u{E000}';
const MARK_CLOSE: char = '\u{E001}';

/// An invisible selectable marker at the top (`end` false) or bottom of a
/// copy group. A selection that reaches past it takes its text, which
/// [`apply_copy_prefixes`] turns back into the group's prefix. It is a
/// Markdown view so it joins the selection in paint order, like the bodies.
fn copy_marker(id: &ElementId, prefix: &str, end: bool, theme: &UiTheme) -> AnyElement {
    let hex: String = prefix.bytes().map(|byte| format!("{byte:02x}")).collect();
    let kind = if end { 'E' } else { 'S' };
    let text = format!("{MARK_OPEN}{kind}{hex}{MARK_CLOSE}");
    div()
        .absolute()
        .left_0()
        .map(|marker| {
            if end {
                marker.bottom_0()
            } else {
                marker.top_0()
            }
        })
        .whitespace_nowrap()
        .text_size(theme.text(1.))
        .line_height(theme.text(1.))
        .text_color(theme.colors.background.opacity(0.))
        .child(
            gpui_kit::component::text::TextView::markdown(
                (id.clone(), if end { "copy-end" } else { "copy-start" }),
                text,
            )
            .selectable(true),
        )
        .into_any_element()
}

/// Wraps `content` as one copy group with `prefix`, like
/// [`Message::copy_prefix`] for a region that is not a `Message`.
pub fn selection_group(
    id: impl Into<ElementId>,
    prefix: impl Into<SharedString>,
    content: impl IntoElement,
    cx: &App,
) -> gpui_kit::Div {
    let theme = UiTheme::read(cx);
    let id = id.into();
    let prefix = prefix.into();
    div()
        .relative()
        .child(copy_marker(&id, &prefix, false, theme))
        .child(content)
        .child(copy_marker(&id, &prefix, true, theme))
}

/// Whether a selection's text crosses a copy group's edge.
#[doc(hidden)]
pub fn has_copy_groups(text: &str) -> bool {
    text.contains(MARK_OPEN)
}

/// Turns the copy-group markers in a selection's text into prefixes: when
/// the selection covers more than one group, each group's text becomes one
/// line that starts with its prefix; otherwise the markers are dropped.
/// `copy_selection` calls this, so apps only need it for their own copy.
pub fn apply_copy_prefixes(text: &str) -> String {
    if !text.contains(MARK_OPEN) {
        return text.to_string();
    }
    let decode = |hex: &str| -> String {
        let bytes: Vec<u8> = (0..hex.len() / 2)
            .filter_map(|i| u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok())
            .collect();
        String::from_utf8_lossy(&bytes).into_owned()
    };
    let mut segments: Vec<(Option<String>, String)> = Vec::new();
    let mut current: Option<String> = None;
    let mut buffer = String::new();
    let mut rest = text;
    while let Some(open) = rest.find(MARK_OPEN) {
        buffer.push_str(&rest[..open]);
        let after = &rest[open + MARK_OPEN.len_utf8()..];
        let Some(close) = after.find(MARK_CLOSE) else {
            rest = after;
            break;
        };
        let mark = &after[..close];
        rest = &after[close + MARK_CLOSE.len_utf8()..];
        let (kind, hex) = mark.split_at(mark.len().min(1));
        match kind {
            // A group starts: what came before belongs to the group already open.
            "S" => {
                segments.push((current.take(), std::mem::take(&mut buffer)));
                current = Some(decode(hex));
            }
            // A group ends: what came before is its text, even when the
            // selection began inside it, after its start marker.
            "E" => {
                segments.push((Some(decode(hex)), std::mem::take(&mut buffer)));
                current = None;
            }
            _ => {}
        }
    }
    buffer.push_str(rest);
    segments.push((current, buffer));
    let segments: Vec<_> = segments
        .into_iter()
        .map(|(prefix, text)| (prefix, text.trim().to_string()))
        .filter(|(_, text)| !text.is_empty())
        .collect();
    if segments.len() < 2 {
        return segments.into_iter().map(|(_, text)| text).collect();
    }
    segments
        .into_iter()
        .map(|(prefix, text)| format!("{}{text}", prefix.unwrap_or_default()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Adds the full-date tooltip to a time label when one is set.
fn time_tooltip(
    label: gpui_kit::Stateful<gpui_kit::Div>,
    tooltip: Option<SharedString>,
) -> gpui_kit::Stateful<gpui_kit::Div> {
    label.when_some(tooltip, |label, text| {
        label.tooltip(move |_, cx| text_tooltip(text.clone(), cx))
    })
}

fn reaction_chip(
    message: &ElementId,
    reaction: &Reaction,
    on_reaction: Option<ReactionHandler>,
    tooltip: Option<ReactionTooltip>,
    theme: &UiTheme,
) -> impl IntoElement {
    let colors = theme.colors;
    let emoji = reaction.emoji.clone();
    let label = reaction
        .label
        .clone()
        .unwrap_or_else(|| format!("{} {}", reaction.emoji, reaction.count).into());
    let key = SharedString::from(format!("reaction-{}", reaction.emoji));
    let glyph = match reaction.image.clone() {
        Some(image) => super::theme::primitives::image(image)
            .size(theme.space(4.))
            .flex_shrink_0()
            .into_any_element(),
        None => div()
            .text_size(theme.text(14.))
            .child(reaction.emoji.clone())
            .into_any_element(),
    };
    let chip = Button::new((message.clone(), key.clone()))
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
        .child(glyph)
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
        );
    let tooltip: Option<ReactionTooltip> = match (tooltip, reaction.tooltip.clone()) {
        (Some(builder), _) => Some(builder),
        (None, Some(text)) => Some(Rc::new(
            move |_: &Reaction, _: &mut Window, cx: &mut App| text_tooltip(text.clone(), cx),
        )),
        (None, None) => None,
    };
    let reaction = reaction.clone();
    div()
        .id((
            message.clone(),
            SharedString::from(format!("{key}-tooltip")),
        ))
        .when_some(tooltip, move |wrapper, builder| {
            wrapper.tooltip(move |window, cx| builder(&reaction, window, cx))
        })
        .child(chip)
}

/// The sender's name as a link-styled button that reports its own bounds.
fn author_button(
    id: &ElementId,
    name: SharedString,
    handler: AuthorHandler,
    theme: &UiTheme,
) -> AnyElement {
    let bounds = Rc::new(std::cell::Cell::new(Bounds::default()));
    let measured = bounds.clone();
    let theme = theme.clone();
    Button::new((id.clone(), "author"))
        .aria_label(name.clone())
        .style_with(move |base| {
            let focus_theme = theme.clone();
            base.role(Role::Link)
                .rounded(theme.radius.sm)
                .justify_start()
                .cursor_pointer()
                .hover(|style| style.underline())
                .focus_visible(move |style| focus::ring(style, &focus_theme))
        })
        .on_click(move |event, window, cx| handler(event, bounds.get(), window, cx))
        .child(name)
        .on_prepaint(move |bounds, _, _| measured.set(bounds))
        .into_any_element()
}

/// The floating bar that holds a message's hover actions.
pub fn message_actions(cx: &App) -> gpui_kit::Div {
    let theme = UiTheme::read(cx);
    div()
        .flex_row_in(theme.direction)
        .items_center()
        .gap(theme.space(0.5))
        .p(theme.space(0.5))
        .rounded(theme.radius.lg)
        .border_1()
        .border_color(theme.colors.border)
        .bg(theme.colors.background)
        .shadow(theme.shadows.sm.to_vec())
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
    let rule = || surface::divider(div().flex_1(), theme, Axis::Horizontal);
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
        .flex_row_in(theme.direction)
        .items_center()
        .gap(theme.space(2.))
        .px(theme.space(4.))
        .py(theme.space(1.))
        .child(
            surface::divider(div().flex_1(), theme, Axis::Horizontal)
                .bg(theme.colors.new_messages()),
        )
        .child(
            div()
                .text_size(theme.text(12.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.colors.new_messages())
                .child(label.into()),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_markers_become_one_line_per_group() {
        let mark = |kind: char, prefix: &str| {
            let hex: String = prefix.bytes().map(|b| format!("{b:02x}")).collect();
            format!("{MARK_OPEN}{kind}{hex}{MARK_CLOSE}")
        };
        // Starts inside Ada's message and ends inside Niklaus's.
        let text = format!(
            "worked!\n{}\n{}\nBack online.",
            mark('E', "Ada [12:24]: "),
            mark('S', "Niklaus [12:24]: ")
        );
        assert_eq!(
            apply_copy_prefixes(&text),
            "Ada [12:24]: worked!\nNiklaus [12:24]: Back online."
        );
        // Only one group has text: no prefix.
        let text = format!("{}\nhello", mark('S', "Ada: "));
        assert_eq!(apply_copy_prefixes(&text), "hello");
        assert_eq!(apply_copy_prefixes("plain"), "plain");
    }
    use gpui_kit::{Context, Pixels, Render, TestAppContext, VisualTestContext};

    struct Probe(MessageDensity);

    impl Render for Probe {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .flex()
                .flex_col()
                .w(px(600.))
                .child(
                    div().debug_selector(|| "post".into()).child(
                        Message::new("post")
                            .density(self.0)
                            .author("Arya")
                            .timestamp("10:42")
                            .header_child(div().child("Bot"))
                            .child("First line"),
                    ),
                )
                .child(
                    div().debug_selector(|| "follow".into()).child(
                        Message::new("follow")
                            .density(self.0)
                            .continuation(true)
                            .gutter_time("10:43")
                            .child("Second"),
                    ),
                )
        }
    }

    fn row_heights(cx: &mut TestAppContext, density: MessageDensity) -> (Pixels, Pixels) {
        let window = cx.add_window(|_, _| Probe(density));
        let mut cx = VisualTestContext::from_window(window.into(), cx);
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let height = |cx: &mut VisualTestContext, id: &'static str| {
            cx.debug_bounds(id)
                .unwrap_or_else(|| panic!("{id} drew"))
                .size
                .height
        };
        (height(&mut cx, "post"), height(&mut cx, "follow"))
    }

    #[gpui_kit::test]
    fn compact_rows_are_one_line_and_shorter(cx: &mut TestAppContext) {
        cx.update(super::super::theme::init);
        let (standard_first, standard_follow) = row_heights(cx, MessageDensity::Standard);
        let (compact_first, compact_follow) = row_heights(cx, MessageDensity::Compact);
        assert!(
            compact_first < standard_first,
            "compact first row {compact_first:?} should be shorter than {standard_first:?}"
        );
        assert!(compact_follow < standard_follow);
        // One line: the name sits inline, so the first row is as tall as a continuation row.
        assert_eq!(compact_first, compact_follow);
    }

    struct GutterProbe(Rc<std::cell::Cell<usize>>);

    impl Render for GutterProbe {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let clicks = self.0.clone();
            div().w(px(600.)).child(
                div().debug_selector(|| "row".into()).child(
                    Message::new("row")
                        .continuation(true)
                        .gutter_time("10:43")
                        .timestamp_tooltip("Tuesday, 3 November 2026 at 10:43")
                        .on_gutter_click(move |_, _, _| clicks.set(clicks.get() + 1))
                        .child("Second"),
                ),
            )
        }
    }

    /// The gutter of a continuation row is a click target, and the body is not
    /// (karyalabs/gpuicn#98).
    #[gpui_kit::test]
    fn the_gutter_selects_the_row_and_the_body_does_not(cx: &mut TestAppContext) {
        cx.update(super::super::theme::init);
        let clicks = Rc::new(std::cell::Cell::new(0));
        let probe = clicks.clone();
        let window = cx.add_window(move |_, _| GutterProbe(probe));
        let mut cx = VisualTestContext::from_window(window.into(), cx);
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let row = cx.debug_bounds("row").expect("row drew");
        let y = row.center().y;
        // The gutter starts after the row's 16px padding and 2px bar.
        cx.simulate_click(
            gpui_kit::point(row.left() + px(30.), y),
            gpui_kit::Modifiers::default(),
        );
        assert_eq!(clicks.get(), 1);
        cx.simulate_click(
            gpui_kit::point(row.left() + px(200.), y),
            gpui_kit::Modifiers::default(),
        );
        assert_eq!(clicks.get(), 1);
    }

    struct ActionsProbe(Rc<std::cell::Cell<usize>>);

    impl Render for ActionsProbe {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let builds = self.0.clone();
            div()
                .w(px(600.))
                .pt(px(40.))
                .flex()
                .flex_col()
                .children(["a", "b", "c"].map(|id| {
                    let builds = builds.clone();
                    let row =
                        Message::new(id)
                            .author("Arya")
                            .child("Hello")
                            .actions_with(move |_, _| {
                                builds.set(builds.get() + 1);
                                div()
                                    .size(px(20.))
                                    .debug_selector(move || format!("bar-{id}"))
                                    .into_any_element()
                            });
                    div().debug_selector(move || format!("row-{id}")).child(row)
                }))
        }
    }

    /// Only the hovered row builds and lays out its action bar, and the bar
    /// stays while the pointer moves onto it (#670 perf: rows that are not
    /// hovered cost nothing for their bar).
    #[gpui_kit::test]
    fn only_the_hovered_row_builds_its_action_bar(cx: &mut TestAppContext) {
        cx.update(super::super::theme::init);
        let builds = Rc::new(std::cell::Cell::new(0));
        let window = cx.add_window({
            let builds = builds.clone();
            move |_, _| ActionsProbe(builds)
        });
        let mut cx = VisualTestContext::from_window(window.into(), cx);
        cx.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(builds.get(), 0, "no row is hovered");
        assert!(cx.debug_bounds("bar-b").is_none());
        let row = cx.debug_bounds("row-b").expect("row drew");
        cx.simulate_mouse_move(row.center(), None, gpui_kit::Modifiers::default());
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let bar = cx
            .debug_bounds("bar-b")
            .expect("the hovered row shows its bar");
        assert!(cx.debug_bounds("bar-a").is_none() && cx.debug_bounds("bar-c").is_none());
        // Onto the bar, which floats over the row's top edge.
        cx.simulate_mouse_move(bar.center(), None, gpui_kit::Modifiers::default());
        cx.update(|window, cx| window.draw(cx).clear(cx));
        assert!(
            cx.debug_bounds("bar-b").is_some(),
            "the bar stays under the pointer"
        );
    }

    type Clicked = Rc<std::cell::RefCell<Vec<Bounds<Pixels>>>>;

    struct AuthorProbe(Clicked, MessageDensity);

    impl Render for AuthorProbe {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let clicks = self.0.clone();
            div()
                .w(px(600.))
                .flex()
                .flex_col()
                .child(
                    Message::new("post")
                        .density(self.1)
                        .author("Arya")
                        .on_author_click(move |_, bounds, _, _| clicks.borrow_mut().push(bounds))
                        .timestamp("10:42")
                        .child("Hello"),
                )
                .child(
                    Message::new("custom")
                        .density(self.1)
                        .author("ignored")
                        .author_element(div().debug_selector(|| "custom-name".into()).child("Dev"))
                        .child("Hi"),
                )
        }
    }

    /// The name opens something with the pointer or the keyboard and reports
    /// its bounds, and `author_element` replaces it (karyalabs/gpuicn#123).
    #[gpui_kit::test]
    fn the_author_name_is_a_button_with_its_bounds(cx: &mut TestAppContext) {
        use gpui_kit::test::TestQueryExt as _;
        use gpui_kit::{KeyDownEvent, KeyUpEvent, Keystroke};
        cx.update(super::super::theme::init);
        for density in [MessageDensity::Standard, MessageDensity::Compact] {
            let clicks = Clicked::default();
            let probe = clicks.clone();
            let window = cx.add_window(move |window, _| {
                window.activate_window();
                AuthorProbe(probe, density)
            });
            let mut cx = VisualTestContext::from_window(window.into(), cx);
            cx.update(|window, cx| window.draw(cx).clear(cx));
            assert!(
                cx.debug_bounds("custom-name").is_some(),
                "the custom element drew"
            );

            let name = cx.update(|window, _| {
                let name = window.find_by_label("Arya");
                assert_eq!(name.role(), Some(Role::Link));
                name.bounds()
            });
            cx.simulate_click(name.center(), gpui_kit::Modifiers::default());
            assert_eq!(
                *clicks.borrow(),
                [name],
                "{density:?}: the click reports the name's bounds"
            );

            cx.simulate_keystrokes("tab");
            cx.update(|window, cx| window.draw(cx).clear(cx));
            let keystroke = Keystroke::parse("enter").unwrap();
            cx.simulate_event(KeyDownEvent {
                keystroke: keystroke.clone(),
                is_held: false,
                prefer_character_input: false,
            });
            cx.simulate_event(KeyUpEvent { keystroke });
            assert_eq!(
                clicks.borrow().len(),
                2,
                "{density:?}: Enter on the focused name"
            );
        }
    }
}
