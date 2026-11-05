//! The avatar group catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn avatar_group_preview(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        gpuicn::avatar_group::AvatarGroup::new()
    }
}

pub(crate) fn preview(
    view: &Showcase,
    window: &mut Window,
    cx: &mut Context<Showcase>,
) -> gpui_kit::AnyElement {
    view.avatar_group_preview(window, cx).into_any_element()
}
