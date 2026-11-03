//! Nova tooltips for pointer hover and keyboard focus.
use super::theme::{UiTheme, primitives::surface};
use gpui_kit::base::{Button, ElementExt as _, Tooltip, TooltipOverlay, TooltipRequest};
use gpui_kit::{
    AnyWindowHandle, App, AppContext as _, Bounds, Context, ElementId, Entity, FocusHandle, Global,
    InteractiveElement as _, ParentElement as _, Pixels, SharedString,
    StatefulInteractiveElement as _, Subscription, WeakEntity, Window, div,
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
    _subscriptions: [Subscription; 2],
}

/// One Escape interceptor for every tooltip in the application, instead of
/// one keystroke hook per tooltip. Active tooltips register here while shown.
#[derive(Default)]
struct EscapeDismissal {
    shown: Vec<(AnyWindowHandle, WeakEntity<TooltipState>)>,
    _subscription: Option<Subscription>,
}
impl Global for EscapeDismissal {}
impl EscapeDismissal {
    fn install(cx: &mut App) {
        if cx.default_global::<Self>()._subscription.is_some() {
            return;
        }
        let subscription = cx.intercept_keystrokes(|event, window, cx| {
            if event.keystroke.key != "escape" {
                return;
            }
            let handle = window.window_handle();
            let shown = std::mem::take(&mut cx.global_mut::<Self>().shown);
            let mut dismissed = false;
            for (owner, state) in shown {
                if owner != handle {
                    cx.global_mut::<Self>().shown.push((owner, state));
                    continue;
                }
                state
                    .update(cx, |state, cx| {
                        if state.active {
                            state.overlay.update(cx, |overlay, cx| overlay.hide(cx));
                            state.active = false;
                            state.dismissed = true;
                            dismissed = true;
                        }
                    })
                    .ok();
            }
            if dismissed {
                cx.stop_propagation();
            }
        });
        cx.global_mut::<Self>()._subscription = Some(subscription);
    }
    fn shown(window: &Window, state: WeakEntity<TooltipState>, cx: &mut App) {
        let handle = window.window_handle();
        let shown = &mut cx.global_mut::<Self>().shown;
        shown.retain(|(_, other)| {
            other.upgrade().is_some() && other.entity_id() != state.entity_id()
        });
        shown.push((handle, state));
    }
    fn hidden(state: &WeakEntity<TooltipState>, cx: &mut App) {
        cx.global_mut::<Self>()
            .shown
            .retain(|(_, other)| other.entity_id() != state.entity_id());
    }
}

impl TooltipState {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        EscapeDismissal::install(cx);
        let focus = cx.focus_handle();
        let subscriptions = [
            cx.on_focus(&focus, window, |state, window, cx| {
                state.update_visibility(window, cx)
            }),
            cx.on_blur(&focus, window, |state, window, cx| {
                state.update_visibility(window, cx)
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
            EscapeDismissal::shown(window, cx.weak_entity(), cx);
        } else {
            self.overlay.update(cx, |overlay, cx| overlay.hide(cx));
            EscapeDismissal::hidden(&cx.weak_entity(), cx);
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
        surface::tooltip(Tooltip::new("tooltip-content"), theme).child(
            div()
                .debug_selector(|| "tooltip-label".into())
                .child(self.0.clone()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{IntoElement, Modifiers, Render, Styled, TestAppContext, px};
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
        cx.update(super::super::theme::init);
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
