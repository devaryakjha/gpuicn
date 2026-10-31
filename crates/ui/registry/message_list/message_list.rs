//! Nova virtualized message transcript built on GPUI Kit's `MessageScroller`.

use gpui_icons::{LucideIcon, lucide};
use gpui_kit::component::message_scroller::MessageScroller;
use std::rc::Rc;

use gpui_kit::{
    App, ElementId, Entity, FocusHandle, InteractiveElement as _, IntoElement, KeyDownEvent,
    ParentElement as _, RenderOnce, SharedString, StyleRefinement, Styled, Window, div,
    prelude::FluentBuilder as _,
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
            style: StyleRefinement::default(),
        }
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
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let scrolled_up = self.state.read(cx).is_scrolled_up();
        let mut renderer = self.renderer;
        // Rows own their spacing and gutters, so the Kit row wrapper adds none.
        let row_style = StyleRefinement::default().px_0().pb_0();
        let list_style = StyleRefinement::default().py(theme.space(2.));
        let scroller = MessageScroller::new(
            self.id.clone(),
            self.state.clone(),
            move |index, window, cx| renderer(index, window, cx),
        )
        .jump_button(false)
        .with_row_style(row_style)
        .with_list_style(list_style)
        .with_bottom_fade(theme.colors.background)
        .size_full();
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
            .child(scroller)
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
                                .shadow(theme.shadows.md.clone())
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
