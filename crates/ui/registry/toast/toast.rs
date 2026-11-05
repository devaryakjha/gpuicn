//! Nova notifications using Kit's toast queue, timers and measured stack.
use super::{
    button::{Button, ButtonSize, ButtonVariant},
    theme::{
        Direction, UiText, UiTheme,
        locale::{self, DirectionalStyled as _},
        primitives::{surface, text},
    },
};
use gpui_icons::{LucideIcon, lucide};
use gpui_kit::base::{
    Toast, ToastManager, ToastMotion, ToastOptions, ToastStack, ToastStackState,
    ToastTransitionStatus,
};
use gpui_kit::{
    Anchor, App, Context, ElementId, FocusHandle, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement as _, Pixels, Render, SharedString, Styled, Task, Window, div, px,
};
use std::{rc::Rc, time::Duration};
use web_time::Instant;

type ActionHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;
type DismissHandler = Rc<dyn Fn(NoticeDismissal, &mut App) + 'static>;

/// Why a notification left the screen, passed to `Notice::on_dismiss`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NoticeDismissal {
    /// The reader pressed the `action` button.
    Action,
    /// The reader pressed the `cancel` button, such as "Later".
    Cancel,
    /// The reader pressed the close button.
    Close,
    /// The timeout ran out.
    Timeout,
    /// The app called `ToastState::dismiss`.
    Dismissed,
    /// The app pushed another notification with the same ID.
    Replaced,
}

impl NoticeDismissal {
    /// Returns whether the reader dismissed the notification themselves, with
    /// the cancel or close button.
    pub fn by_reader(self) -> bool {
        matches!(self, Self::Cancel | Self::Close)
    }
}

/// Where the stack sits in the window, set with `ToastState::set_placement`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToastPlacement {
    /// Top left corner; new notifications enter at the top.
    TopLeft,
    /// Top edge, centered.
    TopCenter,
    /// Top right corner.
    TopRight,
    /// Bottom left corner, the default; new notifications enter at the bottom.
    #[default]
    BottomLeft,
    /// Bottom edge, centered.
    BottomCenter,
    /// Bottom right corner.
    BottomRight,
}

impl ToastPlacement {
    /// The corner on screen: left and right are the reading start and end,
    /// so they swap in right-to-left layouts.
    pub fn physical(self, direction: Direction) -> Self {
        if !direction.is_rtl() {
            return self;
        }
        match self {
            Self::TopLeft => Self::TopRight,
            Self::TopRight => Self::TopLeft,
            Self::BottomLeft => Self::BottomRight,
            Self::BottomRight => Self::BottomLeft,
            other => other,
        }
    }

    fn anchor(self) -> Anchor {
        match self {
            Self::TopLeft => Anchor::TopLeft,
            Self::TopCenter => Anchor::TopCenter,
            Self::TopRight => Anchor::TopRight,
            Self::BottomLeft => Anchor::BottomLeft,
            Self::BottomCenter => Anchor::BottomCenter,
            Self::BottomRight => Anchor::BottomRight,
        }
    }
}

/// One notification: a title, optional description, up to two buttons and an
/// auto-dismiss timeout.
#[derive(Clone)]
pub struct Notice {
    title: SharedString,
    description: SharedString,
    timeout: Option<Duration>,
    action: Option<(SharedString, ActionHandler)>,
    cancel: Option<SharedString>,
    on_dismiss: Option<DismissHandler>,
}
impl std::fmt::Debug for Notice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Notice")
            .field("title", &self.title)
            .field("description", &self.description)
            .field("timeout", &self.timeout)
            .field("action", &self.action.as_ref().map(|(label, _)| label))
            .field("cancel", &self.cancel)
            .field("on_dismiss", &self.on_dismiss.is_some())
            .finish()
    }
}
impl Notice {
    /// Creates a notification that dismisses itself after five seconds.
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            description: SharedString::default(),
            timeout: Some(Duration::from_secs(5)),
            action: None,
            cancel: None,
            on_dismiss: None,
        }
    }
    /// Adds a button, such as "Undo", "Retry" or "Show in folder". Pressing it
    /// runs `handler` and dismisses the notification. Hovering the stack
    /// pauses the timeout so the button can be reached.
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.action = Some((label.into(), Rc::new(handler)));
        self
    }
    /// Adds a secondary button, such as "Dismiss", that only closes the notification.
    pub fn cancel(mut self, label: impl Into<SharedString>) -> Self {
        self.cancel = Some(label.into());
        self
    }
    /// Runs once when the notification starts to leave, with the reason, so
    /// the app can tell "Later" or the close button apart from the action or
    /// a timeout. It runs after the current update, so it may push again.
    pub fn on_dismiss(mut self, handler: impl Fn(NoticeDismissal, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }
    /// Secondary text below the title.
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = description.into();
        self
    }
    /// Dismisses the notification after `timeout`.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    /// Keeps the notification until the user dismisses it.
    pub fn persistent(mut self) -> Self {
        self.timeout = None;
        self
    }
}
/// Poll interval while a hovered stack holds its timers, the only state the
/// manager cannot announce ahead of time.
const PAUSED_POLL: Duration = Duration::from_millis(100);

