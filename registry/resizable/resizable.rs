//! Controlled two-pane layouts. Nest groups for more than two panes.
//!
//! Sizes belong to the application; this component only measures, constrains and
//! emits resize intents. Panes keep their minimum size in small windows and the
//! group becomes scrollable. Pass retained child entities for expensive content.

use super::theme::UiTheme;
use gpui_kit::{
    AccessibleAction, AnyElement, App, Bounds, DispatchPhase, ElementId, FocusHandle,
    InteractiveElement as _, IntoElement, MouseButton, MouseMoveEvent, MouseUpEvent, Orientation,
    ParentElement as _, Pixels, RenderOnce, Role, SharedString, StatefulInteractiveElement as _,
    Styled, Window, accesskit::ActionData, canvas, div, prelude::FluentBuilder as _, px,
};
use std::{rc::Rc, sync::Arc};

const HANDLE: f32 = 8.;
type ResizeHandler = Rc<dyn Fn(&Pixels, &mut Window, &mut App)>;

/// Limits for a pane along the resize axis, in logical pixels.
#[derive(Clone, Copy, Debug)]
pub struct PaneLimits {
    min: f32,
    max: f32,
}
impl PaneLimits {
    /// Creates finite, nonnegative limits. Invalid programmer-supplied limits panic.
    pub fn new(min: Pixels, max: Pixels) -> Self {
        let (min, max) = (f32::from(min), f32::from(max));
        assert!(min.is_finite() && max.is_finite() && min >= 0. && max >= min);
        Self { min, max }
    }
}
impl Default for PaneLimits {
    fn default() -> Self {
        Self {
            min: 80.,
            max: f32::MAX / 4.,
        }
    }
}

struct State {
    bounds: Bounds<Pixels>,
    focus: FocusHandle,
    drag: Option<(f32, f32)>,
}

/// A controlled split with two children and a keyboard-accessible resize handle.
#[derive(IntoElement)]
pub struct Resizable {
    style: gpui_kit::StyleRefinement,
    id: ElementId,
    label: SharedString,
    axis: Orientation,
    size: f32,
    first_limits: PaneLimits,
    second_limits: PaneLimits,
    first: AnyElement,
    second: AnyElement,
    on_resize: ResizeHandler,
}
impl Resizable {
    /// Creates a horizontal (left/right) split. `size` is the first pane's preferred size.
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        size: Pixels,
        first: impl IntoElement,
        second: impl IntoElement,
        on_resize: impl Fn(&Pixels, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            style: gpui_kit::StyleRefinement::default(),
            id: id.into(),
            label: label.into(),
            axis: Orientation::Horizontal,
            size: f32::from(size),
            first_limits: PaneLimits::default(),
            second_limits: PaneLimits::default(),
            first: first.into_any_element(),
            second: second.into_any_element(),
            on_resize: Rc::new(on_resize),
        }
    }
    /// Uses a top/bottom split. Arrow Up/Down resize its horizontal separator.
    pub fn vertical(mut self) -> Self {
        self.axis = Orientation::Vertical;
        self
    }
    /// Sets minimum/maximum sizes for the first pane.
    pub fn first_limits(mut self, limits: PaneLimits) -> Self {
        self.first_limits = limits;
        self
    }
    /// Sets minimum/maximum sizes for the second pane.
    pub fn second_limits(mut self, limits: PaneLimits) -> Self {
        self.second_limits = limits;
        self
    }
}

// When minima do not fit, scroll the group rather than shrinking either pane to zero.
fn geometry(
    extent: f32,
    preferred: f32,
    first: PaneLimits,
    second: PaneLimits,
) -> (f32, f32, f32, f32) {
    let available = (extent - HANDLE)
        .max(first.min + second.min)
        .min(first.max + second.max);
    let low = first.min.max(available - second.max);
    let high = first.max.min(available - second.min).max(low);
    let size = if preferred.is_finite() {
        preferred.clamp(low, high)
    } else {
        low
    };
    (size, available - size, low, high)
}
fn coordinate(axis: Orientation, point: gpui_kit::Point<Pixels>) -> f32 {
    f32::from(if axis == Orientation::Horizontal {
        point.x
    } else {
        point.y
    })
}
fn extent(axis: Orientation, bounds: Bounds<Pixels>) -> f32 {
    f32::from(if axis == Orientation::Horizontal {
        bounds.size.width
    } else {
        bounds.size.height
    })
}

