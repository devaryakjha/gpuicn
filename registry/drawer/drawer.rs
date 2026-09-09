//! Nova drawers with composable content, retained presence and directional swipes.
use super::{
    button::{Button, ButtonVariant},
    dialog::{dialog_backdrop, dialog_description, dialog_title, modal_focus},
    theme::{UiTheme, apply_style},
};
pub use gpui_kit::base::DialogHandle as DrawerHandle;
use gpui_kit::base::{
    ElementExt as _, Sheet,
    motion::{Easing, Presence, Transition, transition},
};
use gpui_kit::{
    AnyElement, App, Div, ElementId, InteractiveElement as _, IntoElement, MouseButton,
    ParentElement, Pixels, Point, RenderOnce, Role, SharedString, Size, Stateful,
    StatefulInteractiveElement as _, StyleRefinement, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use std::time::Duration;

/// The viewport edge from which a drawer opens and toward which it dismisses.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum DrawerSide {
    /// Attach to the top edge.
    Top,
    /// Attach to the right edge.
    Right,
    /// Attach to the bottom edge.
    #[default]
    Bottom,
    /// Attach to the left edge.
    Left,
}
impl DrawerSide {
    fn vertical(self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }
    fn distance(self, start: Point<Pixels>, end: Point<Pixels>) -> Pixels {
        match self {
            Self::Top => start.y - end.y,
            Self::Bottom => end.y - start.y,
            Self::Left => start.x - end.x,
            Self::Right => end.x - start.x,
        }
    }
}

/// A drawer root. Keep it mounted while closed, with its trigger and content.
#[derive(IntoElement)]
pub struct Drawer {
    id: ElementId,
    handle: DrawerHandle,
    side: DrawerSide,
    swipe_handle: bool,
    dismissible: bool,
    content: Option<DrawerContent>,
    base: Div,
}
impl Drawer {
    /// Creates a bottom drawer backed by a caller-owned open/close handle.
    pub fn new(id: impl Into<ElementId>, handle: &DrawerHandle) -> Self {
        Self {
            id: id.into(),
            handle: handle.clone(),
            side: DrawerSide::Bottom,
            swipe_handle: false,
            dismissible: true,
            content: None,
            base: div(),
        }
    }
    /// Sets the opening edge and swipe direction.
    pub fn direction(mut self, side: DrawerSide) -> Self {
        self.side = side;
        self
    }
    /// Adds an axis-aware grip at the exposed edge of the panel.
    pub fn show_swipe_handle(mut self, show: bool) -> Self {
        self.swipe_handle = show;
        self
    }
    /// Allows Escape, backdrop presses and swipes to dismiss. Close buttons remain explicit.
    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }
    /// Supplies the modal content; header and footer remain outside its scrollable body.
    pub fn content(mut self, content: DrawerContent) -> Self {
        self.content = Some(content);
        self
    }
}
impl Styled for Drawer {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}
impl ParentElement for Drawer {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.base.extend(elements);
    }
}

