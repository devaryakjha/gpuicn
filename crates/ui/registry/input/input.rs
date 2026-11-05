//! Nova presentation for GPUI Kit's single-line and multi-line editing states.

use std::ops::Range;
use std::rc::Rc;

use super::theme::{
    UiTheme,
    primitives::{control, disabled, focus},
};
pub use gpui_kit::base::input::{InputEvent, InputState, TextareaState};
use gpui_kit::base::{
    InputBase,
    input::{InputBaseState, InputEditorStyle, InputModeKind},
};
use gpui_kit::component::native_menu::NativeMenu;
use gpui_kit::{
    AccessibleAction, App, ClipboardItem, ContentMask, Context, ElementId, Entity,
    EntityInputHandler as _, EventEmitter, FocusHandle, Focusable as _, InteractiveElement as _,
    IntoElement, MouseButton, ParentElement as _, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, Subscription, UnderlineStyle, Window, canvas, point,
    prelude::FluentBuilder as _, px,
};

type PasteHandler = Rc<dyn Fn(&ClipboardItem, &mut Window, &mut App) -> bool>;

/// The caller owns the editing state, focus and event subscriptions.
#[derive(IntoElement)]
pub struct Input {
    state: Entity<InputState>,
    style: gpui_kit::StyleRefinement,
    label: Option<SharedString>,
    disabled: bool,
    read_only: bool,
    invalid: bool,
    bordered: bool,
    paste_handler: Option<PasteHandler>,
}

/// Nova presentation for GPUI Kit's ordinary multi-line editing state.
#[derive(IntoElement)]
pub struct Textarea {
    state: Entity<TextareaState>,
    style: gpui_kit::StyleRefinement,
    label: Option<SharedString>,
    disabled: bool,
    read_only: bool,
    invalid: bool,
    bordered: bool,
    paste_handler: Option<PasteHandler>,
    misspellings: Option<Entity<Misspellings>>,
}

/// Replaces a misspelled range with a suggestion in one undoable edit.
///
/// `SpellingMenu::suggestion` adds menu items that dispatch it; a textarea
/// showing `Misspellings` handles it.
#[derive(Clone, PartialEq, Debug, gpui_kit::Action)]
#[action(namespace = gpuicn, no_json)]
pub struct ReplaceMisspelling {
    /// The byte range to replace.
    pub range: Range<usize>,
    /// The replacement text.
    pub text: SharedString,
}

type SpellingMenuHandler = Rc<dyn Fn(&mut SpellingMenu, &mut Window, &mut App)>;

/// Misspelled byte ranges in a textarea, drawn as wavy underlines.
///
/// The application checks the spelling itself, with the platform's checker,
/// and passes the ranges to `set` a moment after typing stops. Edits shift
/// the ranges after them and drop the ones they touch, so no underline sits
/// on the wrong text before fresh ranges arrive. Attach it with
/// `Textarea::misspellings`; `ComposerState` has one built in.
pub struct Misspellings {
    input: Entity<TextareaState>,
    ranges: Vec<Range<usize>>,
    text: SharedString,
    menu: Option<SpellingMenuHandler>,
    _subscription: Subscription,
}

/// Notifications from `Misspellings`.
#[derive(Clone, Debug, PartialEq)]
pub enum MisspellingsEvent {
    /// A suggestion replaced a misspelled range.
    Replaced {
        /// The range that was replaced, in the text before the edit.
        range: Range<usize>,
        /// The text that replaced it.
        text: SharedString,
    },
}

impl EventEmitter<MisspellingsEvent> for Misspellings {}

impl Misspellings {
    /// Tracks misspellings for `input`.
    pub fn new(input: &Entity<TextareaState>, cx: &mut Context<Self>) -> Self {
        let subscription = cx.subscribe(input, |this, input, event, cx| {
            if let InputEvent::Change = event {
                let text = input.read(cx).value();
                this.follow_edit(&text);
                this.text = text;
                cx.notify();
            }
        });
        Self {
            text: input.read(cx).value(),
            input: input.clone(),
            ranges: Vec::new(),
            menu: None,
            _subscription: subscription,
        }
    }

