//! Nova presentation for GPUI Kit's single-line editing state.

use super::theme::{ThemeMode, UiTheme};
pub use gpui_kit::base::input::{InputEvent, InputState};
use gpui_kit::base::{InputBase, input::InputEditorStyle};
use gpui_kit::{
    AccessibleAction, App, ElementId, Entity, EntityInputHandler as _, Focusable as _,
    InteractiveElement as _, IntoElement, MouseButton, ParentElement as _, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled, Window, prelude::FluentBuilder as _,
};

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
            .child(self.state);
        super::theme::apply_style(root, &self.style)
    }
}
