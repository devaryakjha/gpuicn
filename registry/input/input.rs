//! Nova presentation for GPUI Kit's single-line and multi-line editing states.

use std::rc::Rc;

use super::theme::{ThemeMode, UiTheme};
pub use gpui_kit::base::input::{InputEvent, InputState, TextareaState};
use gpui_kit::base::{InputBase, input::InputEditorStyle};
use gpui_kit::{
    AccessibleAction, App, ClipboardItem, ElementId, Entity, EntityInputHandler as _, FocusHandle,
    Focusable as _, InteractiveElement as _, IntoElement, MouseButton, ParentElement as _,
    RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window,
    prelude::FluentBuilder as _,
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
    pub(crate) fn is_disabled(&self) -> bool {
        self.disabled
    }
    pub(crate) fn is_read_only(&self) -> bool {
        self.read_only
    }
    pub(crate) fn is_invalid(&self) -> bool {
        self.invalid
    }
    pub(crate) fn field_focus_handle(&self, cx: &App) -> FocusHandle {
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
    pub(crate) fn is_disabled(&self) -> bool {
        self.disabled
    }
    pub(crate) fn is_read_only(&self) -> bool {
        self.read_only
    }
    pub(crate) fn is_invalid(&self) -> bool {
        self.invalid
    }
    pub(crate) fn field_focus_handle(&self, cx: &App) -> FocusHandle {
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
        let colors = theme.colors;
        self.state.update(cx, |state, cx| {
            state.set_disabled(self.disabled, cx);
            state.set_readonly(self.read_only, cx);
            state.set_editor_style(InputEditorStyle {
                foreground: colors.foreground.into(),
                muted_foreground: colors.muted_foreground.into(),
                background: colors.background.into(),
                border: colors.input.into(),
                selection: colors.ring.opacity(0.3).into(),
                caret: colors.foreground.into(),
                ..Default::default()
            });
        });
        let state = self.state.read(cx);
        let focus = state.focus_handle(cx);
        let focused = focus.is_focused(window) && !self.disabled;
        let value = state.value();
        let placeholder = state.presentation().placeholder().clone();
        let action_state = self.state.clone();
        let click_state = self.state.clone();
        let a11y_focus = self.state.clone();
        let paste_handler = self
            .paste_handler
            .filter(|_| !self.disabled && !self.read_only);
        let root = InputBase::new(ElementId::from(("input", self.state.entity_id())))
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
            .h(theme.space(8.))
            .items_center()
            .px(theme.space(2.5))
            .font_family(theme.fonts.body.clone())
            .text_size(theme.text(14.))
            .line_height(theme.text(20.))
            .text_color(colors.foreground)
            .when(self.bordered, |root| {
                root.rounded(theme.radius.lg)
                    .border_1()
                    .border_color(if self.invalid {
                        colors.destructive
                    } else {
                        colors.input
                    })
                    .bg(match theme.mode {
                        ThemeMode::Light => colors.background.opacity(0.),
                        ThemeMode::Dark => colors.input.opacity(0.3),
                    })
            })
            .when(focused && self.bordered, |root| {
                root.border_color(if self.invalid {
                    colors.destructive
                } else {
                    colors.ring
                })
            })
            .when(self.disabled, |root| root.opacity(0.5).cursor_not_allowed())
            .when_some(paste_handler, |root, handler| {
                root.capture_action(move |_: &gpui_kit::base::input::Paste, window, cx| {
                    if let Some(clipboard) = cx.read_from_clipboard()
                        && handler(&clipboard, window, cx)
                    {
                        cx.stop_propagation();
                    }
                })
            })
            .child(self.state);
        super::theme::apply_style(root, &self.style)
    }
}

impl RenderOnce for Textarea {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        self.state.update(cx, |state, cx| {
            state.set_disabled(self.disabled, cx);
            state.set_readonly(self.read_only, cx);
            state.set_editor_style(InputEditorStyle {
                foreground: colors.foreground.into(),
                muted_foreground: colors.muted_foreground.into(),
                background: colors.background.into(),
                border: colors.input.into(),
                selection: colors.ring.opacity(0.3).into(),
                caret: colors.foreground.into(),
                ..Default::default()
            });
        });
        let state = self.state.read(cx);
        let focus = state.focus_handle(cx);
        let focused = focus.is_focused(window) && !self.disabled;
        let value = state.value();
        let placeholder = state.presentation().placeholder().clone();
        let action_state = self.state.clone();
        let click_state = self.state.clone();
        let a11y_focus = self.state.clone();
        let paste_handler = self
            .paste_handler
            .filter(|_| !self.disabled && !self.read_only);
        let root = InputBase::new(ElementId::from(("textarea", self.state.entity_id())))
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
            .min_h(theme.space(16.))
            .items_start()
            .px(theme.space(2.5))
            .py(theme.space(2.))
            .font_family(theme.fonts.body.clone())
            .text_size(theme.text(14.))
            .line_height(theme.text(20.))
            .text_color(colors.foreground)
            .when(self.bordered, |root| {
                root.rounded(theme.radius.lg)
                    .border_1()
                    .border_color(if self.invalid {
                        colors.destructive
                    } else {
                        colors.input
                    })
                    .bg(match theme.mode {
                        ThemeMode::Light => colors.background.opacity(0.),
                        ThemeMode::Dark => colors.input.opacity(0.3),
                    })
            })
            .when(focused && self.bordered, |root| {
                root.border_color(if self.invalid {
                    colors.destructive
                } else {
                    colors.ring
                })
            })
            .when(self.disabled, |root| root.opacity(0.5).cursor_not_allowed())
            .when_some(paste_handler, |root, handler| {
                root.capture_action(move |_: &gpui_kit::base::input::Paste, window, cx| {
                    if let Some(clipboard) = cx.read_from_clipboard()
                        && handler(&clipboard, window, cx)
                    {
                        cx.stop_propagation();
                    }
                })
            })
            .child(self.state);
        super::theme::apply_style(root, &self.style)
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
