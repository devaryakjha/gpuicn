//! Nova tooltips for pointer hover and keyboard focus.
use super::theme::UiTheme;
use gpui_kit::base::{Button, ElementExt as _, Tooltip, TooltipOverlay, TooltipRequest};
use gpui_kit::{
    App, AppContext as _, Bounds, Context, ElementId, Entity, FocusHandle, InteractiveElement as _,
    ParentElement as _, Pixels, SharedString, StatefulInteractiveElement as _, Styled,
    Subscription, Window, div, px,
};

/// Adds a tooltip to one existing button, preserving a single focus target.
/// The button must retain its own accessible name.
pub fn tooltip(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    trigger: Button,
    window: &mut Window,
    cx: &mut App,
) -> gpui_kit::Div {
    let state = window.use_keyed_state(id.into(), cx, TooltipState::new);
    state.update(cx, |state, _| state.label = label.into());
    let focus = state.read(cx).focus.clone();
    let overlay = state.read(cx).overlay.clone();
    let hover = state.clone();
    let measured = state.clone();
    div()
        .child(
            trigger
                .track_focus(&focus)
                .on_prepaint(move |bounds, _, cx| {
                    measured.update(cx, |state, _| state.bounds = bounds)
                })
                .on_hover(move |hovered, window, cx| {
                    hover.update(cx, |state, cx| {
                        state.hovered = *hovered;
                        state.update_visibility(window, cx);
                    })
                }),
        )
        .child(overlay)
}

struct TooltipState {
    focus: FocusHandle,
    overlay: Entity<TooltipOverlay>,
    bounds: Bounds<Pixels>,
    label: SharedString,
    hovered: bool,
    active: bool,
    dismissed: bool,
    _subscriptions: [Subscription; 3],
}
impl TooltipState {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus = cx.focus_handle();
        let owner = window.window_handle();
        let state = cx.weak_entity();
        let subscriptions = [
            cx.on_focus(&focus, window, |state, window, cx| {
                state.update_visibility(window, cx)
            }),
            cx.on_blur(&focus, window, |state, window, cx| {
                state.update_visibility(window, cx)
            }),
            cx.intercept_keystrokes(move |event, window, cx| {
                if window.window_handle() == owner && event.keystroke.key == "escape" {
                    state
                        .update(cx, |state, cx| {
                            if state.active {
                                state.overlay.update(cx, |overlay, cx| overlay.hide(cx));
                                state.active = false;
                                state.dismissed = true;
                                cx.stop_propagation();
                            }
                        })
                        .ok();
                }
            }),
        ];
        Self {
            focus,
            overlay: cx.new(|_| TooltipOverlay::new()),
            bounds: Bounds::default(),
            label: SharedString::default(),
            hovered: false,
            active: false,
            dismissed: false,
            _subscriptions: subscriptions,
        }
    }
    fn update_visibility(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let active = self.hovered || self.focus.is_focused(window);
        if !active {
            self.dismissed = false;
        }
        let active = active && !self.dismissed;
        if active == self.active {
            return;
        }
        self.active = active;
        if active {
            let label = self.label.clone();
            let request =
                TooltipRequest::new(self.bounds, move |_, cx| text_tooltip(label.clone(), cx));
            self.overlay
                .update(cx, |overlay, cx| overlay.request_show(request, window, cx));
        } else {
            self.overlay.update(cx, |overlay, cx| overlay.hide(cx));
        }
    }
}

/// Creates a tooltip view for GPUI's native `.tooltip(...)` attachment point.
/// Use this on an existing interactive control to avoid nesting another button
/// and tab stop. The host control must retain its own full accessible name.
pub fn text_tooltip(label: gpui_kit::SharedString, cx: &mut App) -> gpui_kit::AnyView {
    use gpui_kit::AppContext as _;
    cx.new(|_| TextTooltip(label)).into()
}
struct TextTooltip(gpui_kit::SharedString);
impl gpui_kit::Render for TextTooltip {
    fn render(
        &mut self,
        _: &mut gpui_kit::Window,
        cx: &mut gpui_kit::Context<Self>,
    ) -> impl gpui_kit::IntoElement {
        use gpui_kit::ParentElement as _;
        let theme = UiTheme::read(cx);
        let spacing = theme.spacing.unit;
        let text_scale = theme.text_scale;
        Tooltip::new("tooltip-content")
            .rounded(theme.radius.sm)
            .px(spacing * 3_f32)
            .py(spacing * 1.5_f32)
            .bg(theme.colors.foreground)
            .text_color(theme.colors.background)
            .font_family(theme.fonts.body.clone())
            .text_size(px(12.) * text_scale)
            .child(
                div()
                    .debug_selector(|| "tooltip-label".into())
                    .child(self.0.clone()),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{IntoElement, Modifiers, Render, TestAppContext};
    use std::time::Duration;

    #[gpui_kit::test]
    fn focus_and_hover_show_tooltip_and_escape_dismisses_it(cx: &mut TestAppContext) {
        struct Probe;
        impl Render for Probe {
            fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                div()
                    .child(tooltip(
                        "test-tooltip",
                        "Help",
                        Button::new("help").size(px(100.)).child("Help"),
                        window,
                        cx,
                    ))
                    .child(Button::new("after").child("After"))
            }
        }
        cx.update(crate::init);
        let (_, visual) = cx.add_window_view(|_, _| Probe);
        visual.update(|window, cx| {
            window.activate_window();
            window.draw(cx).clear(cx);
        });
        visual.simulate_mouse_move(
            gpui_kit::point(px(200.), px(200.)),
            None,
            Modifiers::default(),
        );
        visual.simulate_keystrokes("tab");
        visual.executor().advance_clock(Duration::from_millis(550));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(visual.debug_bounds("tooltip-label").is_some());
        visual.simulate_keystrokes("escape");
        assert!(visual.debug_bounds("tooltip-label").is_none());
        visual.simulate_keystrokes("tab shift-tab");
        visual.executor().advance_clock(Duration::from_millis(550));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(visual.debug_bounds("tooltip-label").is_some());
        visual.simulate_keystrokes("tab");
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(visual.debug_bounds("tooltip-label").is_none());
        // Hover the first control while keyboard focus stays on the second.
        visual.simulate_mouse_move(
            gpui_kit::point(px(10.), px(10.)),
            None,
            Modifiers::default(),
        );
        visual.executor().advance_clock(Duration::from_millis(550));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(visual.debug_bounds("tooltip-label").is_some());
        visual.simulate_keystrokes("escape");
        assert!(visual.debug_bounds("tooltip-label").is_none());
    }
}
