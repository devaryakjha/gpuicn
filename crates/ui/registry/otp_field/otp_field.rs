//! Nova OTP cells using GPUI Kit's digit entry, plus pasting a code.

use super::theme::{UiTheme, primitives::disabled};
use gpui_kit::base::OtpInput;
use gpui_kit::base::TestSupportExt as _;
use gpui_kit::base::input::Paste;
pub use gpui_kit::base::{OtpEvent, OtpState};
use gpui_kit::{
    App, Entity, Focusable as _, InteractiveElement as _, IntoElement, MouseButton,
    ParentElement as _, RenderOnce, Role, SharedString, StatefulInteractiveElement as _, Styled,
    Window, div, prelude::FluentBuilder as _, px,
};

/// Fills `state` from pasted `text`, as Cmd/Ctrl+V and the Edit menu's
/// Paste do in an [`OtpField`].
///
/// Only digits count, so "123 456" and "123-456" paste the same code. Enough
/// digits for every cell replace the value; fewer continue it. The state
/// emits [`OtpEvent::Change`], then [`OtpEvent::Complete`] when the last cell
/// fills. Returns whether the value changed.
pub fn paste_code(state: &Entity<OtpState>, text: &str, window: &mut Window, cx: &mut App) -> bool {
    let current = state.read(cx);
    let Some(value) = pasted_value(current.value(), text, current.len()) else {
        return false;
    };
    let length = current.len();
    state.update(cx, |state, cx| {
        state.set_value(value.clone(), window, cx);
        cx.emit(OtpEvent::Change);
        if value.chars().count() == length {
            cx.emit(OtpEvent::Complete);
        }
    });
    true
}

/// The value after pasting `text` into a code of `length` cells holding `current`.
fn pasted_value(current: &str, text: &str, length: usize) -> Option<String> {
    let digits: Vec<char> = text
        .chars()
        .filter_map(|ch| {
            ch.to_digit(10)
                .or_else(|| (ch as u32).checked_sub('０' as u32).filter(|d| *d < 10))
                .and_then(|digit| char::from_digit(digit, 10))
        })
        .collect();
    let mut value: Vec<char> = if digits.len() >= length {
        Vec::new()
    } else {
        current.chars().collect()
    };
    let room = length.saturating_sub(value.len());
    if digits.is_empty() || room == 0 {
        return None;
    }
    value.extend(digits.into_iter().take(room));
    Some(value.into_iter().collect())
}

#[derive(IntoElement)]
/// A Nova presentation of a retained Kit one-time-code editor.
pub struct OtpField {
    state: Entity<OtpState>,
    style: gpui_kit::StyleRefinement,
    label: Option<SharedString>,
    disabled: bool,
    read_only: bool,
}
impl OtpField {
    /// Creates a Nova code editor using the caller's retained Kit state.
    pub fn new(state: &Entity<OtpState>) -> Self {
        Self {
            state: state.clone(),
            style: Default::default(),
            label: None,
            disabled: false,
            read_only: false,
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
            .test_support()
            .role(Role::TextInput)
            .track_focus(&focus)
            .aria_label(self.label.unwrap_or_else(|| {
                super::theme::locale::text(cx, super::theme::UiText::OneTimeCode)
            }))
            .aria_value(value)
            .when(self.read_only, |root| {
                root.aria_description(super::theme::locale::text(
                    cx,
                    super::theme::UiText::ReadOnly,
                ))
            })
            .key_context(super::theme::OTP_CONTEXT)
            .when(!self.disabled && !self.read_only, |root| {
                let state = self.state.clone();
                root.on_action(move |_: &Paste, window, cx| {
                    if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                        paste_code(&state, &text, window, cx);
                    }
                })
            })
            .when(self.disabled, disabled::style)
            .when(!self.disabled, |root| {
                root.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    focus.focus(window, cx)
                })
            })
            .child(editor);
        super::theme::apply_style(root, &self.style)
    }
}

#[cfg(test)]
mod tests {
    use super::pasted_value;

    #[test]
    fn pasting_keeps_digits_and_fills_or_continues_the_code() {
        assert_eq!(pasted_value("", "123 456", 6).as_deref(), Some("123456"));
        assert_eq!(
            pasted_value("", "123-456-789", 6).as_deref(),
            Some("123456")
        );
        assert_eq!(pasted_value("99", "１２3", 6).as_deref(), Some("99123"));
        assert_eq!(
            pasted_value("123456", "654321", 6).as_deref(),
            Some("654321")
        );
        assert_eq!(pasted_value("123456", "7", 6), None);
        assert_eq!(pasted_value("", "code", 6), None);
    }
}
