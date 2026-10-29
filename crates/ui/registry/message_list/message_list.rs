//! Nova virtualized message transcript built on GPUI Kit's `MessageScroller`.

use gpui_icons::{LucideIcon, lucide};
use gpui_kit::component::message_scroller::MessageScroller;
use gpui_kit::{
    App, ElementId, Entity, IntoElement, ParentElement as _, RenderOnce, SharedString,
    StyleRefinement, Styled, Window, div, prelude::FluentBuilder as _,
};

pub use gpui_kit::component::message_scroller::MessageScrollerState as MessageListState;

use super::button::{Button, ButtonSize, ButtonVariant};
use super::theme::{UiTheme, apply_style};

type RowRenderer = Box<dyn FnMut(usize, &mut Window, &mut App) -> gpui_kit::AnyElement>;

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
        let root = div()
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
