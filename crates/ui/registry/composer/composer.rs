//! Nova chat composer: a growing textarea with caret control, paste and key
//! hooks, Enter to send, and an autocomplete popup for `@`, `~`, `:` and `/`.
//!
//! The application owns what the triggers mean. It listens for
//! `ComposerEvent::Autocomplete`, looks up suggestions (synchronously or not)
//! and passes them to `set_suggestions`; accepting one replaces the trigger
//! and query with the suggestion's text.

use std::ops::Range;
use std::rc::Rc;

use gpui_icons::{LucideIcon, lucide};
use gpui_kit::{
    Anchor, AnchoredPositionMode, AnyElement, App, AppContext as _, Bounds, ClipboardItem, Context,
    ElementId, Entity, EntityInputHandler as _, EventEmitter, FocusHandle, Focusable, FontWeight,
    InteractiveElement as _, IntoElement, Keystroke, ParentElement, Pixels, Point, Render,
    RenderOnce, Role, SharedString, StatefulInteractiveElement as _, StyleRefinement, Styled,
    Subscription, Window, anchored, deferred, div, point, prelude::FluentBuilder as _, px,
};

use super::input::{InputEvent, Textarea, TextareaState};
use super::theme::{UiTheme, apply_style};

type KeyHandler = Rc<dyn Fn(&Keystroke, &mut Window, &mut App) -> bool + 'static>;
type PasteHandler = Rc<dyn Fn(&ClipboardItem, &mut Window, &mut App) -> bool + 'static>;
type SuggestionSlot =
    Rc<dyn Fn(&ComposerSuggestion, &mut Window, &mut App) -> Option<AnyElement> + 'static>;

/// One autocomplete entry.
#[derive(Clone, Debug, PartialEq)]
pub struct ComposerSuggestion {
    /// The text that replaces the trigger and query, such as `"@arya"`.
    pub value: SharedString,
    /// The visible name, such as "Arya Jha".
    pub label: SharedString,
    /// Secondary text, such as "@arya" or a channel purpose.
    pub description: Option<SharedString>,
    /// A leading Lucide icon, such as `Hash` for channels.
    pub icon: Option<LucideIcon>,
}

impl ComposerSuggestion {
    /// Creates a suggestion that inserts `value` and shows `label`.
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            description: None,
            icon: None,
        }
    }
    /// Adds secondary text after the label.
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }
    /// Adds a leading Lucide icon.
    pub fn icon(mut self, icon: LucideIcon) -> Self {
        self.icon = Some(icon);
        self
    }
}

/// Notifications from a composer.
#[derive(Clone, Debug, PartialEq)]
pub enum ComposerEvent {
    /// The text changed.
    Change,
    /// Enter was pressed outside autocomplete and IME composition. The
    /// application sends the text and clears the composer with `set_value`.
    Submit(SharedString),
    /// The caret is in a trigger such as `@ar`: look up suggestions for
    /// `query` and pass them to `set_suggestions`.
    Autocomplete {
        /// The trigger character, such as `'@'`.
        trigger: char,
        /// The text typed after the trigger.
        query: SharedString,
    },
    /// The caret left the trigger, or the reader dismissed the popup.
    AutocompleteClosed,
    /// A suggestion replaced the trigger and query.
    Accept {
        /// The trigger character.
        trigger: char,
        /// The accepted suggestion's value.
        value: SharedString,
    },
}

#[derive(Clone, Debug, PartialEq)]
struct ActiveTrigger {
    trigger: char,
    start: usize,
    query: SharedString,
}

/// Finds the trigger the caret is in: a trigger character at the start of the
/// text or after whitespace, followed by non-whitespace up to the caret.
/// `/` only counts at the very start, as a command. Returns the trigger, its
/// byte offset and the query after it.
pub fn active_trigger<'a>(
    text: &'a str,
    cursor: usize,
    triggers: &[char],
) -> Option<(char, usize, &'a str)> {
    let before = text.get(..cursor)?;
    let (start, trigger) = before
        .char_indices()
        .rev()
        .take_while(|(_, c)| !c.is_whitespace())
        .filter(|(_, c)| triggers.contains(c))
        .last()?;
    let preceded_by_space = before[..start]
        .chars()
        .next_back()
        .is_none_or(char::is_whitespace);
    if !preceded_by_space || (trigger == '/' && start != 0) {
        return None;
    }
    Some((trigger, start, &before[start + trigger.len_utf8()..]))
}