/// The styled panel. Override its width or height with standard GPUI sizing methods.
pub struct DrawerContent {
    id: ElementId,
    label: SharedString,
    children: Vec<AnyElement>,
    style: StyleRefinement,
}
impl DrawerContent {
    /// Creates an accessibly named panel with a stable caller-owned ID.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            children: Vec::new(),
            style: Default::default(),
        }
    }
}
impl Styled for DrawerContent {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
impl ParentElement for DrawerContent {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// A keyboard-accessible trigger opening the caller's drawer.
pub fn drawer_trigger(id: impl Into<ElementId>, handle: &DrawerHandle) -> Button {
    let handle = handle.clone();
    Button::new(id)
        .variant(ButtonVariant::Outline)
        .on_click(move |_, window, cx| handle.open(window, cx))
}
/// An explicit close action, with an outline appearance by default.
pub fn drawer_close(id: impl Into<ElementId>, handle: &DrawerHandle) -> Button {
    let handle = handle.clone();
    Button::new(id)
        .variant(ButtonVariant::Outline)
        .on_click(move |_, window, cx| handle.close(window, cx))
}
/// Inset heading region; vertical drawers center their headings, side drawers align left.
pub fn drawer_header(side: DrawerSide, cx: &App) -> Div {
    let t = UiTheme::read(cx);
    div()
        .flex()
        .flex_col()
        .flex_shrink_0()
        .gap(t.space(1.5))
        .p(t.space(4.))
        .when(side.vertical(), |d| d.text_center())
}
/// Scrollable body that shrinks before the header and footer on short viewports.
pub fn drawer_body(id: impl Into<ElementId>, cx: &App) -> Stateful<Div> {
    div()
        .id(id)
        .min_h(px(0.))
        .flex_1()
        .overflow_y_scroll()
        .p(UiTheme::read(cx).space(4.))
}
/// Stacked actions pinned after the body, with consistent insets.
pub fn drawer_footer(cx: &App) -> Div {
    let t = UiTheme::read(cx);
    div()
        .flex()
        .flex_col()
        .flex_shrink_0()
        .mt_auto()
        .gap(t.space(2.))
        .p(t.space(4.))
}
/// A semantic drawer heading using Nova typography.
pub fn drawer_title(id: impl Into<ElementId>, cx: &App) -> Stateful<Div> {
    dialog_title(id, cx)
}
/// Muted supporting text for the drawer heading.
pub fn drawer_description(id: impl Into<ElementId>, cx: &App) -> Stateful<Div> {
    dialog_description(id, cx)
}

#[derive(Default)]
struct Swipe {
    start: Option<(Point<Pixels>, Pixels)>,
    offset: Pixels,
    size: Size<Pixels>,
}

impl RenderOnce for Drawer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let base = self.base.id(self.id.clone());
        let Some(content) = self.content else {
            return base.into_any_element();
        };
        let open = self.handle.is_open();
        let t = UiTheme::read(cx).clone();
        let duration = if t.motion.reduced {
            Duration::ZERO
        } else {
            t.motion.normal
        };
        let policy = Transition::new(duration).easing(Easing::CubicBezier {
            x1: 0.22,
            y1: 1.,
            x2: 0.36,
            y2: 1.,
        });
        let presence = Presence::new((self.id.clone(), "presence"), open)
            .transition(policy.clone())
            .sample(window, cx);
        let visible = presence.should_render() && (open || presence.progress > 0.);
        let focus = modal_focus::prepare(self.id.clone(), visible, window, cx);
        let swipe = window.use_keyed_state((self.id.clone(), "swipe"), cx, |_, _| Swipe::default());
        if !visible {
            swipe.update(cx, |state, _| {
                state.start = None;
                state.offset = px(0.);
            });
            transition(
                (self.id.clone(), "drag"),
                0_f32,
                Transition::new(Duration::ZERO),
                window,
                cx,
            );
            return base.into_any_element();
        }
        let side = self.side;
        let vertical = side.vertical();
        let viewport = window.viewport_size();
        let side_width = if viewport.width >= t.space(160.) {
            t.space(96.)
        } else {
            viewport.width * 0.75
        };
        let state = swipe.read(cx);
        let extent = if vertical {
            state.size.height
        } else {
            state.size.width
        };
        let dragging = state.start.is_some();
        let offset = transition(
            (self.id.clone(), "drag"),
            f32::from(state.offset),
            if dragging {
                Transition::new(Duration::ZERO)
            } else {
                policy
            },
            window,
            cx,
        );
        let extent = if extent > px(0.) {
            extent
        } else if vertical {
            viewport.height
        } else {
            viewport.width
        };
        let shift = px(offset) + (extent - px(offset)).max(px(0.)) * (1. - presence.progress);
        let opacity = presence.progress * (1. - offset / f32::from(extent)).clamp(0., 1.);
        let scope = modal_focus::ModalFocus::new(self.id.clone());
        let measured = swipe.clone();
        let mut surface = scope
            .trap(div(), true)
            .id(content.id)
            .debug_selector(|| "drawer-panel".into())
            .track_focus(&focus)
            .role(Role::Dialog)
            .aria_label(content.label)
            .absolute()
            .occlude()
            .flex()
            .min_h(px(0.))
            .overflow_hidden()
            .bg(t.colors.popover)
            .text_color(t.colors.popover_foreground)
            .font_family(t.fonts.body.clone())
            .text_size(t.text(14.))
            .border_color(t.colors.border)
            .when(vertical, |d| d.flex_col())
            .map(|d| match side {
                DrawerSide::Bottom => d
                    .left_0()
                    .bottom(-shift)
                    .w_full()
                    .max_h((viewport.height - t.space(24.)).max(px(0.)))
                    .rounded_t(t.radius.xl)
                    .border_t_1(),
                DrawerSide::Top => d
                    .left_0()
                    .top(-shift)
                    .w_full()
                    .max_h((viewport.height - t.space(24.)).max(px(0.)))
                    .rounded_b(t.radius.xl)
                    .border_b_1(),
                DrawerSide::Left => d
                    .top_0()
                    .left(-shift)
                    .h(viewport.height)
                    .w(side_width)
                    .border_r_1(),
                DrawerSide::Right => d
                    .top_0()
                    .right(-shift)
                    .h(viewport.height)
                    .w(side_width)
                    .border_l_1(),
            })
            .on_prepaint(move |bounds, window, cx| {
                measured.update(cx, |state, _| {
                    if state.size != bounds.size {
                        state.size = bounds.size;
                        window.request_animation_frame();
                    }
                });
            })
            .child(scope.boundary(false))
            .child(scope.boundary(true));
        let style = content.style;
        let mut body = Some(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_h(px(0.))
                .min_w(px(0.))
                .children(content.children),
        );
        let grip_after = matches!(side, DrawerSide::Top | DrawerSide::Left);
        if grip_after {
            surface = surface.child(body.take().unwrap());
        }
        if self.swipe_handle {
            let start = swipe.clone();
            let enabled = self.dismissible && open;
            surface = surface.child(
                div()
                    .id((self.id.clone(), "grip"))
                    .debug_selector(|| "drawer-grip".into())
                    .flex()
                    .flex_shrink_0()
                    .items_center()
                    .justify_center()
                    .when(vertical, |d| d.w_full().h(t.space(7.)))
                    .when(!vertical, |d| d.h_full().w(t.space(7.)))
                    .when(enabled, |d| d.cursor_grab())
                    .child(
                        div()
                            .rounded_full()
                            .bg(t.colors.muted)
                            .when(vertical, |d| d.h(t.space(1.)).w(t.space(20.)))
                            .when(!vertical, |d| d.w(t.space(1.)).h(t.space(20.))),
                    )
                    .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                        if enabled {
                            start.update(cx, |state, _| {
                                state.start = Some((event.position, px(offset)));
                                state.offset = px(offset);
                            });
                            window.prevent_default();
                            cx.stop_propagation();
                            window.refresh();
                        }
                    }),
            );
        }
        if !grip_after {
            surface = surface.child(body.unwrap());
        }
        let moving = swipe.clone();
        let releasing = swipe;
        let release = self.handle.clone();
        let close = self.handle;
        let dismissible = self.dismissible;
        let host = Sheet::new(cx)
            .focus_handle(focus)
            .overlay(dialog_backdrop(cx).opacity(opacity))
            .overlay_closable(dismissible && open)
            .surface(apply_style(surface, &style))
            .request_close(move |window, cx| {
                if dismissible && open {
                    close.close(window, cx);
                }
            });
        base.child(
            gpui_kit::deferred(
                div()
                    .absolute()
                    .inset_0()
                    .child(
                        gpui_kit::canvas(
                            |_, _, _| (),
                            move |_, _, window, _| {
                                window.on_mouse_event(
                                    move |event: &gpui_kit::MouseMoveEvent, phase, window, cx| {
                                        if phase != gpui_kit::DispatchPhase::Capture
                                            || moving.read(cx).start.is_none()
                                        {
                                            return;
                                        }
                                        moving.update(cx, |state, _| {
                                            if event.pressed_button == Some(MouseButton::Left) {
                                                let (origin, initial) = state.start.unwrap();
                                                state.offset = (initial
                                                    + side.distance(origin, event.position))
                                                .clamp(px(0.), extent);
                                            } else {
                                                state.start = None;
                                                state.offset = px(0.);
                                            }
                                        });
                                        window.refresh();
                                        cx.stop_propagation();
                                    },
                                );
                                window.on_mouse_event(
                                    move |event: &gpui_kit::MouseUpEvent, phase, window, cx| {
                                        if phase != gpui_kit::DispatchPhase::Capture
                                            || event.button != MouseButton::Left
                                            || releasing.read(cx).start.is_none()
                                        {
                                            return;
                                        }
                                        let dismissed = releasing.update(cx, |state, _| {
                                            state.start = None;
                                            let dismissed = state.offset >= extent * 0.25;
                                            if !dismissed {
                                                state.offset = px(0.);
                                            }
                                            dismissed
                                        });
                                        if dismissed {
                                            release.close(window, cx);
                                        }
                                        window.refresh();
                                        cx.stop_propagation();
                                    },
                                );
                            },
                        )
                        .absolute()
                        .size_full(),
                    )
                    .child(host),
            )
            .with_priority(10),
        )
        .into_any_element()
    }
}
