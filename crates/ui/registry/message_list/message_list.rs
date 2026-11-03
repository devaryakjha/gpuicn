//! Nova virtualized message transcript built on GPUI Kit's `MessageScroller`.

use gpui_icons::{LucideIcon, lucide};
use gpui_kit::base::ScrollbarHandle;
use gpui_kit::component::message_scroller::MessageScroller;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use gpui_kit::{
    App, Bounds, ElementId, Entity, FocusHandle, InteractiveElement as _, IntoElement,
    KeyDownEvent, ListOffset, ParentElement as _, Pixels, Point, RenderOnce, SharedString, Size,
    StyleRefinement, Styled, Window, div, point, prelude::FluentBuilder as _, px, size,
};

pub use gpui_kit::component::message_scroller::MessageScrollerState as MessageListState;

use super::button::{Button, ButtonSize, ButtonVariant};
use super::theme::{UiTheme, apply_style};

type RowRenderer = Box<dyn FnMut(usize, &mut Window, &mut App) -> gpui_kit::AnyElement>;
type SelectHandler = Rc<dyn Fn(Option<usize>, &mut Window, &mut App) + 'static>;
type KeyHandler = Rc<dyn Fn(&KeyDownEvent, Option<usize>, &mut Window, &mut App) -> bool + 'static>;
type SelectableRow = Rc<dyn Fn(usize, &App) -> bool + 'static>;

/// A variable-height, virtualized chat transcript that follows new messages.
///
/// Rows are rendered on demand by index, so the application keeps its messages
/// and tells the retained `MessageListState` about appends, prepends and edits.
/// While the reader is scrolled up, a "Jump to latest" pill returns to the end.
#[derive(IntoElement)]
pub struct MessageList {
    id: ElementId,
    state: Entity<MessageListState>,
    renderer: RowRenderer,
    unseen: usize,
    jump_label: SharedString,
    focus: Option<FocusHandle>,
    selected: Option<usize>,
    on_select: Option<SelectHandler>,
    on_key: Option<KeyHandler>,
    selectable: Option<SelectableRow>,
    scrollbar: Option<MessageListScrollbar>,
    style: StyleRefinement,
}