/// Retained composer state: the editor, triggers and autocomplete.
pub struct ComposerState {
    input: Entity<TextareaState>,
    triggers: Vec<char>,
    active: Option<ActiveTrigger>,
    dismissed: Option<usize>,
    suggestions: Vec<ComposerSuggestion>,
    highlighted: usize,
    submit_on_enter: bool,
    on_key: Option<KeyHandler>,
    on_paste: Option<PasteHandler>,
    leading: Option<SuggestionSlot>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ComposerEvent> for ComposerState {}

impl Focusable for ComposerState {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.input.read(cx).focus_handle(cx)
    }
}

impl ComposerState {
    /// Creates a composer that grows from one to `max_rows` lines.
    pub fn new(max_rows: usize, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| {
            TextareaState::new(window, cx)
                .auto_grow(1, max_rows.max(1))
                .placeholder("Write a message…")
        });
        let window_handle = window.window_handle();
        let weak = cx.weak_entity();
        let subscriptions = vec![
            cx.subscribe(&input, |this, _, event, cx| {
                if let InputEvent::Change = event {
                    cx.emit(ComposerEvent::Change);
                }
                this.refresh_trigger(cx);
            }),
            // Cursor moves notify without a change event.
            cx.observe(&input, |this, _, cx| this.refresh_trigger(cx)),
            // Keystrokes reach interceptors before the editor's own bindings,
            // so autocomplete keys, the caller's hook and Enter win over them.
            cx.intercept_keystrokes(move |event, window, cx| {
                if window.window_handle() != window_handle {
                    return;
                }
                let Some(state) = weak.upgrade() else {
                    return;
                };
                let focus = state.read(cx).focus_handle(cx);
                if !focus.is_focused(window) {
                    return;
                }
                if state.update(cx, |state, cx| {
                    state.handle_key(&event.keystroke, window, cx)
                }) {
                    cx.stop_propagation();
                }
            }),
        ];
        Self {
            input,
            triggers: vec!['@', '~', ':', '/'],
            active: None,
            dismissed: None,
            suggestions: Vec::new(),
            highlighted: 0,
            submit_on_enter: true,
            on_key: None,
            on_paste: None,
            leading: None,
            _subscriptions: subscriptions,
        }
    }

    /// The editing state, for anything not covered here.
    pub fn input(&self) -> &Entity<TextareaState> {
        &self.input
    }

    /// Sets the placeholder.
    pub fn set_placeholder(
        &mut self,
        placeholder: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let placeholder = placeholder.into();
        self.input.update(cx, |input, cx| {
            input.set_placeholder(placeholder, window, cx)
        });
    }

    /// Sets the characters that open autocomplete. Defaults to `@ ~ : /`.
    pub fn set_triggers(
        &mut self,
        triggers: impl IntoIterator<Item = char>,
        cx: &mut Context<Self>,
    ) {
        self.triggers = triggers.into_iter().collect();
        self.refresh_trigger(cx);
    }

    /// Whether Enter sends (Shift+Enter adds a line). On by default.
    pub fn set_submit_on_enter(&mut self, submit: bool) {
        self.submit_on_enter = submit;
    }

    /// Sees each key press before the composer and the editor, while the
    /// composer has focus and autocomplete is closed. Return `true` to claim
    /// it, for example Up in an empty composer to edit the last message.
    pub fn set_on_key(
        &mut self,
        handler: impl Fn(&Keystroke, &mut Window, &mut App) -> bool + 'static,
    ) {
        self.on_key = Some(Rc::new(handler));
    }

    /// Sees a paste first. Return `true` to claim it, for example when the
    /// clipboard holds an image or files to upload.
    pub fn set_on_paste(
        &mut self,
        handler: impl Fn(&ClipboardItem, &mut Window, &mut App) -> bool + 'static,
    ) {
        self.on_paste = Some(Rc::new(handler));
    }

    /// Renders the leading element of each suggestion, such as an avatar or
    /// emoji image. Return `None` to fall back to the suggestion's icon.
    pub fn set_suggestion_leading(
        &mut self,
        slot: impl Fn(&ComposerSuggestion, &mut Window, &mut App) -> Option<AnyElement> + 'static,
    ) {
        self.leading = Some(Rc::new(slot));
    }

    /// The text.
    pub fn value(&self, cx: &App) -> SharedString {
        self.input.read(cx).value()
    }

    /// Replaces the text, for example with an empty string after sending.
    pub fn set_value(
        &mut self,
        value: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let value: SharedString = value.into();
        let end = value.len();
        self.input.update(cx, |input, cx| {
            input.set_value(value, window, cx);
            input.set_selected_range(end..end, cx);
        });
        self.refresh_trigger(cx);
    }

    /// The selected byte range; empty at the caret.
    pub fn selection(&self, cx: &App) -> Range<usize> {
        self.input.read(cx).selected_range()
    }

    /// Selects a byte range, or places the caret with an empty range.
    pub fn set_selection(&mut self, range: Range<usize>, cx: &mut Context<Self>) {
        self.input
            .update(cx, |input, cx| input.set_selected_range(range, cx));
    }

    /// Replaces the selection (or inserts at the caret) and leaves the caret after it.
    pub fn insert(
        &mut self,
        text: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let text = text.into();
        self.input
            .update(cx, |input, cx| input.replace(text, window, cx));
    }

    /// Wraps the selection in `before` and `after`, such as `**` for bold,
    /// keeping the original text selected. With no selection the caret ends
    /// between them.
    pub fn wrap_selection(
        &mut self,
        before: &str,
        after: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = self.selection(cx);
        let value = self.value(cx);
        let selected = value.get(range.clone()).unwrap_or_default();
        let wrapped = format!("{before}{selected}{after}");
        let inner = range.start + before.len()..range.start + before.len() + selected.len();
        self.input.update(cx, |input, cx| {
            input.replace(wrapped, window, cx);
            input.set_selected_range(inner, cx);
        });
    }

    /// The caret's rectangle in window coordinates, from the last layout.
    pub fn caret_bounds(&self, cx: &App) -> Option<Bounds<Pixels>> {
        let input = self.input.read(cx);
        let cursor = input.cursor();
        input.range_to_bounds(&(cursor..cursor))
    }

    /// Whether an input method is composing text, such as CJK or a dead key.
    /// Keys during composition belong to the input method.
    pub fn is_composing(&self, window: &mut Window, cx: &mut App) -> bool {
        self.input.update(cx, |input, cx| {
            input.marked_text_range(window, cx).is_some()
        })
    }

    /// Focuses the editor.
    pub fn focus(&self, window: &mut Window, cx: &mut App) {
        self.input.update(cx, |input, cx| input.focus(window, cx));
    }

    /// Shows suggestions for the current trigger; an empty list hides the popup.
    pub fn set_suggestions(
        &mut self,
        suggestions: impl IntoIterator<Item = ComposerSuggestion>,
        cx: &mut Context<Self>,
    ) {
        self.suggestions = suggestions.into_iter().collect();
        self.highlighted = 0;
        cx.notify();
    }

    /// Whether the autocomplete popup is showing.
    pub fn is_autocomplete_open(&self) -> bool {
        self.active.is_some() && !self.suggestions.is_empty()
    }

    /// The highlighted suggestion while the popup is open.
    pub fn highlighted_suggestion(&self) -> Option<&ComposerSuggestion> {
        self.is_autocomplete_open()
            .then(|| self.suggestions.get(self.highlighted))
            .flatten()
    }

    /// Replaces the trigger and query with suggestion `index` and a space.
    pub fn accept(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let (Some(active), Some(suggestion)) =
            (self.active.clone(), self.suggestions.get(index).cloned())
        else {
            return;
        };
        let end = active.start + active.trigger.len_utf8() + active.query.len();
        let text = format!("{} ", suggestion.value);
        self.input.update(cx, |input, cx| {
            input.set_selected_range(active.start..end, cx);
            input.replace(text, window, cx);
        });
        self.active = None;
        self.suggestions.clear();
        cx.emit(ComposerEvent::Accept {
            trigger: active.trigger,
            value: suggestion.value,
        });
        cx.notify();
    }

    fn close_autocomplete(&mut self, cx: &mut Context<Self>) {
        if self.active.take().is_some() {
            self.suggestions.clear();
            cx.emit(ComposerEvent::AutocompleteClosed);
            cx.notify();
        }
    }

    fn refresh_trigger(&mut self, cx: &mut Context<Self>) {
        let input = self.input.read(cx);
        let value = input.value();
        let range = input.selected_range();
        let found = (range.is_empty())
            .then(|| active_trigger(&value, range.end, &self.triggers))
            .flatten()
            .map(|(trigger, start, query)| ActiveTrigger {
                trigger,
                start,
                query: query.to_string().into(),
            });
        if self.dismissed.is_some() && found.as_ref().map(|found| found.start) != self.dismissed {
            self.dismissed = None;
        }
        let found = found.filter(|found| Some(found.start) != self.dismissed);
        if found == self.active {
            return;
        }
        match found {
            Some(found) => {
                cx.emit(ComposerEvent::Autocomplete {
                    trigger: found.trigger,
                    query: found.query.clone(),
                });
                self.active = Some(found);
            }
            None => self.close_autocomplete(cx),
        }
        cx.notify();
    }

    fn handle_key(
        &mut self,
        keystroke: &Keystroke,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.is_composing(window, cx) {
            return false;
        }
        let modifiers = keystroke.modifiers;
        let plain = !(modifiers.control || modifiers.alt || modifiers.platform || modifiers.shift);
        if self.is_autocomplete_open() && plain {
            let count = self.suggestions.len();
            match keystroke.key.as_str() {
                "up" => self.highlighted = (self.highlighted + count - 1) % count,
                "down" => self.highlighted = (self.highlighted + 1) % count,
                "enter" | "tab" => {
                    self.accept(self.highlighted, window, cx);
                    return true;
                }
                "escape" => {
                    self.dismissed = self.active.as_ref().map(|active| active.start);
                    self.close_autocomplete(cx);
                    return true;
                }
                _ => return false,
            }
            cx.notify();
            return true;
        }
        if let Some(handler) = self.on_key.clone()
            && handler(keystroke, window, cx)
        {
            return true;
        }
        if self.submit_on_enter && keystroke.key == "enter" && plain {
            cx.emit(ComposerEvent::Submit(self.value(cx)));
            return true;
        }
        false
    }
}

