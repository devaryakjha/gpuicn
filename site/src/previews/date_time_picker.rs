//! The date time picker catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn date_time_picker_preview(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        gpuicn::date_time_picker::DateTimePicker::new()
    }
}

pub(crate) fn preview(
    view: &Showcase,
    window: &mut Window,
    cx: &mut Context<Showcase>,
) -> gpui_kit::AnyElement {
    view.date_time_picker_preview(window, cx).into_any_element()
}
