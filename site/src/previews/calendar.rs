//! The calendar catalog preview.

use crate::*;
use gpuicn::calendar::{Calendar, CalendarDate, CalendarState};

impl Showcase {
    pub(crate) fn calendar_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = window.use_keyed_state("calendar.state", cx, |_, cx| {
            let mut state = CalendarState::new(cx);
            let today = CalendarDate::new(2026, 3, 14).unwrap();
            state.set_today(today, cx);
            state.set_selected(Some(today.add_days(3)), cx);
            state
        });
        let theme = UiTheme::read(cx);
        div()
            .rounded(theme.radius.lg)
            .border_1()
            .border_color(theme.colors.border)
            .bg(theme.colors.card)
            .child(Calendar::new(&state))
    }
}

pub(crate) fn preview(
    view: &Showcase,
    window: &mut Window,
    cx: &mut Context<Showcase>,
) -> gpui_kit::AnyElement {
    view.calendar_preview(window, cx).into_any_element()
}