    /// Replaces the misspelled ranges: byte offsets into the current value.
    /// Ranges outside the text or off character boundaries are dropped.
    pub fn set(&mut self, ranges: impl IntoIterator<Item = Range<usize>>, cx: &mut Context<Self>) {
        let text = self.input.read(cx).value();
        let mut ranges: Vec<_> = ranges
            .into_iter()
            .filter(|range| {
                range.start < range.end
                    && text.is_char_boundary(range.start)
                    && text.is_char_boundary(range.end)
            })
            .collect();
        ranges.sort_by_key(|range| range.start);
        self.ranges = ranges;
        self.text = text;
        cx.notify();
    }

    /// The current misspelled ranges, in order.
    pub fn ranges(&self) -> &[Range<usize>] {
        &self.ranges
    }

    /// The misspelled range at byte `offset`, its ends included.
    pub fn range_at(&self, offset: usize) -> Option<Range<usize>> {
        self.ranges
            .iter()
            .find(|range| range.start <= offset && offset <= range.end)
            .cloned()
    }

    /// Builds the menu for a right-click inside a misspelled range. Its items
    /// go above Cut, Copy, Paste and Select All; a right-click elsewhere shows
    /// only those.
    pub fn on_menu(&mut self, handler: impl Fn(&mut SpellingMenu, &mut Window, &mut App) + 'static) {
        self.menu = Some(Rc::new(handler));
    }

    /// Replaces `range` with `text` as one undoable edit and selects nothing,
    /// leaving the caret after it.
    pub fn replace(
        &mut self,
        range: Range<usize>,
        text: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let text = text.into();
        let value = self.input.read(cx).value();
        if range.end > value.len()
            || !value.is_char_boundary(range.start)
            || !value.is_char_boundary(range.end)
        {
            return;
        }
        let replacement = text.clone();
        self.input.update(cx, |input, cx| {
            input.set_selected_range(range.clone(), cx);
            input.replace(replacement, window, cx);
        });
        cx.emit(MisspellingsEvent::Replaced { range, text });
    }

    /// Moves the ranges after an edit and drops the ranges it touched.
    fn follow_edit(&mut self, new: &str) {
        let old = self.text.as_ref();
        if old == new || self.ranges.is_empty() {
            return;
        }
        let prefix = old
            .char_indices()
            .zip(new.chars())
            .find(|((_, a), b)| a != b)
            .map_or(old.len().min(new.len()), |((index, _), _)| index);
        let suffix = old[prefix..]
            .chars()
            .rev()
            .zip(new[prefix..].chars().rev())
            .take_while(|(a, b)| a == b)
            .map(|(a, _)| a.len_utf8())
            .sum::<usize>();
        let removed = prefix..old.len() - suffix;
        let inserted = new.len() - suffix - prefix;
        self.ranges.retain_mut(|range| {
            if range.end < removed.start || (range.end == removed.start && inserted == 0) {
                true
            } else if range.start > removed.end
                || (range.start == removed.end && removed.end > removed.start)
            {
                range.start = range.start - removed.len() + inserted;
                range.end = range.end - removed.len() + inserted;
                true
            } else {
                false
            }
        });
    }
}

/// The items a right-click on a misspelled word adds above Cut, Copy and
/// Paste: suggestions, then the application's own actions such as
/// "Ignore" or "Add to dictionary".
pub struct SpellingMenu {
    range: Range<usize>,
    word: SharedString,
    menu: NativeMenu,
}

