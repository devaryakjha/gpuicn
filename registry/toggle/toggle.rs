#![allow(missing_docs)]
//! Nova-styled Toggle backed by Base GPUI pressed-state behavior.

use std::rc::Rc;

use base_gpui::toggle::{Toggle as BaseToggle, TogglePressedChangeDetails};
use gpui::{
    AnyElement, App, BoxShadow, ElementId, InteractiveElement as _, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, Window, prelude::FluentBuilder as _, px,
};

use super::theme::UiTheme;

type ChangeHandler =
    Rc<dyn Fn(bool, &mut TogglePressedChangeDetails, &mut Window, &mut App) + 'static>;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToggleVariant {
    #[default]
    Default,
    Outline,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToggleSize {
    Sm,
    #[default]
    Default,
    Lg,
}

#[derive(IntoElement)]
pub struct Toggle {
    id: ElementId,
    default_pressed: bool,
    pressed: Option<bool>,
    disabled: bool,
    aria_label: Option<SharedString>,
    variant: ToggleVariant,
    size: ToggleSize,
    children: Vec<AnyElement>,
    on_pressed_change: Option<ChangeHandler>,
}

impl Toggle {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            default_pressed: false,
            pressed: None,
            disabled: false,
            aria_label: None,
            variant: ToggleVariant::Default,
            size: ToggleSize::Default,
            children: Vec::new(),
            on_pressed_change: None,
        }
    }
    pub fn default_pressed(mut self, pressed: bool) -> Self {
        self.default_pressed = pressed;
        self
    }
    pub fn pressed(mut self, pressed: bool) -> Self {
        self.pressed = Some(pressed);
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }
    pub fn variant(mut self, variant: ToggleVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn size(mut self, size: ToggleSize) -> Self {
        self.size = size;
        self
    }
    pub fn on_pressed_change(
        mut self,
        handler: impl Fn(bool, &mut TogglePressedChangeDetails, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_pressed_change = Some(Rc::new(handler));
        self
    }
}

impl ParentElement for Toggle {
    fn extend(&mut self, children: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(children);
    }
}

impl RenderOnce for Toggle {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let variant = self.variant;
        let (height, radius, text_size) = match self.size {
            ToggleSize::Sm => (28., 8., 12.8),
            ToggleSize::Default => (32., 10., 14.),
            ToggleSize::Lg => (36., 10., 14.),
        };
        let mut toggle: BaseToggle<SharedString> = BaseToggle::new()
            .id(self.id)
            .default_pressed(self.default_pressed)
            .disabled(self.disabled)
            .style_with_state(move |state, base| {
                let base = base
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap(px(4.))
                    .min_w(px(height))
                    .h(px(height))
                    .px(px(10.))
                    .rounded(px(radius))
                    .text_size(px(text_size))
                    .text_color(colors.muted_foreground)
                    .bg(if state.pressed {
                        colors.muted
                    } else {
                        colors.background.alpha(0.)
                    })
                    .focus_visible(move |style| {
                        style.border_color(colors.ring).shadow(vec![
                            BoxShadow::new(px(0.), px(0.), colors.ring.alpha(0.50).into())
                                .spread_radius(px(3.)),
                        ])
                    })
                    .when(state.disabled, |base| {
                        base.opacity(0.50).cursor_not_allowed()
                    });
                match variant {
                    ToggleVariant::Default => base,
                    ToggleVariant::Outline => base.border_1().border_color(colors.input),
                }
            })
            .children(self.children);
        if let Some(pressed) = self.pressed {
            toggle = toggle.pressed(Some(pressed));
        }
        if let Some(label) = self.aria_label {
            toggle = toggle.aria_label(label);
        }
        if let Some(handler) = self.on_pressed_change {
            toggle = toggle.on_pressed_change(move |pressed, details, window, cx| {
                handler(pressed, details, window, cx)
            });
        }
        toggle
    }
}
