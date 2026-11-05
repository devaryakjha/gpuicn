//! The alert catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn alert_preview(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        gpuicn::alert::Alert::new()
    }
}

pub(crate) fn preview(
    view: &Showcase,
    window: &mut Window,
    cx: &mut Context<Showcase>,
) -> gpui_kit::AnyElement {
    view.alert_preview(window, cx).into_any_element()
}