impl SpellingMenu {
    /// The misspelled byte range under the pointer.
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }
    /// The misspelled word.
    pub fn word(&self) -> &SharedString {
        &self.word
    }
    /// Adds a suggestion; choosing it replaces the word in one undoable edit.
    pub fn suggestion(&mut self, text: impl Into<SharedString>) -> &mut Self {
        let text = text.into();
        let action = ReplaceMisspelling {
            range: self.range.clone(),
            text: text.clone(),
        };
        self.push(|menu| menu.menu(text, Box::new(action)));
        self
    }
    /// Adds an item that dispatches the application's own action.
    pub fn item(
        &mut self,
        label: impl Into<SharedString>,
        action: Box<dyn gpui_kit::Action>,
    ) -> &mut Self {
        let label = label.into();
        self.push(|menu| menu.menu(label, action));
        self
    }
    /// Adds a separator line.
    pub fn separator(&mut self) -> &mut Self {
        self.push(NativeMenu::separator);
        self
    }
    fn push(&mut self, add: impl FnOnce(NativeMenu) -> NativeMenu) {
        self.menu = add(std::mem::take(&mut self.menu));
    }
}

/// Cut, Copy, Paste and Select All, after `menu`'s own items.
fn edit_menu(menu: NativeMenu, copyable: bool, editable: bool) -> NativeMenu {
    let menu = if menu.is_empty() {
        menu
    } else {
        menu.separator()
    };
    menu.menu_with_disabled(
        "Cut",
        !(editable && copyable),
        Box::new(gpui_kit::base::input::Cut),
    )
    .menu_with_disabled("Copy", !copyable, Box::new(gpui_kit::base::input::Copy))
    .menu_with_disabled("Paste", !editable, Box::new(gpui_kit::base::input::Paste))
    .separator()
    .menu("Select All", Box::new(gpui_kit::base::input::SelectAll))
}

/// Paints a 1px wavy underline under each misspelled range that is laid out.
fn misspelling_underlines(
    input: Entity<TextareaState>,
    misspellings: Entity<Misspellings>,
    theme: &UiTheme,
) -> impl IntoElement {
    let color = theme.colors.destructive;
    canvas(
        |_, _, _| {},
        move |bounds, _, window, cx| {
            let input = input.read(cx);
            let style = UnderlineStyle {
                thickness: px(1.),
                color: Some(color.into()),
                wavy: true,
            };
            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                for range in misspellings.read(cx).ranges() {
                    let Some(word) = input.range_to_bounds(range) else {
                        continue;
                    };
                    let line = input.line_height().unwrap_or(px(20.));
                    // A word that wraps onto the next line has no single box.
                    if word.size.height > line * 1.5 || word.size.width <= px(0.) {
                        continue;
                    }
                    let origin = point(word.left(), word.bottom() - px(3.));
                    window.paint_underline(origin, word.size.width, &style);
                }
            });
        },
    )
    .absolute()
    .inset_0()
}

impl Textarea {
    /// Creates a Nova textarea using the caller's retained Kit editing state.
    pub fn new(state: &Entity<TextareaState>) -> Self {
        Self {
            state: state.clone(),
            style: Default::default(),
            label: None,
            disabled: false,
            read_only: false,
            invalid: false,
            bordered: true,
            paste_handler: None,
            misspellings: None,
        }
    }
    /// Underlines the ranges in `misspellings` with a wavy destructive line
    /// and adds its spelling items, then Cut, Copy, Paste and Select All, to
    /// the right-click menu.
    pub fn misspellings(mut self, misspellings: &Entity<Misspellings>) -> Self {
        self.misspellings = Some(misspellings.clone());
        self
    }
    /// Sets the accessible name of the control.
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
    /// Disables interaction and applies the disabled appearance.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    /// Prevents user changes while preserving focus and reading.
    pub fn read_only(mut self, value: bool) -> Self {
        self.read_only = value;
        self
    }
    /// Applies validation error colors and an invalid accessibility description.
    pub fn invalid(mut self, value: bool) -> Self {
        self.invalid = value;
        self
    }
    /// Controls the Nova textarea border and background.
    pub fn bordered(mut self, value: bool) -> Self {
        self.bordered = value;
        self
    }
    /// Intercepts native clipboard payloads before the default text insertion.
    /// Returning `true` consumes the paste; `false` keeps normal text paste.
    /// Browser image/file paste needs async clipboard permission and is not
    /// available through this synchronous hook.
    pub fn on_paste(
        mut self,
        handler: impl Fn(&ClipboardItem, &mut Window, &mut App) -> bool + 'static,
    ) -> Self {
        self.paste_handler = Some(Rc::new(handler));
        self
    }
    #[doc(hidden)]
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
    #[doc(hidden)]
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }
    #[doc(hidden)]
    pub fn is_invalid(&self) -> bool {
        self.invalid
    }
    #[doc(hidden)]
    pub fn field_focus_handle(&self, cx: &App) -> FocusHandle {
        self.state.read(cx).focus_handle(cx)
    }
}

