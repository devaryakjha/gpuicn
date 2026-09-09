//! Single-value slider using Kit state and pointer primitives.
use super::theme::{UiTheme, apply_style};
pub use gpui_kit::base::slider::{SliderEvent, SliderState, SliderValue};
use gpui_kit::base::{SliderIndicator, SliderThumb, SliderTrack};
use gpui_kit::{
    AccessibleAction, App, ElementId, Entity, InteractiveElement as _, IntoElement, MouseButton,
    ParentElement as _, RenderOnce, Role, SharedString, StatefulInteractiveElement as _,
    StyleRefinement, Styled, Window, div, prelude::FluentBuilder as _, relative,
};

#[derive(IntoElement)]
/// A Nova presentation of retained Kit slider state.
pub struct Slider {
    state: Entity<SliderState>,
    disabled: bool,
    label: Option<SharedString>,
    style: StyleRefinement,
}
impl Slider {
    /// Creates a Nova slider using the caller's retained Kit state.
    pub fn new(state: &Entity<SliderState>) -> Self {
        Self {
            state: state.clone(),
            disabled: false,
            label: None,
            style: Default::default(),
        }
    }
    /// Disables interaction and applies the disabled appearance.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    /// Sets the accessible name of the control.
    pub fn aria_label(mut self, value: impl Into<SharedString>) -> Self {
        self.label = Some(value.into());
        self
    }
}
impl Styled for Slider {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
impl RenderOnce for Slider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let state = self.state.read(cx);
        assert!(
            state.value().is_single(),
            "Slider expects single-value state"
        );
        let (min, max, step, value) = (
            state.min_value(),
            state.max_value(),
            state.step_value(),
            state.value().end(),
        );
        assert!(
            min.is_finite()
                && max.is_finite()
                && min < max
                && step.is_finite()
                && step > 0.
                && value.is_finite()
        );
        let percent = state.percentage().end;
        let focus = window
            .use_keyed_state(
                ElementId::from(("slider-focus", self.state.entity_id())),
                cx,
                |_, cx| cx.focus_handle(),
            )
            .read(cx)
            .clone()
            .tab_stop(!self.disabled);
        let focused = focus.is_focused(window);
        let click_focus = focus.clone();
        let thumb_focus = focus.clone();
        let input_state = self.state.clone();
        let increment = self.state.clone();
        let decrement = self.state.clone();
        let set_value = self.state.clone();
        let release = self.state.clone();
        let release_out = self.state.clone();
        let thumb = SliderThumb::new(&self.state)
            .disabled(self.disabled)
            .absolute()
            .left(relative(percent))
            .ml(-theme.space(1.5))
            .top(-theme.space(1.))
            .size(theme.space(3.))
            .rounded_full()
            .border_1()
            .border_color(colors.ring)
            .bg(colors.background)
            .when(focused && !self.disabled, |t| t.shadow(theme.focus_ring()))
            .when(!self.disabled, |t| {
                t.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    thumb_focus.focus(window, cx)
                })
            });
        let root = div()
            .id(("slider", self.state.entity_id()))
            .role(Role::Slider)
            .aria_numeric_value(value as f64)
            .aria_min_numeric_value(min as f64)
            .aria_max_numeric_value(max as f64)
            .aria_numeric_value_step(step as f64)
            .aria_orientation(gpui_kit::Orientation::Horizontal)
            .when_some(self.label, |root, label| root.aria_label(label))
            .relative()
            .w_full()
            .h(theme.space(5.))
            .flex()
            .items_center()
            .when(self.disabled, |root| root.opacity(0.5).cursor_not_allowed())
            .when(!self.disabled, |root| {
                root.track_focus(&focus)
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                        click_focus.focus(window, cx)
                    })
                    .on_key_down(move |event, window, cx| {
                        if event.keystroke.modifiers.modified() {
                            return;
                        }
                        let state = input_state.read(cx);
                        let value = match event.keystroke.key.as_str() {
                            "right" | "up" => state.value().end() + state.step_value(),
                            "left" | "down" => state.value().end() - state.step_value(),
                            "pageup" => state.value().end() + state.step_value() * 10.,
                            "pagedown" => state.value().end() - state.step_value() * 10.,
                            "home" => state.min_value(),
                            "end" => state.max_value(),
                            _ => return,
                        };
                        update_value(&input_state, value, window, cx);
                        cx.stop_propagation();
                    })
                    .on_a11y_action(AccessibleAction::Increment, move |_, window, cx| {
                        let state = increment.read(cx);
                        let value = state.value().end() + state.step_value();
                        update_value(&increment, value, window, cx);
                    })
                    .on_a11y_action(AccessibleAction::Decrement, move |_, window, cx| {
                        let state = decrement.read(cx);
                        let value = state.value().end() - state.step_value();
                        update_value(&decrement, value, window, cx);
                    })
                    .on_a11y_action(AccessibleAction::SetValue, move |data, window, cx| {
                        if let Some(gpui_kit::accesskit::ActionData::NumericValue(value)) = data {
                            update_value(&set_value, *value as f32, window, cx);
                        }
                    })
                    .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                        release.update(cx, |state, cx| state.handle_release(cx))
                    })
                    .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                        release_out.update(cx, |state, cx| state.handle_release(cx))
                    })
            })
            .child(
                SliderTrack::new(&self.state)
                    .disabled(self.disabled)
                    .relative()
                    .w_full()
                    .h(theme.space(1.))
                    .child(
                        SliderIndicator::new(&self.state)
                            .relative()
                            .w_full()
                            .h_full()
                            .rounded_full()
                            .bg(colors.muted)
                            .child(
                                div()
                                    .h_full()
                                    .w(relative(percent))
                                    .rounded_full()
                                    .bg(colors.primary),
                            ),
                    )
                    .child(thumb),
            );
        apply_style(root, &self.style)
    }
}
fn update_value(state: &Entity<SliderState>, value: f32, window: &mut Window, cx: &mut App) {
    if !value.is_finite() {
        return;
    }
    state.update(cx, |state, cx| {
        let min = state.min_value();
        let value = (min + ((value - min) / state.step_value()).round() * state.step_value())
            .clamp(min, state.max_value());
        state.set_value(value, window, cx);
        state.handle_release(cx);
    });
}
