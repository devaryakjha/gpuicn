#![allow(missing_docs)]
//! Nova-styled OTP Field backed by Base GPUI slot and paste behavior.

use std::rc::Rc;

use base_gpui::otp_field::{OTPFieldChangeDetails, OTPFieldInput, OTPFieldRoot};
use gpui::{
    App, ElementId, IntoElement, RenderOnce, SharedString, Styled, Window,
    prelude::FluentBuilder as _, px,
};

use super::theme::{ThemeMode, UiTheme, focus_outline};

type ChangeHandler =
    Rc<dyn Fn(SharedString, OTPFieldChangeDetails, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct OtpField {
    style: gpui::StyleRefinement,
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
            style: gpui::StyleRefinement::default(),
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
        let spacing = theme.spacing.unit;
        let text_scale = theme.text_scale;
        let colors = theme.colors;
        let mode = theme.mode;
        let focus_ring = theme.focus_ring();
        let invalid_focus_ring = theme.destructive_focus_ring();
        let radius = theme.radius.lg;
        let root = OTPFieldRoot::new()
            .id(self.id)
            .length(self.length)
            .default_value(self.default_value)
            .mask(self.mask)
            .disabled(self.disabled)
            .read_only(self.read_only)
            .flex()
            .gap(px(0.));
        let mut root = super::theme::apply_style(root, &self.style);
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
            let focus_ring = focus_ring.clone();
            let invalid_focus_ring = invalid_focus_ring.clone();
            OTPFieldInput::new()
                .with_slot_index(index)
                .style_with_state(move |state, base| {
                    let focus_ring = if state.root.invalid {
                        invalid_focus_ring.clone()
                    } else {
                        focus_ring.clone()
                    };
                    base.flex()
                        .items_center()
                        .justify_center()
                        .size(spacing * 8_f32)
                        .border_t_1()
                        .border_b_1()
                        .border_r_1()
                        .when(index == 0, |base| base.border_l_1())
                        .border_color(if state.root.invalid {
                            colors.destructive
                        } else if state.active && state.root.focused {
                            colors.ring
                        } else {
                            colors.input
                        })
                        .bg(if mode == ThemeMode::Dark {
                            colors.background.blend(colors.input.opacity(0.30))
                        } else {
                            colors.background
                        })
                        .text_color(colors.foreground)
                        .text_size(px(14.) * text_scale)
                        .when(index == 0, |base| base.rounded_l(radius))
                        .when(index + 1 == state.root.length, |base| {
                            base.rounded_r(radius)
                        })
                        .when(
                            state.active && state.root.focused && !state.root.disabled,
                            move |base| {
                                focus_outline(
                                    base,
                                    focus_ring[0].color.into(),
                                    gpui::Corners {
                                        top_left: if index == 0 { radius } else { px(0.) },
                                        bottom_left: if index == 0 { radius } else { px(0.) },
                                        top_right: if index + 1 == state.root.length {
                                            radius
                                        } else {
                                            px(0.)
                                        },
                                        bottom_right: if index + 1 == state.root.length {
                                            radius
                                        } else {
                                            px(0.)
                                        },
                                    },
                                )
                            },
                        )
                        .when(state.root.disabled, |base| {
                            base.opacity(0.50).cursor_not_allowed()
                        })
                })
        }))
    }
}

impl gpui::Styled for OtpField {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