impl Styled for Textarea {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}
impl Input {
    /// Creates a Nova input using the caller's retained Kit editing state.
    pub fn new(state: &Entity<InputState>) -> Self {
        Self {
            state: state.clone(),
            style: Default::default(),
            label: None,
            disabled: false,
            read_only: false,
            invalid: false,
            bordered: true,
            paste_handler: None,
        }
    }
    /// Sets the accessible name of the control.
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
    /// Disables interaction and applies the disabled appearance.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    /// Prevents user changes while preserving focus and reading.
    pub fn read_only(mut self, value: bool) -> Self {
        self.read_only = value;
        self
    }
    /// Applies validation error colors and an invalid accessibility description.
    pub fn invalid(mut self, value: bool) -> Self {
        self.invalid = value;
        self
    }
    /// Controls the Nova input border and background.
    pub fn bordered(mut self, value: bool) -> Self {
        self.bordered = value;
        self
    }
    /// Intercepts native clipboard payloads before the default text insertion.
    /// Returning `true` consumes the paste; `false` keeps normal text paste.
    /// Browser image/file paste needs async clipboard permission and is not
    /// available through this synchronous hook.
    pub fn on_paste(
        mut self,
        handler: impl Fn(&ClipboardItem, &mut Window, &mut App) -> bool + 'static,
    ) -> Self {
        self.paste_handler = Some(Rc::new(handler));
        self
    }
    #[doc(hidden)]
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
    #[doc(hidden)]
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }
    #[doc(hidden)]
    pub fn is_invalid(&self) -> bool {
        self.invalid
    }
    #[doc(hidden)]
    pub fn field_focus_handle(&self, cx: &App) -> FocusHandle {
        self.state.read(cx).focus_handle(cx)
    }
}
impl Styled for Input {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}
impl RenderOnce for Input {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let core = InputCore {
            label: self.label,
            disabled: self.disabled,
            read_only: self.read_only,
            invalid: self.invalid,
            bordered: self.bordered,
            paste_handler: self.paste_handler,
        };
        let root = core
            .render("input", self.state, &theme, window, cx)
            .h(theme.space(8.))
            .items_center();
        super::theme::apply_style(root, &self.style)
    }
}

impl RenderOnce for Textarea {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let core = InputCore {
            label: self.label,
            disabled: self.disabled,
            read_only: self.read_only,
            invalid: self.invalid,
            bordered: self.bordered,
            paste_handler: self.paste_handler,
        };
        let input = self.state.clone();
        let root = core
            .render("textarea", self.state, &theme, window, cx)
            .min_h(theme.space(16.))
            .items_start()
            .py(theme.space(2.));
        let editable = !self.disabled && !self.read_only;
        let root = match self.misspellings {
            Some(misspellings) => {
                install_spelling_menu(&input, &misspellings, editable, cx);
                let replacer = misspellings.clone();
                root.relative()
                    .when(editable, |root| {
                        root.on_action(move |action: &ReplaceMisspelling, window, cx| {
                            replacer.update(cx, |misspellings, cx| {
                                misspellings.replace(
                                    action.range.clone(),
                                    action.text.clone(),
                                    window,
                                    cx,
                                )
                            });
                        })
                    })
                    .child(misspelling_underlines(input, misspellings, &theme))
            }
            None => root,
        };
        super::theme::apply_style(root, &self.style)
    }
}

