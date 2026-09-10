//! Nova notifications using Kit's toast queue, timers and measured stack.
use super::{
    button::{Button, ButtonSize, ButtonVariant},
    theme::UiTheme,
};
use gpui_icons::{LucideIcon, lucide};
use gpui_kit::base::{
    Toast, ToastManager, ToastMotion, ToastOptions, ToastStack, ToastStackState,
    ToastTransitionStatus,
};
use gpui_kit::{
    Anchor, Context, ElementId, FocusHandle, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, Styled, Task, Window, div, px,
};
use std::time::Duration;
use web_time::Instant;

struct Notice {
    title: SharedString,
    description: SharedString,
}
/// Own this entity in the application and mount it once above the page content.
pub struct ToastState {
    manager: ToastManager<SharedString, Notice>,
    stack: ToastStackState,
    focus: FocusHandle,
    timer: Option<Task<()>>,
}
impl ToastState {
    /// Creates an empty notification queue and its focus handle.
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            manager: ToastManager::new(ToastMotion::sonner()),
            stack: Default::default(),
            focus: cx.focus_handle(),
            timer: None,
        }
    }
    /// Replaces an existing ID or adds a new notification. `None` keeps it until dismissed.
    pub fn push(
        &mut self,
        id: impl Into<SharedString>,
        title: impl Into<SharedString>,
        description: impl Into<SharedString>,
        timeout: Option<Duration>,
        cx: &mut Context<Self>,
    ) {
        let empty = self.manager.is_empty();
        self.manager.push(
            id.into(),
            Notice {
                title: title.into(),
                description: description.into(),
            },
            ToastOptions { timeout },
            Instant::now(),
        );
        if empty {
            self.timer = Some(cx.spawn(async move |this, cx| {
                loop {
                    cx.background_executor()
                        .timer(Duration::from_millis(50))
                        .await;
                    let active = this
                        .update(cx, |state, cx| {
                            if state
                                .manager
                                .advance(Instant::now(), state.stack.is_expanded())
                                .changed
                            {
                                cx.notify();
                            }
                            !state.manager.is_empty()
                        })
                        .unwrap_or(false);
                    if !active {
                        break;
                    }
                }
            }));
        }
        cx.notify();
    }
    /// Removes the notification with this ID.
    pub fn dismiss(&mut self, id: &SharedString, cx: &mut Context<Self>) {
        if self.manager.dismiss(id, Instant::now()) {
            cx.notify();
        }
    }
    /// Returns whether the notification queue is empty.
    pub fn is_empty(&self) -> bool {
        self.manager.is_empty()
    }
}
impl Render for ToastState {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let viewport = window.viewport_size();
        let mut stack = ToastStack::new("toasts", self.stack.clone())
            .placement(Anchor::BottomLeft)
            .focus_handle(self.focus.clone())
            .absolute()
            .left(theme.space(4.))
            .bottom(theme.space(4.))
            .w((viewport.width - theme.space(8.)).min(theme.space(96.)));
        for (id, notice, phase) in self.manager.visible(3) {
            let dismiss = id.clone();
            let state = cx.entity().downgrade();
            let close = Button::new(ElementId::from((ElementId::from(id.clone()), "close")))
                .aria_label("Close notification")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .absolute()
                .top(theme.space(2.))
                .right(theme.space(2.))
                .on_click(move |_, _, cx| {
                    state
                        .update(cx, |state, cx| state.dismiss(&dismiss, cx))
                        .ok();
                })
                .child(
                    lucide(LucideIcon::X)
                        .size(theme.space(4.))
                        .text_color(colors.muted_foreground),
                );
            let presence = super::theme::presence(
                ElementId::from((ElementId::from(id.clone()), "presence")),
                phase != ToastTransitionStatus::Ending,
                window,
                cx,
            );
            let toast = Toast::new(id.clone())
                .transition_status(phase)
                .relative()
                .occlude()
                .w_full()
                .rounded(theme.radius.two_xl)
                .border_1()
                .border_color(colors.border)
                .bg(colors.popover)
                .text_color(colors.popover_foreground)
                .font_family(theme.fonts.body.clone())
                .p(theme.space(4.))
                .pr(theme.space(12.))
                .flex()
                .flex_col()
                .gap(theme.space(1.))
                .opacity(presence.progress)
                .child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .text_size(theme.text(14.))
                        .child(notice.title.clone()),
                )
                .child(
                    div()
                        .text_size(theme.text(14.))
                        .text_color(colors.muted_foreground)
                        .child(notice.description.clone()),
                )
                .child(close);
            stack = stack.item(id.clone(), toast);
        }
        gpui_kit::deferred(
            gpui_kit::anchored()
                .position(gpui_kit::point(px(0.), px(0.)))
                .child(div().w(viewport.width).h(viewport.height).child(stack)),
        )
        .with_priority(20)
    }
}
