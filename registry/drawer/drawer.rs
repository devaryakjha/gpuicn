//! Nova edge drawers with Kit's modal sheet host.
use super::{
    dialog::{DialogHandle, dialog_backdrop, modal_focus},
    theme::{UiTheme, apply_style},
};
use gpui_kit::{
    AnyElement, App, AppContext as _, Context, ElementId, InteractiveElement as _, IntoElement,
    ParentElement, Pixels, Point, Render, RenderOnce, Role, SharedString,
    StatefulInteractiveElement as _, StyleRefinement, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
/// The viewport edge from which a drawer appears.
pub enum DrawerSide {
    /// Attach to the top edge.
    Top,
    /// Attach to the right edge.
    Right,
    #[default]
    /// Attach to the bottom edge.
    Bottom,
    /// Attach to the left edge.
    Left,
}
#[derive(IntoElement)]
/// A Nova modal drawer with a retained open/close handle.
pub struct Drawer {
    id: ElementId,
    handle: DialogHandle,
    side: DrawerSide,
    label: SharedString,
    children: Vec<AnyElement>,
    style: StyleRefinement,
}
impl Drawer {
    /// Creates a bottom drawer with a retained handle and an accessible name.
    pub fn new(
        id: impl Into<ElementId>,
        handle: &DialogHandle,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            handle: handle.clone(),
            label: label.into(),
            side: Default::default(),
            children: Vec::new(),
            style: Default::default(),
        }
    }
    /// Selects the edge from which the drawer appears.
    pub fn side(mut self, side: DrawerSide) -> Self {
        self.side = side;
        self
    }
}
impl Styled for Drawer {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
impl ParentElement for Drawer {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}
#[derive(Clone)]
struct Swipe {
    id: ElementId,
    start: Point<Pixels>,
}
impl Render for Swipe {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        gpui_kit::Empty
    }
}
impl RenderOnce for Drawer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let open = self.handle.is_open();
        let focus_handle = modal_focus::prepare(self.id.clone(), open, window, cx);
        let offset = window.use_keyed_state((self.id.clone(), "swipe"), cx, |_, _| px(0.));
        if !open {
            offset.update(cx, |offset, _| *offset = px(0.));
            return gpui_kit::Empty.into_any_element();
        }
        let t = UiTheme::read(cx).clone();
        let side = self.side;
        let delta = *offset.read(cx);
        let scope = modal_focus::ModalFocus::new(self.id.clone());
        let close = self.handle.clone();
        let release = self.handle;
        let drag_id = self.id.clone();
        let swipe_offset = offset.clone();
        let release_offset = offset.clone();
        let grip = div()
            .id((self.id.clone(), "grip"))
            .w_full()
            .h(t.space(8.))
            .flex()
            .items_center()
            .justify_center()
            .cursor_grab()
            .child(
                div()
                    .h(t.space(1.))
                    .w(t.space(24.))
                    .rounded_full()
                    .bg(t.colors.muted),
            )
            .on_drag(self.id.clone(), |id, _, window, cx| {
                cx.new(|_| Swipe {
                    id: id.clone(),
                    start: window.mouse_position(),
                })
            });
        let surface = scope
            .trap(div(), true)
            .id(self.id)
            .track_focus(&focus_handle)
            .role(Role::Dialog)
            .aria_label(self.label)
            .absolute()
            .occlude()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(t.colors.popover)
            .text_color(t.colors.popover_foreground)
            .font_family(t.fonts.body.clone())
            .text_size(t.text(14.))
            .border_color(t.colors.border)
            .map(|d| match side {
                DrawerSide::Bottom => d
                    .left_0()
                    .bottom(-delta)
                    .w_full()
                    .max_h(t.space(100.))
                    .rounded_t(t.radius.xl)
                    .border_t_1(),
                DrawerSide::Top => d
                    .left_0()
                    .top(-delta)
                    .w_full()
                    .max_h(t.space(100.))
                    .rounded_b(t.radius.xl)
                    .border_b_1(),
                DrawerSide::Left => d
                    .top_0()
                    .left(-delta)
                    .h_full()
                    .w(t.space(80.))
                    .max_w_full()
                    .rounded_r(t.radius.xl)
                    .border_r_1(),
                DrawerSide::Right => d
                    .top_0()
                    .right(-delta)
                    .h_full()
                    .w(t.space(80.))
                    .max_w_full()
                    .rounded_l(t.radius.xl)
                    .border_l_1(),
            })
            .on_drag_move(move |event: &gpui_kit::DragMoveEvent<Swipe>, window, cx| {
                let drag = event.drag(cx);
                if drag.id != drag_id {
                    return;
                }
                let delta = match side {
                    DrawerSide::Top => drag.start.y - event.event.position.y,
                    DrawerSide::Bottom => event.event.position.y - drag.start.y,
                    DrawerSide::Left => drag.start.x - event.event.position.x,
                    DrawerSide::Right => event.event.position.x - drag.start.x,
                }
                .max(px(0.));
                swipe_offset.update(cx, |value, _| *value = delta);
                window.refresh();
            })
            .on_mouse_up(gpui_kit::MouseButton::Left, move |_, window, cx| {
                let dismissed = *release_offset.read(cx) >= px(80.);
                release_offset.update(cx, |value, _| *value = px(0.));
                if dismissed {
                    release.close(window, cx);
                }
                window.refresh();
            })
            .child(scope.boundary(false))
            .child(scope.boundary(true))
            .child(grip)
            .children(self.children);
        let sheet = gpui_kit::base::Sheet::new(cx)
            .overlay(dialog_backdrop(cx))
            .surface(apply_style(surface, &self.style))
            .request_close(move |window, cx| close.close(window, cx));
        gpui_kit::deferred(sheet)
            .with_priority(10)
            .into_any_element()
    }
}