/// Shows the spelling items and the edit items on a right-click.
fn install_spelling_menu(
    input: &Entity<TextareaState>,
    misspellings: &Entity<Misspellings>,
    editable: bool,
    cx: &mut App,
) {
    let misspellings = misspellings.downgrade();
    let weak_input = input.downgrade();
    input.update(cx, |state, _| {
        state.on_context_menu(Rc::new(move |_, capabilities, position, window, cx| {
            let (Some(input), Some(misspellings)) = (weak_input.upgrade(), misspellings.upgrade())
            else {
                return;
            };
            // The right-click has already moved the caret to the word.
            let (cursor, value) = {
                let input = input.read(cx);
                (input.cursor(), input.value())
            };
            let found = misspellings.read(cx).range_at(cursor);
            let handler = misspellings.read(cx).menu.clone();
            let mut menu = NativeMenu::new();
            if let (Some(range), Some(handler), true) = (found, handler, editable) {
                let mut spelling = SpellingMenu {
                    word: value.get(range.clone()).unwrap_or_default().to_string().into(),
                    range,
                    menu,
                };
                handler(&mut spelling, window, cx);
                menu = spelling.menu;
            }
            edit_menu(
                menu,
                capabilities.is_copyable(),
                editable && !capabilities.is_readonly(),
            )
            .show(position, window, cx);
        }));
    });
}

/// The Nova editing frame around any Kit input state. `Input` and `Textarea`
/// differ only in their size, so both build on this.
#[doc(hidden)]
pub struct InputCore {
    pub label: Option<SharedString>,
    pub disabled: bool,
    pub read_only: bool,
    pub invalid: bool,
    pub bordered: bool,
    pub paste_handler: Option<PasteHandler>,
}

impl InputCore {
    /// The editor colors every Kit input in a gpuicn app uses.
    pub fn editor_style(theme: &UiTheme) -> InputEditorStyle {
        let colors = theme.colors;
        InputEditorStyle {
            foreground: colors.foreground.into(),
            muted_foreground: colors.muted_foreground.into(),
            background: colors.background.into(),
            border: colors.input.into(),
            selection: colors.ring.opacity(0.3).into(),
            caret: colors.foreground.into(),
            ..Default::default()
        }
    }