/// Own this entity in the application and mount it once above the page content.
pub struct ToastState {
    manager: ToastManager<SharedString, Notice>,
    motion: ToastMotion,
    stack: ToastStackState,
    focus: FocusHandle,
    /// When each mounted toast next changes phase, by ID. Entry and exit
    /// transitions end after the motion durations; a shown toast ends when its
    /// timeout runs out, which a hovered stack pushes back.
    deadlines: Vec<(SharedString, Instant)>,
    timer: Option<Task<()>>,
    placement: ToastPlacement,
    inset: Option<(Pixels, Pixels)>,
}
impl ToastState {
    /// Creates an empty notification queue and its focus handle.
    pub fn new(cx: &mut Context<Self>) -> Self {
        let motion = ToastMotion::sonner();
        Self {
            manager: ToastManager::new(motion),
            motion,
            stack: Default::default(),
            focus: cx.focus_handle(),
            deadlines: Vec::new(),
            timer: None,
            placement: ToastPlacement::default(),
            inset: None,
        }
    }
    /// Moves the stack to another corner or edge of the window.
    pub fn set_placement(&mut self, placement: ToastPlacement, cx: &mut Context<Self>) {
        self.placement = placement;
        cx.notify();
    }
    /// Sets the stack's distance from the window's side and from its top or
    /// bottom edge, such as to clear a composer. The default is 16px each.
    pub fn set_inset(&mut self, horizontal: Pixels, vertical: Pixels, cx: &mut Context<Self>) {
        self.inset = Some((horizontal, vertical));
        cx.notify();
    }
    /// Returns whether a notification with this ID is queued or on screen and
    /// not already leaving.
    pub fn contains(&self, id: &str) -> bool {
        self.manager
            .iter()
            .any(|(other, _, phase)| other.as_ref() == id && phase != ToastTransitionStatus::Ending)
    }
    /// Replaces an existing ID or adds a new notification.
    pub fn push(&mut self, id: impl Into<SharedString>, notice: Notice, cx: &mut Context<Self>) {
        let id = id.into();
        let timeout = notice.timeout;
        let now = Instant::now();
        let showing = self.contains(&id);
        let replaced = self
            .manager
            .push(id.clone(), notice, ToastOptions { timeout }, now);
        if let Some(replaced) = replaced.filter(|_| showing) {
            Self::notify_dismissed(&replaced, NoticeDismissal::Replaced, cx);
        }
        self.set_deadline(id, now + self.motion.duration);
        self.schedule(cx);
        cx.notify();
    }
    /// Removes the notification with this ID.
    pub fn dismiss(&mut self, id: &SharedString, cx: &mut Context<Self>) {
        self.dismiss_with(id, NoticeDismissal::Dismissed, cx);
    }
    fn dismiss_with(&mut self, id: &SharedString, reason: NoticeDismissal, cx: &mut Context<Self>) {
        let now = Instant::now();
        if self.manager.dismiss(id, now) {
            if let Some(notice) = self.manager.get(id) {
                Self::notify_dismissed(notice, reason, cx);
            }
            self.set_deadline(id.clone(), now + self.motion.exit_duration);
            self.schedule(cx);
            cx.notify();
        }
    }
    fn notify_dismissed(notice: &Notice, reason: NoticeDismissal, cx: &mut App) {
        if let Some(handler) = notice.on_dismiss.clone() {
            cx.defer(move |cx| handler(reason, cx));
        }
    }
    fn set_deadline(&mut self, id: SharedString, at: Instant) {
        self.deadlines.retain(|(other, _)| *other != id);
        self.deadlines.push((id, at));
    }
    /// Moves the manager's clock forward and records when it next needs to move.
    fn advance(&mut self, now: Instant, cx: &mut Context<Self>) {
        let paused = self.stack.is_expanded();
        let advance = self.manager.advance(now, paused);
        for id in advance.presented {
            match self.manager.get(&id).and_then(|notice| notice.timeout) {
                Some(timeout) => self.set_deadline(id, now + timeout),
                None => self.deadlines.retain(|(other, _)| *other != id),
            }
        }
        for id in advance.ending {
            if let Some(notice) = self.manager.get(&id) {
                Self::notify_dismissed(notice, NoticeDismissal::Timeout, cx);
            }
            self.set_deadline(id, now + self.motion.exit_duration);
        }
        for (id, _) in &advance.removed {
            self.deadlines.retain(|(other, _)| other != id);
        }
        if advance.changed {
            cx.notify();
        }
    }
    /// Sleeps until the next deadline, or polls while a hover pauses the timers.
    fn schedule(&mut self, cx: &mut Context<Self>) {
        let now = Instant::now();
        let next = self.deadlines.iter().map(|(_, at)| *at).min();
        let Some(next) = next else {
            self.timer = None;
            return;
        };
        let wait = if self.stack.is_expanded() {
            PAUSED_POLL
        } else {
            next.saturating_duration_since(now) + Duration::from_millis(1)
        };
        self.timer = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(wait).await;
            this.update(cx, |state, cx| {
                let now = Instant::now();
                if state.stack.is_expanded() {
                    // Shown toasts hold their remaining time while hovered; keep
                    // their deadlines moving with the clock.
                    let shown: Vec<SharedString> = state
                        .manager
                        .iter()
                        .filter(|(_, _, phase)| *phase == ToastTransitionStatus::Present)
                        .map(|(id, _, _)| id.clone())
                        .collect();
                    for (id, at) in &mut state.deadlines {
                        if shown.contains(id) {
                            *at += wait;
                        }
                    }
                }
                state.advance(now, cx);
                state.schedule(cx);
            })
            .ok();
        }));
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
        let (inset_x, inset_y) = self.inset.unwrap_or((theme.space(4.), theme.space(4.)));
        let width = (viewport.width - inset_x * 2.).min(theme.space(96.));
        let dir = theme.direction;
        let placement = self.placement.physical(dir);
        let close_label = locale::text(cx, UiText::CloseNotification);
        let mut stack = ToastStack::new("toasts", self.stack.clone())
            .placement(placement.anchor())
            .focus_handle(self.focus.clone())
            .absolute()
            .w(width);
        stack = match placement {
            ToastPlacement::TopLeft | ToastPlacement::BottomLeft => stack.left(inset_x),
            ToastPlacement::TopRight | ToastPlacement::BottomRight => stack.right(inset_x),
            ToastPlacement::TopCenter | ToastPlacement::BottomCenter => {
                stack.left((viewport.width - width) / 2.)
            }
        };
        stack = match placement {
            ToastPlacement::TopLeft | ToastPlacement::TopCenter | ToastPlacement::TopRight => {
                stack.top(inset_y)
            }
            _ => stack.bottom(inset_y),
        };
        for (id, notice, phase) in self.manager.visible(3) {
            let dismiss = id.clone();
            let state = cx.entity().downgrade();
            let button = |key: &'static str,
                          label: SharedString,
                          variant: ButtonVariant,
                          reason: NoticeDismissal,
                          handler: Option<ActionHandler>| {
                let dismiss = id.clone();
                let state = state.clone();
                Button::new(ElementId::from((ElementId::from(id.clone()), key)))
                    .variant(variant)
                    .size(ButtonSize::Xs)
                    .label(label)
                    .on_click(move |_, window, cx| {
                        if let Some(handler) = &handler {
                            handler(window, cx);
                        }
                        state
                            .update(cx, |state, cx| state.dismiss_with(&dismiss, reason, cx))
                            .ok();
                    })
            };
            // In tab order the buttons come before the close button.
            let buttons = (notice.action.is_some() || notice.cancel.is_some()).then(|| {
                div()
                    .flex_row_in(dir)
                    .items_center()
                    .gap(theme.space(2.))
                    .pt(theme.space(2.))
                    .children(notice.action.clone().map(|(label, handler)| {
                        button(
                            "action",
                            label,
                            ButtonVariant::Default,
                            NoticeDismissal::Action,
                            Some(handler),
                        )
                    }))
                    .children(notice.cancel.clone().map(|label| {
                        button(
                            "cancel",
                            label,
                            ButtonVariant::Outline,
                            NoticeDismissal::Cancel,
                            None,
                        )
                    }))
            });
            let close = Button::new(ElementId::from((ElementId::from(id.clone()), "close")))
                .aria_label(close_label.clone())
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .absolute()
                .top(theme.space(2.))
                .inset_end(dir, theme.space(2.))
                .on_click(move |_, _, cx| {
                    state
                        .update(cx, |state, cx| {
                            state.dismiss_with(&dismiss, NoticeDismissal::Close, cx)
                        })
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
                .w_full();
            let toast = surface::toast(toast, &theme)
                .p(theme.space(4.))
                .padding_end(dir, theme.space(12.))
                .flex()
                .flex_col()
                .gap(theme.space(1.))
                .opacity(presence.progress)
                .child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .child(notice.title.clone()),
                )
                .child(text::muted(div(), &theme).child(notice.description.clone()))
                .children(buttons)
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

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{AppContext as _, TestAppContext};

    fn phases(state: &ToastState) -> Vec<(SharedString, ToastTransitionStatus)> {
        state
            .manager
            .iter()
            .map(|(id, _, phase)| (id.clone(), phase))
            .collect()
    }

    #[gpui_kit::test]
    fn notices_present_time_out_and_leave_on_their_deadlines(cx: &mut TestAppContext) {
        cx.update(super::super::theme::init);
        let state = cx.update(|cx| cx.new(ToastState::new));
        let start = Instant::now();
        state.update(cx, |state, cx| {
            state.push(
                "saved",
                Notice::new("Saved").timeout(Duration::from_secs(2)),
                cx,
            );
            state.push("pinned", Notice::new("Pinned").persistent(), cx);
            assert_eq!(state.deadlines.len(), 2, "both wait for their entry motion");
            assert!(state.timer.is_some());

            state.advance(start + Duration::from_millis(401), cx);
            assert!(
                phases(state)
                    .iter()
                    .all(|(_, phase)| *phase == ToastTransitionStatus::Present)
            );
            // Only the timed notice still has a deadline.
            assert_eq!(state.deadlines.len(), 1);
            assert_eq!(state.deadlines[0].0, "saved");

            state.advance(start + Duration::from_millis(2402), cx);
            assert_eq!(
                phases(state),
                [
                    ("saved".into(), ToastTransitionStatus::Ending),
                    ("pinned".into(), ToastTransitionStatus::Present)
                ]
            );
            state.advance(start + Duration::from_millis(2603), cx);
            assert_eq!(
                phases(state),
                [("pinned".into(), ToastTransitionStatus::Present)]
            );
            assert!(state.deadlines.is_empty());

            state.dismiss(&"pinned".into(), cx);
            assert_eq!(state.deadlines.len(), 1);
            state.advance(start + Duration::from_millis(2804), cx);
            assert!(state.is_empty());
            state.schedule(cx);
            assert!(state.timer.is_none(), "nothing left to wait for");
        });
    }

    /// Tab from the stack reaches a notice's action before its close button,
    /// and pressing it runs the handler and dismisses the notice
    /// (devaryakjha/gpuicn#95).
    #[gpui_kit::test]
    fn an_action_runs_and_dismisses_its_notice(cx: &mut TestAppContext) {
        use gpui_kit::{KeyDownEvent, KeyUpEvent, Keystroke};
        cx.update(super::super::theme::init);
        let ran = Rc::new(std::cell::Cell::new(0));
        let count = ran.clone();
        let (state, cx) = cx.add_window_view(|window, cx| {
            window.activate_window();
            ToastState::new(cx)
        });
        state.update(cx, |state, cx| {
            state.push(
                "download",
                Notice::new("review-notes.pdf")
                    .description("Saved to Downloads")
                    .persistent()
                    .action("Show in folder", move |_, _| count.set(count.get() + 1))
                    .cancel("Dismiss"),
                cx,
            )
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.update(|window, cx| {
            let focus = state.read(cx).focus.clone();
            focus.focus(window, cx);
            window.draw(cx).clear(cx);
        });
        cx.simulate_keystrokes("tab");
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let keystroke = Keystroke::parse("enter").unwrap();
        cx.simulate_event(KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(KeyUpEvent { keystroke });
        assert_eq!(ran.get(), 1, "the action ran");
        state.update(cx, |state, _| {
            assert!(
                state
                    .manager
                    .iter()
                    .all(|(_, _, phase)| phase == ToastTransitionStatus::Ending),
                "the notice is leaving"
            );
        });
    }

    type Reasons = Rc<std::cell::RefCell<Vec<(&'static str, NoticeDismissal)>>>;

    fn record(reasons: &Reasons, id: &'static str) -> impl Fn(NoticeDismissal, &mut App) + use<> {
        let reasons = reasons.clone();
        move |reason, _| reasons.borrow_mut().push((id, reason))
    }

    /// The app hears why each notice left and can ask whether one is still
    /// showing (devaryakjha/gpuicn#122).
    #[gpui_kit::test]
    fn on_dismiss_reports_timeouts_replacements_and_dismissals(cx: &mut TestAppContext) {
        cx.update(super::super::theme::init);
        let reasons = Reasons::default();
        let state = cx.update(|cx| cx.new(ToastState::new));
        let start = Instant::now();
        state.update(cx, |state, cx| {
            state.push(
                "saved",
                Notice::new("Saved")
                    .timeout(Duration::from_secs(2))
                    .on_dismiss(record(&reasons, "saved")),
                cx,
            );
            state.push(
                "update",
                Notice::new("Update ready")
                    .persistent()
                    .on_dismiss(record(&reasons, "first update")),
                cx,
            );
            assert!(state.contains("saved") && state.contains("update"));
            assert!(!state.contains("other"));
            // Pushing the same ID again replaces the notice.
            state.push(
                "update",
                Notice::new("Update ready")
                    .persistent()
                    .on_dismiss(record(&reasons, "update")),
                cx,
            );
            state.advance(start + Duration::from_millis(401), cx);
            state.advance(start + Duration::from_millis(2402), cx);
            assert!(
                !state.contains("saved"),
                "a leaving notice no longer counts"
            );
            assert!(state.contains("update"));
            state.dismiss(&"update".into(), cx);
            assert!(!state.contains("update"));
            // A second dismissal of a leaving notice reports nothing.
            state.dismiss(&"update".into(), cx);
        });
        let reported = reasons.borrow().clone();
        assert_eq!(
            reported,
            [
                ("first update", NoticeDismissal::Replaced),
                ("saved", NoticeDismissal::Timeout),
                ("update", NoticeDismissal::Dismissed),
            ]
        );
    }

    /// "Later" and the close button report that the reader dismissed the
    /// notice, which the action does not (devaryakjha/gpuicn#122).
    #[gpui_kit::test]
    fn buttons_report_which_one_dismissed_the_notice(cx: &mut TestAppContext) {
        use gpui_kit::{KeyDownEvent, KeyUpEvent, Keystroke};
        cx.update(super::super::theme::init);
        let reasons = Reasons::default();
        let (state, cx) = cx.add_window_view(|window, cx| {
            window.activate_window();
            ToastState::new(cx)
        });
        let press = |cx: &mut gpui_kit::VisualTestContext, tabs: usize| {
            cx.update(|window, cx| {
                let focus = state.read(cx).focus.clone();
                focus.focus(window, cx);
                window.draw(cx).clear(cx);
            });
            for _ in 0..tabs {
                cx.simulate_keystrokes("tab");
                cx.update(|window, cx| window.draw(cx).clear(cx));
            }
            let keystroke = Keystroke::parse("enter").unwrap();
            cx.simulate_event(KeyDownEvent {
                keystroke: keystroke.clone(),
                is_held: false,
                prefer_character_input: false,
            });
            cx.simulate_event(KeyUpEvent { keystroke });
        };
        let notice = |id: &'static str| {
            Notice::new("Update ready")
                .persistent()
                .action("Restart to update", |_, _| {})
                .cancel("Later")
                .on_dismiss(record(&reasons, id))
        };
        // Tab order: action, cancel, close.
        for (id, tabs) in [("action", 1), ("later", 2), ("close", 3)] {
            state.update(cx, |state, cx| state.push(id, notice(id), cx));
            cx.update(|window, cx| window.draw(cx).clear(cx));
            press(cx, tabs);
            cx.run_until_parked();
            assert!(!state.read_with(cx, |state, _| state.contains(id)));
            // Let the notice leave so the next one is alone in the stack.
            state.update(cx, |state, cx| {
                state.advance(Instant::now() + Duration::from_secs(1), cx)
            });
        }
        let reported = reasons.borrow().clone();
        assert_eq!(
            reported,
            [
                ("action", NoticeDismissal::Action),
                ("later", NoticeDismissal::Cancel),
                ("close", NoticeDismissal::Close),
            ]
        );
        assert!(NoticeDismissal::Cancel.by_reader() && !NoticeDismissal::Action.by_reader());
    }
}