impl Render for ComposerState {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let id = ElementId::from(("composer", cx.entity_id()));
        let on_paste = self.on_paste.clone();
        let textarea = Textarea::new(&self.input)
            .aria_label("Message")
            .bordered(false)
            .min_h(px(0.))
            .px(text_inset(&theme).x)
            .py(text_inset(&theme).y)
            .when_some(on_paste, |textarea, handler| {
                textarea.on_paste(move |item, window, cx| handler(item, window, cx))
            });

        let popup = self
            .active
            .clone()
            .filter(|_| !self.suggestions.is_empty())
            .and_then(|active| {
                // Both rectangles come from the editor's last layout, so their
                // difference stays right even if the page moved since then.
                let input = self.input.read(cx);
                let trigger = input.range_to_bounds(&(active.start..active.start));
                let (Some(trigger), Some(text)) = (trigger, input.text_bounds()) else {
                    // The caret has no position until the editor's first layout.
                    window.request_animation_frame();
                    return None;
                };
                let anchor = trigger.origin - text.origin + text_inset(&theme);
                Some((active, anchor))
            })
            .map(|(active, anchor)| {
                let leading = self.leading.clone();
                let rows: Vec<AnyElement> = self
                    .suggestions
                    .clone()
                    .into_iter()
                    .enumerate()
                    .map(|(index, suggestion)| {
                        let selected = index == self.highlighted;
                        let leading = leading
                            .as_ref()
                            .and_then(|slot| slot(&suggestion, window, cx));
                        div()
                            .id((
                                id.clone(),
                                SharedString::from(format!("suggestion-{index}")),
                            ))
                            .role(Role::ListBoxOption)
                            .aria_label(suggestion.label.clone())
                            .aria_selected(selected)
                            .when(selected, |row| row.aria_active_descendant())
                            .flex()
                            .items_center()
                            .gap(theme.space(2.))
                            .h(theme.space(8.))
                            .px(theme.space(2.))
                            .rounded(theme.radius.md)
                            .cursor_pointer()
                            .when(selected, |row| row.bg(colors.muted))
                            .on_hover(cx.listener(move |this, hovered, _, cx| {
                                if *hovered && this.highlighted != index {
                                    this.highlighted = index;
                                    cx.notify();
                                }
                            }))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.accept(index, window, cx);
                                this.focus(window, cx);
                            }))
                            .map(|row| match (leading, suggestion.icon) {
                                (Some(leading), _) => row.child(
                                    div().flex_shrink_0().flex().items_center().child(leading),
                                ),
                                (None, Some(icon)) => row.child(
                                    lucide(icon)
                                        .size(theme.space(4.))
                                        .flex_shrink_0()
                                        .text_color(colors.muted_foreground),
                                ),
                                (None, None) => row,
                            })
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(suggestion.label.clone()),
                            )
                            .children(suggestion.description.clone().map(|description| {
                                div()
                                    .min_w_0()
                                    .overflow_hidden()
                                    .text_ellipsis()
                                    .whitespace_nowrap()
                                    .text_color(colors.muted_foreground)
                                    .child(description)
                            }))
                            .into_any_element()
                    })
                    .collect();
                deferred(
                    anchored()
                        .position_mode(AnchoredPositionMode::Local)
                        .position(point(anchor.x - theme.space(2.), anchor.y - px(4.)))
                        .anchor(Anchor::BottomLeft)
                        .snap_to_window_with_margin(theme.space(2.))
                        .child(
                            div()
                                .id((id.clone(), "suggestions"))
                                .role(Role::ListBox)
                                .aria_label(match active.trigger {
                                    '@' => "People",
                                    '~' => "Channels",
                                    ':' => "Emoji",
                                    '/' => "Commands",
                                    _ => "Suggestions",
                                })
                                .occlude()
                                .w(theme.space(72.))
                                .max_h(theme.space(72.))
                                .overflow_y_scroll()
                                .p(theme.space(1.))
                                .rounded(theme.radius.lg)
                                .border_1()
                                .border_color(colors.border)
                                .bg(colors.popover)
                                .font_family(theme.fonts.body.clone())
                                .text_color(colors.popover_foreground)
                                .text_size(theme.text(14.))
                                .shadow(theme.shadows.md.to_vec())
                                .children(rows),
                        ),
                )
                .with_priority(1)
            });

        div()
            .id(id)
            .key_context("Composer")
            .relative()
            .w_full()
            .child(textarea)
            .children(popup.map(|popup| div().absolute().top_0().left_0().child(popup)))
    }
}