impl RenderOnce for Resizable {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let axis = self.axis;
        let key = ElementId::NamedChild(Arc::new(self.id.clone()), "resize-state".into());
        let state = window.use_keyed_state(key, cx, |_, cx| State {
            bounds: Bounds::default(),
            focus: cx.focus_handle(),
            drag: None,
        });
        let theme = UiTheme::read(cx).clone();
        let first_limits = self.first_limits;
        let second_limits = self.second_limits;
        let (first_size, second_size, low, high) = geometry(
            extent(axis, state.read(cx).bounds),
            self.size,
            first_limits,
            second_limits,
        );
        let focus = state.read(cx).focus.clone();
        let movable = high > low;
        let down_state = state.clone();
        let key_state = state.clone();
        let measure_state = state.clone();
        let move_state = state.clone();
        let resize = self.on_resize.clone();
        let keyboard_resize = self.on_resize.clone();
        let accessibility_resize = self.on_resize;
        let horizontal = axis == Orientation::Horizontal;
        let handle_id = ElementId::NamedChild(Arc::new(self.id.clone()), "handle".into());
        let mut handle = div()
            .id(handle_id)
            .debug_selector(|| "resizable-handle".into())
            .role(Role::Splitter)
            .aria_label(self.label)
            .aria_orientation(if horizontal {
                Orientation::Vertical
            } else {
                Orientation::Horizontal
            })
            .aria_numeric_value(first_size as f64)
            .aria_min_numeric_value(low as f64)
            .aria_max_numeric_value(high as f64)
            .aria_numeric_value_step(8.)
            .track_focus(&focus.tab_stop(movable).tab_index(0))
            .relative()
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .when(horizontal, |el| {
                el.w(px(HANDLE)).h_full().cursor_col_resize()
            })
            .when(!horizontal, |el| {
                el.h(px(HANDLE)).w_full().cursor_row_resize()
            })
            .when(!movable, |el| el.cursor_default())
            .hover(|style| style.bg(theme.colors.sidebar_accent))
            .focus_visible(|style| {
                style
                    .bg(theme.colors.sidebar_accent)
                    .border_1()
                    .border_color(theme.colors.ring)
            })
            .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                if !movable {
                    return;
                }
                down_state.update(cx, |state, cx| {
                    state.focus.focus(window, cx);
                    state.drag = Some((coordinate(axis, event.position), first_size));
                });
                cx.stop_propagation();
            })
            .on_key_down(move |event, window, cx| {
                let step = if event.keystroke.modifiers.shift {
                    32.
                } else {
                    8.
                };
                let key = event.keystroke.key.as_str();
                if key == "escape" {
                    key_state.update(cx, |state, _| state.drag = None);
                    return;
                }
                let next = match key {
                    "left" if horizontal => first_size - step,
                    "right" if horizontal => first_size + step,
                    "up" if !horizontal => first_size - step,
                    "down" if !horizontal => first_size + step,
                    "home" => low,
                    "end" => high,
                    _ => return,
                }
                .clamp(low, high);
                if movable && next != first_size {
                    keyboard_resize(&px(next), window, cx);
                }
                cx.stop_propagation();
            })
            .child(
                div()
                    .absolute()
                    .bg(theme.colors.border)
                    .when(horizontal, |el| el.w(px(1.)).h_full())
                    .when(!horizontal, |el| el.h(px(1.)).w_full()),
            )
            .child(
                div()
                    .bg(theme.colors.border)
                    .rounded(theme.radius.md)
                    .when(horizontal, |el| el.w(theme.space(1.)).h(theme.space(6.)))
                    .when(!horizontal, |el| el.h(theme.space(1.)).w(theme.space(6.))),
            );
        for action in [
            AccessibleAction::Increment,
            AccessibleAction::Decrement,
            AccessibleAction::SetValue,
        ] {
            let resize = accessibility_resize.clone();
            handle = handle.on_a11y_action(action, move |data, window, cx| {
                let value = match (action, data) {
                    (AccessibleAction::Increment, _) => first_size + 8.,
                    (AccessibleAction::Decrement, _) => first_size - 8.,
                    (AccessibleAction::SetValue, Some(ActionData::NumericValue(value)))
                        if value.is_finite() =>
                    {
                        *value as f32
                    }
                    _ => return,
                }
                .clamp(low, high);
                if movable && value != first_size {
                    resize(&px(value), window, cx);
                }
            });
        }
        let panes = div()
            .flex()
            .flex_shrink_0()
            .when(horizontal, |el| {
                el.w(px(first_size + second_size + HANDLE)).h_full()
            })
            .when(!horizontal, |el| {
                el.flex_col()
                    .h(px(first_size + second_size + HANDLE))
                    .w_full()
            })
            .child(
                div()
                    .flex_shrink_0()
                    .min_w_0()
                    .min_h_0()
                    .overflow_hidden()
                    .when(horizontal, |el| el.w(px(first_size)).h_full())
                    .when(!horizontal, |el| el.h(px(first_size)).w_full())
                    .child(self.first),
            )
            .child(handle)
            .child(
                div()
                    .flex_shrink_0()
                    .min_w_0()
                    .min_h_0()
                    .overflow_hidden()
                    .when(horizontal, |el| el.w(px(second_size)).h_full())
                    .when(!horizontal, |el| el.h(px(second_size)).w_full())
                    .child(self.second),
            );
        div()
            .id(self.id)
            .relative()
            .size_full()
            .min_w_0()
            .min_h_0()
            .overflow_scroll()
            .child(
                canvas(
                    move |bounds, _, cx| {
                        measure_state.update(cx, |state, cx| {
                            if state.bounds != bounds {
                                state.bounds = bounds;
                                cx.notify();
                            }
                        });
                    },
                    move |_, _, window, _| {
                        // Capture-phase listeners keep a drag alive over expensive or occluding children.
                        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                            if phase != DispatchPhase::Capture {
                                return;
                            }
                            let Some((start, size)) = move_state.read(cx).drag else {
                                return;
                            };
                            if event.pressed_button != Some(MouseButton::Left) {
                                move_state.update(cx, |state, _| state.drag = None);
                                return;
                            }
                            let (_, _, low, high) = geometry(
                                extent(axis, move_state.read(cx).bounds),
                                size,
                                first_limits,
                                second_limits,
                            );
                            let next =
                                (size + coordinate(axis, event.position) - start).clamp(low, high);
                            resize(&px(next), window, cx);
                            cx.stop_propagation();
                        });
                        window.on_mouse_event(move |event: &MouseUpEvent, phase, _, cx| {
                            if phase == DispatchPhase::Capture && event.button == MouseButton::Left
                            {
                                state.update(cx, |state, _| state.drag = None);
                            }
                        });
                    },
                )
                .absolute()
                .size_full(),
            )
            .child(panes)
            .map(|base| super::theme::apply_style(base, &self.style))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sizes_obey_both_panes_and_small_windows_remain_scrollable() {
        let a = PaneLimits::new(px(160.), px(400.));
        let b = PaneLimits::new(px(240.), px(800.));
        for width in [0., 200., 408., 700., 2000.] {
            for desired in [-100., 200., 10000., f32::NAN] {
                let (first, second, low, high) = geometry(width, desired, a, b);
                assert!((160. ..=400.).contains(&first));
                assert!((240. ..=800.).contains(&second));
                assert!(first >= low && first <= high);
                if width < 408. {
                    assert_eq!(first + second + HANDLE, 408.);
                }
            }
        }
    }
}

