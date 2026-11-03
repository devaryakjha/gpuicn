//! The skeleton catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn skeleton_preview(&self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let message = |id: &'static str, lines: &'static [f32]| {
            div()
                .w_full()
                .flex()
                .gap(px(12.))
                .child(Skeleton::new((id, 100usize)).size(px(32.)).rounded_full())
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .gap(px(8.))
                        .child(Skeleton::new((id, 101usize)).w(px(96.)).h(px(12.)))
                        .children(lines.iter().enumerate().map(move |(index, width)| {
                            Skeleton::new((id, index)).w(gpui_kit::relative(*width))
                        })),
                )
        };
        div()
            .w_full()
            .max_w(px(360.0))
            .flex()
            .flex_col()
            .gap(px(20.0))
            .child(message("preview.skeleton.first", &[1., 0.6]))
            .child(message("preview.skeleton.second", &[0.8]))
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.skeleton_preview(window, cx).into_any_element()
}
