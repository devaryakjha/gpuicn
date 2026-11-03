//! The accordion catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn accordion_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let expanded = window.use_keyed_state("faq-open", cx, |_, _| 0_usize);
        let selected = *expanded.read(cx);
        accordion("preview.accordion", cx).w(px(480.)).children(
            [
                (
                    "Is it accessible?",
                    "Use Tab and Enter to open each section.",
                ),
                (
                    "Is it styled?",
                    "Yes. It follows the shadcn Nova visual system.",
                ),
                (
                    "Is it native?",
                    "Yes. GPUI Kit supplies the native controls.",
                ),
            ]
            .into_iter()
            .enumerate()
            .map(|(i, (title, text))| {
                let expanded = expanded.clone();
                accordion_item(cx)
                    .open(selected == i)
                    .header(accordion_header(
                        accordion_trigger(("faq", i), selected == i, false, cx)
                            .child(title)
                            .on_click(move |_, _, cx| {
                                expanded.update(cx, |selected, cx| {
                                    *selected = if *selected == i { usize::MAX } else { i };
                                    cx.notify();
                                })
                            }),
                    ))
                    .panel(accordion_content(
                        ("faq-content", i),
                        selected == i,
                        text,
                        window,
                        cx,
                    ))
            }),
        )
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.accordion_preview(window, cx).into_any_element()
}