    #[doc(hidden)]
    pub fn render<M: InputModeKind>(
        self,
        kind: &'static str,
        state: Entity<InputBaseState<M>>,
        theme: &UiTheme,
        window: &mut Window,
        cx: &mut App,
    ) -> InputBase {
        let editor_style = Self::editor_style(theme);
        state.update(cx, |state, cx| {
            state.set_disabled(self.disabled, cx);
            state.set_readonly(self.read_only, cx);
            state.set_editor_style(editor_style);
        });
        let read = state.read(cx);
        let focus_handle = read.focus_handle(cx);
        let focused = focus_handle.is_focused(window) && !self.disabled;
        let value = read.value();
        let placeholder = read.presentation().placeholder().clone();
        let action_state = state.clone();
        let click_state = state.clone();
        let a11y_focus = state.clone();
        let paste_handler = self
            .paste_handler
            .filter(|_| !self.disabled && !self.read_only);
        let root = InputBase::new(ElementId::from((kind, state.entity_id())))
            .focused(focused)
            .disabled(self.disabled)
            .when_some(self.label, |root, label| root.accessibility_label(label))
            .aria_value(value)
            .aria_placeholder(placeholder)
            .when(self.read_only, |root| root.aria_description("Read only"))
            .when(self.invalid, |root| root.aria_description("Invalid value"))
            .when(!self.disabled, |root| {
                root.on_a11y_action(AccessibleAction::Focus, move |_, window, cx| {
                    a11y_focus.update(cx, |state, cx| state.focus(window, cx));
                })
            })
            .when(!self.disabled, |root| {
                root.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    click_state.update(cx, |state, cx| state.focus(window, cx));
                })
            })
            .when(!self.disabled && !self.read_only, |root| {
                root.on_a11y_action(AccessibleAction::SetValue, move |data, window, cx| {
                    if let Some(gpui_kit::accesskit::ActionData::Value(value)) = data {
                        action_state.update(cx, |state, cx| {
                            let end = state.value().encode_utf16().count();
                            state.replace_text_in_range(Some(0..end), value, window, cx);
                        });
                    }
                })
            })
            .flex()
            .w_full()
            .min_w_0()
            .px(theme.space(2.5))
            .text_color(theme.colors.foreground);
        let root = if self.bordered {
            focus::shown_for(
                control::frame(root, theme, self.invalid),
                theme,
                focused,
                self.invalid,
            )
            // The ring is a shadow painted under the whole box; a translucent
            // fill would show it across a tall textarea.
            .when(focused, |root| root.bg(theme.opaque(theme.control().background)))
        } else {
            super::theme::primitives::text::sm(root, theme)
        };
        root.when(self.disabled, disabled::style)
            .when_some(paste_handler, |root, handler| {
                root.capture_action(move |_: &gpui_kit::base::input::Paste, window, cx| {
                    if let Some(clipboard) = cx.read_from_clipboard()
                        && handler(&clipboard, window, cx)
                    {
                        cx.stop_propagation();
                    }
                })
            })
            .child(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{AppContext as _, Context, Render, TestAppContext, VisualTestContext};
    use std::{cell::Cell, rc::Rc};

    struct PasteProbe {
        input: Entity<InputState>,
        textarea: Entity<TextareaState>,
        multiline: bool,
        disabled: bool,
        read_only: bool,
        hits: Rc<Cell<usize>>,
    }

    impl Render for PasteProbe {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let hits = self.hits.clone();
            let handler = move |item: &ClipboardItem, _: &mut Window, _: &mut App| {
                hits.set(hits.get() + 1);
                item.text().as_deref() == Some("asset")
            };
            if self.multiline {
                Textarea::new(&self.textarea)
                    .disabled(self.disabled)
                    .read_only(self.read_only)
                    .on_paste(handler)
                    .into_any_element()
            } else {
                Input::new(&self.input)
                    .disabled(self.disabled)
                    .read_only(self.read_only)
                    .on_paste(handler)
                    .into_any_element()
            }
        }
    }

    #[gpui_kit::test]
    fn paste_hooks_consume_payloads_and_respect_editability(cx: &mut TestAppContext) {
        cx.update(super::super::theme::init);
        for multiline in [false, true] {
            let hits = Rc::new(Cell::new(0));
            let (probe, visual) = cx.add_window_view({
                let hits = hits.clone();
                move |window, cx| PasteProbe {
                    input: cx.new(|cx| InputState::new(window, cx)),
                    textarea: cx.new(|cx| TextareaState::new(window, cx)),
                    multiline,
                    disabled: false,
                    read_only: false,
                    hits,
                }
            });
            let draw = |visual: &mut VisualTestContext| {
                visual.update(|window, cx| window.draw(cx).clear(cx));
            };
            let dispatch = |visual: &mut VisualTestContext, text: &str| {
                visual.update(|window, cx| {
                    if multiline {
                        let textarea = probe.read(cx).textarea.clone();
                        textarea.update(cx, |state, cx| state.focus(window, cx));
                    } else {
                        let input = probe.read(cx).input.clone();
                        input.update(cx, |state, cx| state.focus(window, cx));
                    }
                    cx.write_to_clipboard(ClipboardItem::new_string(text.into()));
                    window.dispatch_action(Box::new(gpui_kit::base::input::Paste), cx);
                });
            };
            let value = |visual: &mut VisualTestContext| {
                visual.update(|_, cx| {
                    let probe = probe.read(cx);
                    if multiline {
                        probe.textarea.read(cx).value().to_string()
                    } else {
                        probe.input.read(cx).value().to_string()
                    }
                })
            };

            draw(visual);
            dispatch(visual, "asset");
            assert_eq!(hits.get(), 1);
            assert_eq!(value(visual), "");

            dispatch(visual, "text");
            assert_eq!(hits.get(), 2);
            assert_eq!(value(visual), "text");

            for (disabled, read_only) in [(true, false), (false, true)] {
                probe.update(visual, |probe, cx| {
                    probe.disabled = disabled;
                    probe.read_only = read_only;
                    cx.notify();
                });
                draw(visual);
                dispatch(visual, "asset");
                assert_eq!(hits.get(), 2);
                assert_eq!(value(visual), "text");
            }
        }
    }
}