#[cfg(test)]
mod interaction_tests {
    use super::*;
    use gpui_kit::{
        AppContext as _, Context, Modifiers, Render, TestAppContext, VisualTestContext, point,
    };
    struct View {
        size: Pixels,
    }
    impl Render for View {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div().w(px(700.)).h(px(300.)).child(
                Resizable::new(
                    "test.split",
                    "Resize test",
                    self.size,
                    div(),
                    div(),
                    cx.listener(|this, value: &Pixels, _, cx| {
                        this.size = *value;
                        cx.notify();
                    }),
                )
                .first_limits(PaneLimits::new(px(160.), px(400.)))
                .second_limits(PaneLimits::new(px(240.), px(800.))),
            )
        }
    }
    #[test]
    fn pointer_and_keyboard_share_bounds_and_release_ends_drag() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| UiTheme::set(cx, UiTheme::neutral_light()));
        let window = cx.add_window(|_, _| View { size: px(200.) });
        for _ in 0..2 {
            cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
                .unwrap();
        }
        let mut visual = VisualTestContext::from_window(window.into(), &cx);
        let handle = visual.debug_bounds("resizable-handle").unwrap().center();
        visual.simulate_mouse_down(handle, MouseButton::Left, Modifiers::default());
        visual.simulate_mouse_move(
            point(handle.x + px(80.), handle.y),
            Some(MouseButton::Left),
            Modifiers::default(),
        );
        visual.simulate_mouse_up(
            point(handle.x + px(80.), handle.y),
            MouseButton::Left,
            Modifiers::default(),
        );
        assert_eq!(
            cx.read_window(&window, |view, cx| view.read(cx).size)
                .unwrap(),
            px(280.)
        );
        visual.simulate_mouse_move(point(px(600.), handle.y), None, Modifiers::default());
        assert_eq!(
            cx.read_window(&window, |view, cx| view.read(cx).size)
                .unwrap(),
            px(280.)
        );
        visual.simulate_keystrokes("right");
        assert_eq!(
            cx.read_window(&window, |view, cx| view.read(cx).size)
                .unwrap(),
            px(288.)
        );
        visual.simulate_keystrokes("home");
        assert_eq!(
            cx.read_window(&window, |view, cx| view.read(cx).size)
                .unwrap(),
            px(160.)
        );
        visual.simulate_keystrokes("end");
        assert_eq!(
            cx.read_window(&window, |view, cx| view.read(cx).size)
                .unwrap(),
            px(400.)
        );
    }
}

impl gpui_kit::Styled for Resizable {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}