/// The editor's text origin inside the textarea: its padding.
fn text_inset(theme: &UiTheme) -> Point<Pixels> {
    point(theme.space(3.), theme.space(2.))
}

/// A composer bound to retained `ComposerState`, with an optional toolbar.
///
/// Children render in a row under the text, such as attach, emoji and send
/// buttons; the composer draws the border and focus ring around both.
#[derive(IntoElement)]
pub struct Composer {
    state: Entity<ComposerState>,
    toolbar: Vec<AnyElement>,
    style: StyleRefinement,
}

impl Composer {
    /// Creates a composer for retained state.
    pub fn new(state: &Entity<ComposerState>) -> Self {
        Self {
            state: state.clone(),
            toolbar: Vec::new(),
            style: StyleRefinement::default(),
        }
    }
}

impl ParentElement for Composer {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.toolbar.extend(elements);
    }
}

impl Styled for Composer {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Composer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let focused = self.state.read(cx).focus_handle(cx).is_focused(window);
        let root = div()
            .w_full()
            .flex()
            .flex_col()
            .rounded(theme.radius.lg)
            .border_1()
            .border_color(if focused { colors.ring } else { colors.input })
            .bg(colors.background)
            .child(self.state.clone())
            .when(!self.toolbar.is_empty(), |root| {
                root.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(theme.space(1.))
                        .px(theme.space(2.))
                        .pb(theme.space(2.))
                        .children(self.toolbar),
                )
            });
        apply_style(root, &self.style)
    }
}

