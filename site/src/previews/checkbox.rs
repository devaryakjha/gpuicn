//! The checkbox catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn checkbox_preview(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let checkbox_view = cx.entity().downgrade();
        let label_view = cx.entity().downgrade();
        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        Checkbox::new("preview.checkbox.interactive")
                            .checked(self.checked)
                            .aria_label("Accept terms")
                            .on_change(move |checked, _, _, cx| {
                                checkbox_view
                                    .update(cx, |this, cx| {
                                        this.checked = checked;
                                        cx.notify();
                                    })
                                    .ok();
                            }),
                    )
                    .child(
                        div()
                            .id("preview.checkbox.interactive-label")
                            .cursor_pointer()
                            .on_click(move |_, _, cx| {
                                label_view
                                    .update(cx, |this, cx| {
                                        this.checked = !this.checked;
                                        cx.notify();
                                    })
                                    .ok();
                            })
                            .child(if self.checked {
                                "Accepted"
                            } else {
                                "Accept terms"
                            }),
                    ),
            )
            .child(checkbox_row(
                Checkbox::new("preview.checkbox.indeterminate")
                    .indeterminate(true)
                    .aria_label("Partly selected"),
                "Partly selected",
            ))
            .child(checkbox_row(
                Checkbox::new("preview.checkbox.disabled")
                    .checked(true)
                    .disabled(true)
                    .aria_label("Disabled"),
                "Disabled",
            ))
            .child(checkbox_row(
                Checkbox::new("preview.checkbox.readonly")
                    .checked(true)
                    .read_only(true)
                    .aria_label("Read only"),
                "Read only",
            ))
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.checkbox_preview(window, cx).into_any_element()
}