impl MessageList {
    /// Creates a transcript bound to retained state, rendering row `index` on demand.
    pub fn new<E: IntoElement>(
        id: impl Into<ElementId>,
        state: &Entity<MessageListState>,
        mut row: impl FnMut(usize, &mut Window, &mut App) -> E + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            renderer: Box::new(move |index, window, cx| row(index, window, cx).into_any_element()),
            unseen: 0,
            jump_label: "Jump to latest".into(),
            focus: None,
            selected: None,
            on_select: None,
            on_key: None,
            selectable: None,
            scrollbar: None,
            style: StyleRefinement::default(),
        }
    }

    /// Shares the scrollbar's position model with the application, which can
    /// read the estimated offset and content height from it. Without this the
    /// transcript keeps one per element id.
    pub fn scrollbar_handle(mut self, handle: &MessageListScrollbar) -> Self {
        self.scrollbar = Some(handle.clone());
        self
    }

    /// Counts messages that arrived while the reader was scrolled up; the jump
    /// pill then reads "N new messages".
    pub fn unseen(mut self, count: usize) -> Self {
        self.unseen = count;
        self
    }

    /// Sets the jump pill's label when there are no unseen messages.
    pub fn jump_label(mut self, label: impl Into<SharedString>) -> Self {
        self.jump_label = label.into();
        self
    }

    /// Makes the transcript focusable for keyboard selection mode. A click
    /// focuses it; while it has focus, Up and Down move the selection and
    /// Escape clears it.
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
        self
    }

    /// Sets the selected row, which the application renders with
    /// `MessageHighlight::Selected`. The transcript scrolls it into view.
    pub fn selected(mut self, index: Option<usize>) -> Self {
        self.selected = index;
        self
    }

    /// Runs when Up, Down or Escape changes the selection.
    pub fn on_select(
        mut self,
        handler: impl Fn(Option<usize>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    /// Sees every key press before the selection keys, with the selected row.
    /// Return `true` to claim it, for example E to edit or R to reply.
    pub fn on_key(
        mut self,
        handler: impl Fn(&KeyDownEvent, Option<usize>, &mut Window, &mut App) -> bool + 'static,
    ) -> Self {
        self.on_key = Some(Rc::new(handler));
        self
    }

    /// Limits keyboard selection to rows that pass, skipping dividers.
    pub fn selectable(mut self, selectable: impl Fn(usize, &App) -> bool + 'static) -> Self {
        self.selectable = Some(Rc::new(selectable));
        self
    }
}

/// Where a scrollbar drag asked the transcript to go, applied on the next render.
#[derive(Clone, Copy, Debug, PartialEq)]
enum ScrollTarget {
    Item(usize),
    End,
}

#[derive(Default)]
struct ScrollbarModel {
    viewport: Bounds<Pixels>,
    /// Row heights measured so far, by index. Prepends shift the indices, which
    /// only matters for the average, so the map is not rekeyed.
    heights: HashMap<usize, Pixels>,
    height_sum: Pixels,
    count: usize,
    top: ListOffset,
    at_end: bool,
    dragging: bool,
    requested: Option<Pixels>,
    pending: Option<ScrollTarget>,
}

impl ScrollbarModel {
    fn average_row_height(&self) -> Option<Pixels> {
        (!self.heights.is_empty()).then(|| self.height_sum / self.heights.len() as f32)
    }

    fn content_height(&self) -> Pixels {
        match self.average_row_height() {
            Some(average) if self.count > 0 => {
                (average * self.count as f32).max(self.viewport.size.height)
            }
            _ => self.viewport.size.height,
        }
    }

    fn max_offset(&self) -> Pixels {
        (self.content_height() - self.viewport.size.height).max(px(0.))
    }

    /// Scrolled distance from the top, estimated from the row under the viewport's top edge.
    fn scrolled(&self) -> Pixels {
        if let Some(requested) = self.requested.filter(|_| self.dragging) {
            return requested;
        }
        if self.at_end {
            return self.max_offset();
        }
        let Some(average) = self.average_row_height() else {
            return px(0.);
        };
        (average * self.top.item_ix as f32 + self.top.offset_in_item)
            .clamp(px(0.), self.max_offset())
    }

    fn request(&mut self, scrolled: Pixels) {
        let max = self.max_offset();
        let scrolled = scrolled.clamp(px(0.), max);
        self.requested = Some(scrolled);
        self.pending = Some(match self.average_row_height() {
            Some(average) if scrolled < max - px(0.5) && average > px(0.) => ScrollTarget::Item(
                ((scrolled / average).floor() as usize).min(self.count.saturating_sub(1)),
            ),
            _ => ScrollTarget::End,
        });
    }
}

/// The transcript's scrollbar position, estimated from measured rows.
///
/// GPUI's `list` counts a row it has never laid out as 0px, so a scrollbar
/// fed by it describes only the rows the reader has passed: the thumb jumps
/// as rows are measured and resets when a width change drops the cache. This
/// handle instead measures every row the transcript lays out, estimates the
/// content height as `row count × average row height`, and places the thumb
/// from the row under the viewport's top edge. Dragging the thumb scrolls to
/// the row at that fraction of the transcript; dragging to the bottom resumes
/// following the tail.
#[derive(Clone, Default)]
pub struct MessageListScrollbar(Rc<RefCell<ScrollbarModel>>);

impl MessageListScrollbar {
    /// Creates a handle to share between a `MessageList` and the application.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of distinct rows measured so far.
    pub fn measured_rows(&self) -> usize {
        self.0.borrow().heights.len()
    }

    /// Estimated distance scrolled from the top of the transcript, in pixels.
    pub fn scrolled(&self) -> Pixels {
        self.0.borrow().scrolled()
    }

    /// Estimated height of the whole transcript, never less than the viewport.
    pub fn content_height(&self) -> Pixels {
        self.0.borrow().content_height()
    }

    fn measure(&self, index: usize, height: Pixels) {
        let mut model = self.0.borrow_mut();
        if let Some(previous) = model.heights.insert(index, height) {
            model.height_sum -= previous;
        }
        model.height_sum += height;
    }

    fn capture(&self, viewport: Bounds<Pixels>, state: &MessageListState) {
        let mut model = self.0.borrow_mut();
        if model.viewport.size.width != viewport.size.width {
            // Rows reflow at a new width; measure them again.
            model.heights.clear();
            model.height_sum = px(0.);
        }
        model.viewport = viewport;
        model.count = state.item_count();
        model.top = state.logical_scroll_top();
        model.at_end = !state.is_scrolled_up();
    }

    fn take_pending(&self) -> Option<ScrollTarget> {
        let mut model = self.0.borrow_mut();
        let pending = model.pending.take();
        if !model.dragging {
            model.requested = None;
        }
        pending
    }
}

impl ScrollbarHandle for MessageListScrollbar {
    fn viewport_bounds(&self) -> Bounds<Pixels> {
        self.0.borrow().viewport
    }

    fn offset(&self) -> Point<Pixels> {
        point(px(0.), -self.scrolled())
    }

    fn set_offset(&self, offset: Point<Pixels>) {
        self.0.borrow_mut().request(-offset.y);
    }

    fn content_size(&self) -> Size<Pixels> {
        let model = self.0.borrow();
        size(model.viewport.size.width, model.content_height())
    }

    fn start_drag(&self) {
        self.0.borrow_mut().dragging = true;
    }

    fn end_drag(&self) {
        let mut model = self.0.borrow_mut();
        model.dragging = false;
        model.requested = None;
    }
}

/// The row a selection key moves to, or `None` to leave the selection alone.
/// Up from no selection starts at the newest selectable row.
#[doc(hidden)]
pub fn next_selection(
    key: &str,
    selected: Option<usize>,
    count: usize,
    selectable: impl Fn(usize) -> bool,
) -> Option<Option<usize>> {
    match key {
        "escape" => selected.map(|_| None),
        "up" => {
            let start = selected.unwrap_or(count);
            (0..start).rev().find(|&index| selectable(index)).map(Some)
        }
        "down" => {
            let start = selected? + 1;
            (start..count).find(|&index| selectable(index)).map(Some)
        }
        _ => None,
    }
}

impl Styled for MessageList {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for MessageList {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let scrollbar = match self.scrollbar {
            Some(handle) => handle,
            None => window
                .use_keyed_state((self.id.clone(), "scrollbar-model"), cx, |_, _| {
                    MessageListScrollbar::new()
                })
                .read(cx)
                .clone(),
        };
        // A thumb drag lands here, before the list lays out this frame.
        if let Some(target) = scrollbar.take_pending() {
            self.state.update(cx, |state, cx| match target {
                ScrollTarget::Item(index) => {
                    state.scroll_to_item(index, cx);
                }
                ScrollTarget::End => state.scroll_to_end(cx),
            });
        }
        let scrolled_up = self.state.read(cx).is_scrolled_up();
        let mut renderer = self.renderer;
        let measure = scrollbar.clone();
        // Rows own their spacing and gutters, so the Kit row wrapper adds none.
        let row_style = StyleRefinement::default().px_0().pb_0();
        let list_style = StyleRefinement::default().py(theme.space(2.));
        let scroller = MessageScroller::new(
            self.id.clone(),
            self.state.clone(),
            move |index, window, cx| {
                let measure = measure.clone();
                div()
                    .w_full()
                    .on_children_prepainted(move |bounds, _, _| {
                        measure.measure(index, bounds.iter().map(|b| b.size.height).sum())
                    })
                    .child(renderer(index, window, cx))
            },
        )
        .jump_button(false)
        .scrollbar(false)
        .with_row_style(row_style)
        .with_list_style(list_style)
        .with_bottom_fade(theme.colors.background)
        .size_full();
        let capture = scrollbar.clone();
        let capture_state = self.state.clone();
        let viewport = div()
            .size_full()
            .min_h_0()
            .child(scroller)
            .on_children_prepainted(move |bounds, _, cx| {
                if let Some(bounds) = bounds.first() {
                    capture.capture(*bounds, capture_state.read(cx));
                }
            });
        let label: SharedString = match self.unseen {
            0 => self.jump_label.clone(),
            1 => "1 new message".into(),
            count => format!("{count} new messages").into(),
        };
        let state = self.state.clone();
        let keys_state = self.state.clone();
        let selected = self.selected;
        let on_select = self.on_select.clone();
        let on_key = self.on_key.clone();
        let selectable = self.selectable.clone();
        let root = div()
            .key_context("MessageList")
            .when_some(self.focus.clone(), |root, focus| {
                let target = focus.clone();
                root.track_focus(&focus).on_mouse_down(
                    gpui_kit::MouseButton::Left,
                    move |_, window, cx| {
                        if !target.contains_focused(window, cx) {
                            window.focus(&target, cx);
                        }
                    },
                )
            })
            .on_key_down(move |event, window, cx| {
                if let Some(handler) = &on_key
                    && handler(event, selected, window, cx)
                {
                    cx.stop_propagation();
                    return;
                }
                let modifiers = event.keystroke.modifiers;
                if modifiers.control || modifiers.alt || modifiers.platform || modifiers.shift {
                    return;
                }
                let count = keys_state.read(cx).item_count();
                let next = next_selection(&event.keystroke.key, selected, count, |index| {
                    selectable.as_ref().is_none_or(|matches| matches(index, cx))
                });
                let Some(next) = next else {
                    return;
                };
                cx.stop_propagation();
                if let Some(index) = next {
                    keys_state.update(cx, |state, cx| {
                        state.scroll_to_item(index, cx);
                    });
                }
                if let Some(handler) = &on_select {
                    handler(next, window, cx);
                }
            })
            .relative()
            .size_full()
            .min_h_0()
            .bg(theme.colors.background)
            .child(viewport)
            .child(super::scroll_area::scroll_area_scrollbar_for(
                (self.id.clone(), "scrollbar"),
                &scrollbar,
                cx,
            ))
            .when(scrolled_up, |root| {
                root.child(
                    div()
                        .absolute()
                        .left_0()
                        .right_0()
                        .bottom(theme.space(3.))
                        .flex()
                        .justify_center()
                        .child(
                            Button::new((self.id, "jump-to-latest"))
                                .variant(if self.unseen > 0 {
                                    ButtonVariant::Default
                                } else {
                                    ButtonVariant::Secondary
                                })
                                .size(ButtonSize::Sm)
                                .rounded_full()
                                .shadow(theme.shadows.md.to_vec())
                                .on_click(move |_, _, cx| {
                                    state.update(cx, |state, cx| state.scroll_to_end(cx))
                                })
                                .child(
                                    lucide(LucideIcon::ArrowDown)
                                        .size(theme.space(3.5))
                                        .text_color(if self.unseen > 0 {
                                            theme.colors.primary_foreground
                                        } else {
                                            theme.colors.secondary_foreground
                                        }),
                                )
                                .label(label),
                        ),
                )
            });
        apply_style(root, &self.style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{AppContext as _, Context, Render, TestAppContext};

    const ROWS: usize = 400;
    const ROW_HEIGHT: f32 = 40.;

    struct Transcript {
        state: Entity<MessageListState>,
        scrollbar: MessageListScrollbar,
    }
    impl Render for Transcript {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w(px(600.)).h(px(400.)).child(
                MessageList::new("transcript", &self.state, |index, _, _| {
                    div().h(px(ROW_HEIGHT)).child(format!("Row {index}"))
                })
                .scrollbar_handle(&self.scrollbar),
            )
        }
    }

    fn draw(cx: &mut gpui_kit::VisualTestContext) {
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.run_until_parked();
    }

    #[test]
    fn selection_keys_skip_unselectable_rows() {
        let selectable = |index: usize| index % 2 == 1;
        assert_eq!(next_selection("up", None, 6, selectable), Some(Some(5)));
        assert_eq!(next_selection("up", Some(5), 6, selectable), Some(Some(3)));
        assert_eq!(next_selection("up", Some(1), 6, selectable), None);
        assert_eq!(next_selection("down", None, 6, selectable), None);
        assert_eq!(
            next_selection("down", Some(3), 6, selectable),
            Some(Some(5))
        );
        assert_eq!(next_selection("escape", Some(3), 6, selectable), Some(None));
        assert_eq!(next_selection("escape", None, 6, selectable), None);
    }

    /// The thumb describes the whole transcript, not only the rows GPUI has
    /// measured (devaryakjha/gpuicn#83).
    #[gpui_kit::test]
    fn the_scrollbar_tracks_position_through_unmeasured_rows(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let scrollbar = MessageListScrollbar::new();
        let handle = scrollbar.clone();
        let (view, cx) = cx.add_window_view(move |_, cx| Transcript {
            state: cx.new(|cx| MessageListState::new(ROWS, cx)),
            scrollbar: handle,
        });
        draw(cx);
        draw(cx);
        let viewport = scrollbar.viewport_bounds().size.height;
        assert!(viewport > px(300.), "viewport measured: {viewport:?}");
        // Only a screen of rows is measured, yet the content height covers all of them.
        assert!(scrollbar.measured_rows() < ROWS / 4);
        let content = scrollbar.content_height();
        let expected = px(ROW_HEIGHT * ROWS as f32);
        assert!(
            (content - expected).abs() < px(ROW_HEIGHT * 2.),
            "content {content:?}, expected about {expected:?}"
        );
        // Following the tail puts the thumb at the end of the track.
        assert_eq!(scrollbar.scrolled(), content - viewport);

        // Jump to the middle: the thumb sits at half the track.
        view.update(cx, |view, cx| {
            view.state.update(cx, |state, cx| {
                state.scroll_to_item(ROWS / 2, cx);
            })
        });
        draw(cx);
        let scrolled = scrolled_px(&scrollbar);
        assert!(
            (scrolled - px(ROW_HEIGHT * (ROWS / 2) as f32)).abs() < px(ROW_HEIGHT),
            "scrolled {scrolled:?}"
        );
        assert!(scrollbar.measured_rows() < ROWS / 2);

        // Dragging the thumb to a quarter of the track lands on that row.
        scrollbar.start_drag();
        scrollbar.set_offset(point(px(0.), -px(ROW_HEIGHT * 100.)));
        assert_eq!(scrolled_px(&scrollbar), px(ROW_HEIGHT * 100.));
        scrollbar.end_drag();
        draw(cx);
        let top = cx.update(|_, cx| view.read(cx).state.read(cx).logical_scroll_top());
        assert!(
            (99..=101).contains(&top.item_ix),
            "scrolled to row {}",
            top.item_ix
        );

        // Dragging to the bottom of the track resumes following the tail.
        scrollbar.start_drag();
        scrollbar.set_offset(point(px(0.), -px(ROW_HEIGHT * ROWS as f32)));
        scrollbar.end_drag();
        draw(cx);
        draw(cx);
        assert!(cx.update(|_, cx| view.read(cx).state.read(cx).is_following_tail()));
        assert_eq!(scrollbar.scrolled(), scrollbar.content_height() - viewport);
    }

    fn scrolled_px(scrollbar: &MessageListScrollbar) -> Pixels {
        -scrollbar.offset().y
    }
}
