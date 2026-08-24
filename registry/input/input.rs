#![allow(missing_docs)]
//! Nova-styled text Input backed by Base GPUI editing and focus behavior.

use std::rc::Rc;

use base_gpui::input::Input as BaseInput;
use gpui::{
    App, BoxShadow, ElementId, InteractiveElement as _, IntoElement, RenderOnce, SharedString,
    Styled, Window, prelude::FluentBuilder as _, px,
};

use super::theme::{ThemeMode, UiTheme};

type ValueChange = Rc<dyn Fn(SharedString) + 'static>;

#[derive(IntoElement)]
pub struct Input {
    id: ElementId,
    value: Option<SharedString>,
    default_value: Option<SharedString>,
    placeholder: Option<SharedString>,
    aria_label: Option<SharedString>,
    disabled: bool,
    read_only: bool,
    required: bool,
    on_value_change: Option<ValueChange>,
}

impl Input {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: None,
            default_value: None,
            placeholder: None,
            aria_label: None,
            disabled: false,
            read_only: false,
            required: false,
            on_value_change: None,
        }
    }
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }
    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default_value = Some(value.into());
        self
    }
    pub fn placeholder(mut self, value: impl Into<SharedString>) -> Self {
        self.placeholder = Some(value.into());
        self
    }
    pub fn aria_label(mut self, value: impl Into<SharedString>) -> Self {
        self.aria_label = Some(value.into());
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
    pub fn required(mut self, value: bool) -> Self {
        self.required = value;
        self
    }
    pub fn on_value_change(mut self, handler: impl Fn(SharedString) + 'static) -> Self {
        self.on_value_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Input {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let background = match theme.mode {
            ThemeMode::Light => colors.background.alpha(0.0),
            ThemeMode::Dark => colors.input.alpha(0.30),
        };
        let mut input = BaseInput::new()
            .id(self.id)
            .disabled(self.disabled)
            .read_only(self.read_only)
            .required(self.required)
            .font_family(theme.fonts.body)
            .style_with_state(move |state, base| {
                base.w_full()
                    .min_w_0()
                    .h(px(32.))
                    .px(px(10.))
                    .py(px(4.))
                    .rounded(px(10.))
                    .border_1()
                    .border_color(if state.invalid {
                        colors.destructive
                    } else {
                        colors.input
                    })
                    .bg(background)
                    .text_color(colors.foreground)
                    .text_size(px(14.))
                    .focus_visible(move |style| {
                        style.border_color(colors.ring).shadow(vec![
                            BoxShadow::new(px(0.), px(0.), colors.ring.alpha(0.50).into())
                                .spread_radius(px(3.)),
                        ])
                    })
                    .when(state.disabled, |base| {
                        base.opacity(0.50).cursor_not_allowed()
                    })
            });
        if let Some(value) = self.value {
            input = input.value(value);
        }
        if let Some(value) = self.default_value {
            input = input.default_value(value);
        }
        if let Some(value) = self.placeholder {
            input = input.placeholder(value);
        }
        if let Some(value) = self.aria_label {
            input = input.aria_label(value);
        }
        if let Some(handler) = self.on_value_change {
            input = input.on_value_change(move |value| handler(value));
        }
        input
    }
}
