//! The calendar catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn calendar_preview(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        gpuicn::calendar::Calendar::new()
    }
}

pub(crate) fn preview(
    view: &Showcase,
    window: &mut Window,
    cx: &mut Context<Showcase>,
) -> gpui_kit::AnyElement {
    view.calendar_preview(window, cx).into_any_element()
}
