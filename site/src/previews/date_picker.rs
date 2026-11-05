//! The date picker catalog preview.

use crate::*;
use gpuicn::calendar::{CalendarDate, CalendarState};
use gpuicn::date_picker::DatePicker;

impl Showcase {
    pub(crate) fn date_picker_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let empty = window.use_keyed_state("date-picker.empty", cx, |_, cx| CalendarState::new(cx));
        let picked = window.use_keyed_state("date-picker.picked", cx, |_, cx| {
            let mut state = CalendarState::new(cx);
            state.set_selected(CalendarDate::new(2026, 3, 14), cx);
            state
        });
        let theme = UiTheme::read(cx);
        div()
            .w(px(260.))
            .flex()
            .flex_col()
            .gap(theme.space(3.))
            .child(DatePicker::new("date-picker.due", &empty).aria_label("Due date").w_full())
            .child(DatePicker::new("date-picker.start", &picked).aria_label("Start date").w_full())
    }
}

pub(crate) fn preview(
    view: &Showcase,
    window: &mut Window,
    cx: &mut Context<Showcase>,
) -> gpui_kit::AnyElement {
    view.date_picker_preview(window, cx).into_any_element()
}
