//! The date picker catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn date_picker_preview(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        gpuicn::date_picker::DatePicker::new()
    }
}

pub(crate) fn preview(
    view: &Showcase,
    window: &mut Window,
    cx: &mut Context<Showcase>,
) -> gpui_kit::AnyElement {
    view.date_picker_preview(window, cx).into_any_element()
}
