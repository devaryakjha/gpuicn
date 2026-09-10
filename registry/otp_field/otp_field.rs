//! Nova OTP cells using GPUI Kit's digit entry and paste handling.

use super::theme::UiTheme;
use gpui_kit::base::OtpInput;
pub use gpui_kit::base::{OtpEvent, OtpState};
use gpui_kit::{
    App, Entity, Focusable as _, InteractiveElement as _, IntoElement, MouseButton,
    ParentElement as _, RenderOnce, Role, SharedString, StatefulInteractiveElement as _, Styled,
    Window, div, prelude::FluentBuilder as _, px,
};

#[derive(IntoElement)]
/// A Nova presentation of a retained Kit one-time-code editor.
pub struct OtpField {
    state: Entity<OtpState>,
    style: gpui_kit::StyleRefinement,
    label: SharedString,
    disabled: bool,
    read_only: bool,
}
impl OtpField {
    /// Creates a Nova code editor using the caller's retained Kit state.
    pub fn new(state: &Entity<OtpState>) -> Self {
        Self {
            state: state.clone(),
            style: Default::default(),
            label: "One-time code".into(),
            disabled: false,
            read_only: false,
        }
    }
    /// Sets the accessible name of the control.
    pub fn aria_label(mut self, value: impl Into<SharedString>) -> Self {
        self.label = value.into();
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
}
impl Styled for OtpField {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}
impl RenderOnce for OtpField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = UiTheme::read(cx).clone();
        let state = self.state.read(cx);
        let chars: Vec<_> = state.value().chars().collect();
        let length = state.len();
        let masked = state.is_masked();
        let focus = state.focus_handle(cx);
        let focused = focus.is_focused(window) && !self.disabled;
        let cursor = focused && state.cursor_visible(cx) && chars.len() < length;
        let value = if masked {
            "•".repeat(chars.len())
        } else {
            state.value().to_string()
        };
        let cells = (0..length).map(|index| {
            div()
                .size(t.space(8.))
                .flex()
                .items_center()
                .justify_center()
                .border_1()
                .border_color(t.colors.input)
                .rounded(t.radius.md)
                .font_family(t.fonts.mono.clone())
                .text_size(t.text(14.))
                .text_color(t.colors.foreground)
                .when(
                    focused && index == chars.len().min(length.saturating_sub(1)),
                    |cell| cell.border_color(t.colors.ring),
                )
                .when_some(chars.get(index), |cell, ch| {
                    cell.child(if masked { "•".into() } else { ch.to_string() })
                })
                .when(cursor && index == chars.len(), |cell| {
                    cell.child(div().w(px(1.)).h(t.text(16.)).bg(t.colors.foreground))
                })
        });
        let editor = OtpInput::new(&self.state)
            .disabled(self.disabled || self.read_only)
            .flex()
            .gap(t.space(1.))
            .children(cells);
        let root = div()
            .id(("otp", self.state.entity_id()))
            .role(Role::TextInput)
            .track_focus(&focus)
            .aria_label(self.label)
            .aria_value(value)
            .when(self.read_only, |root| root.aria_description("Read only"))
            .when(self.disabled, |root| root.opacity(0.5).cursor_not_allowed())
            .when(!self.disabled, |root| {
                root.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    focus.focus(window, cx)
                })
            })
            .child(editor);
        super::theme::apply_style(root, &self.style)
    }
}