#[cfg(test)]
mod tests {
    use super::active_trigger;

    const TRIGGERS: [char; 4] = ['@', '~', ':', '/'];

    #[test]
    fn triggers_start_after_whitespace_and_end_at_the_caret() {
        assert_eq!(active_trigger("hi @ar", 6, &TRIGGERS), Some(('@', 3, "ar")));
        assert_eq!(active_trigger("@", 1, &TRIGGERS), Some(('@', 0, "")));
        assert_eq!(
            active_trigger("see ~gen", 8, &TRIGGERS),
            Some(('~', 4, "gen"))
        );
        assert_eq!(
            active_trigger("nice :ta", 8, &TRIGGERS),
            Some((':', 5, "ta"))
        );
        assert_eq!(
            active_trigger("/away", 5, &TRIGGERS),
            Some(('/', 0, "away"))
        );
        // Inside a word, after a space, or a slash mid-message: no trigger.
        assert_eq!(active_trigger("mail@host", 9, &TRIGGERS), None);
        assert_eq!(active_trigger("@arya done", 10, &TRIGGERS), None);
        assert_eq!(active_trigger("a/b", 3, &TRIGGERS), None);
        assert_eq!(active_trigger("x /cmd", 6, &TRIGGERS), None);
        // The caret, not the end of the text, ends the query.
        assert_eq!(active_trigger("@arya", 3, &TRIGGERS), Some(('@', 0, "ar")));
    }
}
