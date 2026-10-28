//! Native scrolling with Nova-styled GPUI Kit scrollbars.

use super::theme::UiTheme;
use gpui_kit::base::{Scrollbar, ScrollbarHandle, ScrollbarMode};
use gpui_kit::{
    AnyElement, App, Axis, ElementId, InteractiveElement as _, IntoElement, ParentElement,
    RenderOnce, Role, ScrollHandle, SharedString, StatefulInteractiveElement as _, Styled, Window,
    div, prelude::FluentBuilder as _, px,
};

/// Overlay on a relative container holding a list or scroll viewport.
pub fn scroll_area_scrollbar_for<H: ScrollbarHandle + Clone>(
    id: impl Into<ElementId>,
    target: &H,
    cx: &App,
) -> Scrollbar {
    let t = UiTheme::read(cx);
    Scrollbar::vertical(target)
        .id(id)
        .mode(ScrollbarMode::Scrolling)
        .styles(|styles| {
            styles
                .track(|style| style.width(t.space(2.5)))
                .thumb(|style| {
                    style
                        .bg(t.colors.border)
                        .width(t.space(2.))
                        .inset(px(1.))
                        .radius(t.space(1.))
                })
                .thumb_hover(|style| style.bg(t.colors.muted_foreground.opacity(0.72)))
        })
}

#[derive(IntoElement)]
/// A native scroll viewport with a Nova-styled Kit scrollbar.
pub struct ScrollArea {
    id: ElementId,
    style: gpui_kit::StyleRefinement,
    label: Option<SharedString>,
    axis: Axis,
    children: Vec<AnyElement>,
}
impl ScrollArea {
    /// Creates a `ScrollArea` with a stable caller-owned ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            style: Default::default(),
            label: None,
            axis: Axis::Vertical,
            children: vec![],
        }
    }
    /// Sets the accessible name of the control.
    pub fn aria_label(mut self, value: impl Into<SharedString>) -> Self {
        self.label = Some(value.into());
        self
    }
    /// Selects horizontal or vertical layout and keyboard navigation.
    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }
}
impl ParentElement for ScrollArea {
    fn extend(&mut self, children: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(children);
    }
}
impl Styled for ScrollArea {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}
impl RenderOnce for ScrollArea {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = UiTheme::read(cx).clone();
        let handle = window
            .use_keyed_state((self.id.clone(), "scroll"), cx, |_, _| ScrollHandle::new())
            .read(cx)
            .clone();
        let keyboard = handle.clone();
        let axis = self.axis;
        let viewport = div()
            .id("viewport")
            .role(Role::ScrollView)
            .focusable()
            .size_full()
            .track_scroll(&handle)
            .when(axis == Axis::Vertical, |viewport| {
                viewport.overflow_y_scroll()
            })
            .when(axis == Axis::Horizontal, |viewport| {
                viewport.overflow_x_scroll()
            })
            .when_some(self.label, |viewport, label| viewport.aria_label(label))
            .on_key_down(move |event, window, cx| {
                if event.keystroke.modifiers.modified() {
                    return;
                }
                let mut offset = keyboard.offset();
                let max = if axis == Axis::Vertical {
                    keyboard.max_offset().y
                } else {
                    keyboard.max_offset().x
                };
                let page = if axis == Axis::Vertical {
                    keyboard.bounds().size.height
                } else {
                    keyboard.bounds().size.width
                };
                let value = if axis == Axis::Vertical {
                    &mut offset.y
                } else {
                    &mut offset.x
                };
                *value = match event.keystroke.key.as_str() {
                    "up" if axis == Axis::Vertical => *value + px(32.),
                    "down" if axis == Axis::Vertical => *value - px(32.),
                    "left" if axis == Axis::Horizontal => *value + px(32.),
                    "right" if axis == Axis::Horizontal => *value - px(32.),
                    "pageup" => *value + page,
                    "pagedown" => *value - page,
                    "home" => px(0.),
                    "end" => -max,
                    _ => return,
                };
                *value = (*value).clamp(-max.max(px(0.)), px(0.));
                keyboard.set_offset(offset);
                window.refresh();
                cx.stop_propagation();
            })
            .child(div().flex().flex_col().min_w_full().children(self.children));
        let scrollbar = scroll_area_scrollbar_for("scrollbar", &handle, cx).axis(axis);
        let root = div()
            .id(self.id)
            .relative()
            .min_h_0()
            .min_w_0()
            .font_family(t.fonts.body)
            .text_color(t.colors.foreground)
            .child(viewport)
            .child(scrollbar);
        super::theme::apply_style(root, &self.style)
    }
}
