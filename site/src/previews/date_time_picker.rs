//! The date and time picker catalog preview.

use crate::*;
use gpuicn::calendar::CalendarDate;
use gpuicn::date_time_picker::{CalendarTime, DateTimePicker, DateTimeState, HourCycle};

impl Showcase {
    pub(crate) fn date_time_picker_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = window.use_keyed_state("date-time-picker.state", cx, |_, cx| {
            let mut state = DateTimeState::new(cx);
            state.set_hour_cycle(HourCycle::for_locale("en-US"), cx);
            state.set_value(CalendarDate::new(2026, 3, 16), CalendarTime::new(9, 30).unwrap(), cx);
            state
        });
        div()
            .w(px(280.))
            .child(DateTimePicker::new("date-time-picker.send", &state).aria_label("Send at").w_full())
    }
}

pub(crate) fn preview(
    view: &Showcase,
    window: &mut Window,
    cx: &mut Context<Showcase>,
) -> gpui_kit::AnyElement {
    view.date_time_picker_preview(window, cx).into_any_element()
}
