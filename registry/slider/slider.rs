#![allow(missing_docs)]
//! Nova-styled single-value Slider backed by Base GPUI dragging and keyboard behavior.

use std::rc::Rc;

use base_gpui::slider::{
    SliderControl, SliderIndicator, SliderRoot, SliderThumb, SliderTrack, SliderValueChangeDetails,
    SliderValues,
};
use gpui::{
    App, BoxShadow, ElementId, IntoElement, RenderOnce, SharedString, Styled, Window,
    prelude::FluentBuilder as _, px,
};

use super::theme::UiTheme;

type ChangeHandler =
    Rc<dyn Fn(SliderValues, &mut SliderValueChangeDetails, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct Slider {
    id: ElementId,
    default_value: f64,
    value: Option<f64>,
    min: f64,
    max: f64,
    step: f64,
    disabled: bool,
    aria_label: Option<SharedString>,
    on_value_change: Option<ChangeHandler>,
}
impl Slider {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            default_value: 0.,
            value: None,
            min: 0.,
            max: 100.,
            step: 1.,
            disabled: false,
            aria_label: None,
            on_value_change: None,
        }
    }
    pub fn default_value(mut self, value: f64) -> Self {
        self.default_value = value;
        self
    }
    pub fn value(mut self, value: f64) -> Self {
        self.value = Some(value);
        self
    }
    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
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
    pub fn on_value_change(
        mut self,
        handler: impl Fn(SliderValues, &mut SliderValueChangeDetails, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(handler));
        self
    }
}
impl RenderOnce for Slider {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let mut root = SliderRoot::new()
            .id(self.id)
            .default_value(SliderValues::Single(self.default_value))
            .min(self.min)
            .max(self.max)
            .step(self.step)
            .disabled(self.disabled)
            .w_full()
            .h(px(20.))
            .style_with_state(|state, base| {
                base.when(state.disabled, |base| {
                    base.opacity(0.50).cursor_not_allowed()
                })
            });
        if let Some(value) = self.value {
            root = root.value(SliderValues::Single(value));
        }
        if let Some(label) = self.aria_label {
            root = root.aria_label(label);
        }
        if let Some(handler) = self.on_value_change {
            root = root.on_value_change(move |values, details, window, cx| {
                handler(values, details, window, cx)
            });
        }
        root.child(
            SliderControl::new()
                .relative()
                .w_full()
                .h_full()
                .flex()
                .items_center()
                .child(
                    SliderTrack::new()
                        .relative()
                        .w_full()
                        .h(px(4.))
                        .rounded_full()
                        .bg(colors.muted)
                        .child(
                            SliderIndicator::new()
                                .h_full()
                                .rounded_full()
                                .bg(colors.primary),
                        ),
                )
                .child(
                    SliderThumb::new()
                        .size(px(12.))
                        .rounded_full()
                        .border_1()
                        .border_color(colors.ring)
                        .bg(colors.background)
                        .style_with_state(move |state, style| {
                            if state.focused || state.active {
                                style.shadow(vec![
                                    BoxShadow::new(px(0.), px(0.), colors.ring.alpha(0.50).into())
                                        .spread_radius(px(3.)),
                                ])
                            } else {
                                style
                            }
                        }),
                ),
        )
    }
}
