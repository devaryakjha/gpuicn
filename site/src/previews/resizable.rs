//! The resizable catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn resizable_preview(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        use gpuicn::resizable::{PaneLimits, Resizable};
        div().w(px(400.)).h(px(200.)).child(
            Resizable::new("preview.split", "Resize panels", self.pane_width)
                .first(div().p(px(16.)).child("One"))
                .second(div().p(px(16.)).child("Two"))
                .on_resize(cx.listener(|this, size: &gpui_kit::Pixels, _, cx| {
                    this.pane_width = *size;
                    cx.notify();
                }))
                .first_limits(PaneLimits::new(px(80.), px(300.)))
                .second_limits(PaneLimits::new(px(80.), px(300.))),
        )
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.resizable_preview(window, cx).into_any_element()
}
