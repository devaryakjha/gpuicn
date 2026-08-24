#![allow(missing_docs)]
//! Nova-styled OTP Field backed by Base GPUI slot and paste behavior.

use std::rc::Rc;

use base_gpui::otp_field::{OTPFieldChangeDetails, OTPFieldInput, OTPFieldRoot};
use gpui::{
    App, BoxShadow, ElementId, IntoElement, RenderOnce, SharedString, Styled, Window,
    prelude::FluentBuilder as _, px,
};

use super::theme::{ThemeMode, UiTheme};

type ChangeHandler =
    Rc<dyn Fn(SharedString, OTPFieldChangeDetails, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct OtpField {
    id: ElementId,
    length: usize,
    default_value: SharedString,
    value: Option<SharedString>,
    mask: bool,
    disabled: bool,
    read_only: bool,
    aria_label: Option<SharedString>,
    on_value_change: Option<ChangeHandler>,
    on_value_complete: Option<ChangeHandler>,
}
impl OtpField {
    pub fn new(id: impl Into<ElementId>, length: usize) -> Self {
        Self {
            id: id.into(),
            length,
            default_value: SharedString::default(),
            value: None,
            mask: false,
            disabled: false,
            read_only: false,
            aria_label: None,
            on_value_change: None,
            on_value_complete: None,
        }
    }
    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default_value = value.into();
        self
    }
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }
    pub fn mask(mut self, value: bool) -> Self {
        self.mask = value;
        self
    }
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    pub fn read_only(mut self, value: bool) -> Self {
        self.read_only = value;
        self
    }
    pub fn aria_label(mut self, value: impl Into<SharedString>) -> Self {
        self.aria_label = Some(value.into());
        self
    }
    pub fn on_value_change(
        mut self,
        handler: impl Fn(SharedString, OTPFieldChangeDetails, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(handler));
        self
    }
    pub fn on_value_complete(
        mut self,
        handler: impl Fn(SharedString, OTPFieldChangeDetails, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_complete = Some(Rc::new(handler));
        self
    }
}
impl RenderOnce for OtpField {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let mode = theme.mode;
        let mut root = OTPFieldRoot::new()
            .id(self.id)
            .length(self.length)
            .default_value(self.default_value)
            .mask(self.mask)
            .disabled(self.disabled)
            .read_only(self.read_only)
            .flex()
            .gap(px(0.));
        if let Some(value) = self.value {
            root = root.value(value);
        }
        if let Some(label) = self.aria_label {
            root = root.aria_label(label);
        }
        if let Some(handler) = self.on_value_change {
            root = root.on_value_change(move |value, details, window, cx| {
                handler(value, details, window, cx)
            });
        }
        if let Some(handler) = self.on_value_complete {
            root = root.on_value_complete(move |value, details, window, cx| {
                handler(value, details, window, cx)
            });
        }
        root.children((0..self.length).map(move |index| {
            OTPFieldInput::new()
                .with_slot_index(index)
                .style_with_state(move |state, base| {
                    base.flex()
                        .items_center()
                        .justify_center()
                        .size(px(32.))
                        .border_t_1()
                        .border_b_1()
                        .border_r_1()
                        .when(index == 0, |base| base.border_l_1())
                        .border_color(if state.root.invalid {
                            colors.destructive
                        } else if state.active {
                            colors.ring
                        } else {
                            colors.input
                        })
                        .bg(if mode == ThemeMode::Dark {
                            colors.background.blend(colors.input.alpha(0.30))
                        } else {
                            colors.background
                        })
                        .text_color(colors.foreground)
                        .text_size(px(14.))
                        .when(index == 0, |base| base.rounded_l(px(10.)))
                        .when(index + 1 == state.root.length, |base| {
                            base.rounded_r(px(10.))
                        })
                        .when(state.active, move |base| {
                            base.shadow(vec![
                                BoxShadow::new(px(0.), px(0.), colors.ring.alpha(0.50).into())
                                    .spread_radius(px(3.)),
                            ])
                        })
                        .when(state.root.disabled, |base| base.opacity(0.50))
                })
        }))
    }
}