#[cfg(test)]
mod spelling_tests {
    use super::*;
    use gpui_kit::{AppContext as _, Render, TestAppContext, VisualTestContext};

    struct Notes {
        input: Entity<TextareaState>,
        misspellings: Entity<Misspellings>,
    }

    impl Render for Notes {
        fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
            gpui_kit::div().w(px(400.)).child(
                Textarea::new(&self.input)
                    .aria_label("Notes")
                    .misspellings(&self.misspellings),
            )
        }
    }

    fn notes(text: &'static str, cx: &mut TestAppContext) -> (Entity<Notes>, VisualTestContext) {
        cx.update(super::super::theme::init);
        let window = cx.add_window(move |window, cx| {
            let input = cx.new(|cx| TextareaState::new(window, cx));
            input.update(cx, |input, cx| input.set_value(text, window, cx));
            let misspellings = cx.new(|cx| Misspellings::new(&input, cx));
            Notes {
                input,
                misspellings,
            }
        });
        let view = window.root(cx).unwrap();
        let mut cx = VisualTestContext::from_window(window.into(), cx);
        cx.update(|window, cx| {
            window.activate_window();
            let input = view.read(cx).input.clone();
            input.update(cx, |input, cx| input.focus(window, cx));
            window.draw(cx).clear(cx);
        });
        (view, cx)
    }

    #[gpui_kit::test]
    fn edits_move_later_ranges_and_drop_touched_ones(cx: &mut TestAppContext) {
        let (view, mut cx) = notes("teh cat sat on teh mat", cx);
        let (input, misspellings) = cx.update(|_, cx| {
            let view = view.read(cx);
            (view.input.clone(), view.misspellings.clone())
        });
        cx.update(|_, cx| misspellings.update(cx, |m, cx| m.set([0..3, 15..18, 30..40], cx)));
        cx.update(|_, cx| {
            assert_eq!(misspellings.read(cx).ranges(), [0..3, 15..18], "out of range dropped")
        });
        // Typing before the second word moves it.
        cx.update(|window, cx| {
            input.update(cx, |input, cx| {
                input.set_selected_range(4..4, cx);
                input.replace("big ", window, cx);
            })
        });
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(misspellings.read(cx).ranges(), [0..3, 19..22]));
        // Typing inside the first word drops it.
        cx.update(|window, cx| {
            input.update(cx, |input, cx| {
                input.set_selected_range(1..1, cx);
                input.replace("x", window, cx);
            })
        });
        cx.run_until_parked();
        cx.update(|_, cx| {
            let misspellings = misspellings.read(cx);
            assert_eq!(misspellings.ranges(), [20..23]);
            assert_eq!(misspellings.range_at(21), Some(20..23));
            assert_eq!(misspellings.range_at(5), None);
        });
    }

    #[gpui_kit::test]
    fn a_suggestion_replaces_the_word_in_one_undoable_edit(cx: &mut TestAppContext) {
        let (view, mut cx) = notes("see teh cat", cx);
        let input = cx.update(|_, cx| view.read(cx).input.clone());
        cx.update(|_, cx| {
            view.read(cx)
                .misspellings
                .clone()
                .update(cx, |m, cx| m.set([4..7], cx));
        });
        cx.dispatch_action(ReplaceMisspelling {
            range: 4..7,
            text: "the".into(),
        });
        cx.update(|_, cx| assert_eq!(input.read(cx).value(), "see the cat"));
        cx.simulate_keystrokes("secondary-z");
        cx.update(|_, cx| assert_eq!(input.read(cx).value(), "see teh cat"));
    }

    #[test]
    fn the_spelling_menu_puts_suggestions_above_the_edit_items() {
        let mut menu = SpellingMenu {
            range: 0..3,
            word: "teh".into(),
            menu: NativeMenu::new(),
        };
        menu.suggestion("the").suggestion("ten");
        let menu = edit_menu(menu.menu, false, true);
        assert!(!menu.is_empty());
    }
}
