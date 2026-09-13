//! Nova presentation for GPUI Kit's numeric input and step actions.

use super::{field::FieldControl, input::Input, theme::UiTheme};
use gpui_icons::{LucideIcon, lucide};
use gpui_kit::base::{NumberInput, input::InputState};
pub use gpui_kit::base::{NumberInputEvent, NumberStep, StepAction};
use gpui_kit::{
    AnyElement, App, Entity, FocusHandle, Focusable as _, IntoElement, ParentElement as _,
    RenderOnce, SharedString, Styled, Window, prelude::FluentBuilder as _,
};

#[derive(IntoElement)]
/// A Kit numeric editor with Nova stepper controls.
pub struct NumberField {
    state: Entity<InputState>,
    style: gpui_kit::StyleRefinement,
    label: Option<SharedString>,
    disabled: bool,
    read_only: bool,
    invalid: bool,
}
impl NumberField {
    /// Configure range and step on the caller-owned `InputState`.
    pub fn new(state: &Entity<InputState>) -> Self {
        Self {
            state: state.clone(),
            style: Default::default(),
            label: None,
            disabled: false,
            read_only: false,
            invalid: false,
        }
    }
    /// Sets the accessible name of the control.
    pub fn aria_label(mut self, value: impl Into<SharedString>) -> Self {
        self.label = Some(value.into());
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
}
impl Styled for NumberField {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}
impl RenderOnce for NumberField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = UiTheme::read(cx).clone();
        let focused = self.state.focus_handle(cx).is_focused(window) && !self.disabled;
        let mut input = Input::new(&self.state)
            .bordered(false)
            .disabled(self.disabled)
            .read_only(self.read_only)
            .invalid(self.invalid);
        if let Some(label) = self.label {
            input = input.aria_label(label);
        }
        let mut root = NumberInput::new(&self.state)
            .disabled(self.disabled)
            .w_full()
            .h(t.space(8.))
            .rounded(t.radius.lg)
            .border_1()
            .border_color(if self.invalid {
                t.colors.destructive
            } else {
                t.colors.input
            })
            .when(focused, |root| {
                root.border_color(if self.invalid {
                    t.colors.destructive
                } else {
                    t.colors.ring
                })
            })
            .when(self.disabled, |root| root.opacity(0.5))
            .decrement_button({
                let t = t.clone();
                move |button| {
                    button.size(t.space(8.)).cursor_pointer().child(
                        lucide(LucideIcon::Minus)
                            .size(t.space(3.5))
                            .text_color(t.colors.foreground),
                    )
                }
            })
            .increment_button({
                let t = t.clone();
                move |button| {
                    button.size(t.space(8.)).cursor_pointer().child(
                        lucide(LucideIcon::Plus)
                            .size(t.space(3.5))
                            .text_color(t.colors.foreground),
                    )
                }
            })
            .input(input);
        if self.read_only {
            root = root.on_step(|_, _, _| {});
        }
        super::theme::apply_style(root, &self.style)
    }
}

impl FieldControl for NumberField {
    fn field_focus_handle(&self, cx: &App) -> FocusHandle {
        self.state.read(cx).focus_handle(cx)
    }

    fn into_field_control(self, label: SharedString, disabled: bool, invalid: bool) -> AnyElement {
        self.aria_label(label)
            .disabled(disabled)
            .invalid(invalid)
            .into_any_element()
    }
}
