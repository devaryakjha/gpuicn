//! The switch catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn switch_preview(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let switch_view = cx.entity().downgrade();
        let row_view = cx.entity().downgrade();
        div()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(switch_row(
                Switch::new("preview.switch")
                    .checked(self.checked)
                    .aria_label("Airplane mode")
                    .on_change(move |checked, _, _, cx| {
                        switch_view
                            .update(cx, |this, cx| {
                                this.checked = checked;
                                cx.notify();
                            })
                            .ok();
                    }),
                div()
                    .id("preview.switch.interactive-label")
                    .debug_selector(|| "switch-label".into())
                    .cursor_pointer()
                    .on_click(move |_, _, cx| {
                        row_view
                            .update(cx, |this, cx| {
                                this.checked = !this.checked;
                                cx.notify();
                            })
                            .ok();
                    })
                    .child(if self.checked {
                        "Airplane mode on"
                    } else {
                        "Airplane mode"
                    }),
            ))
            .child(switch_row(
                Switch::new("preview.switch.checked")
                    .checked(true)
                    .aria_label("Notifications"),
                "Notifications",
            ))
            .child(switch_row(
                Switch::new("preview.switch.disabled")
                    .checked(true)
                    .disabled(true)
                    .aria_label("Disabled setting"),
                "Disabled",
            ))
            .child(switch_row(
                Switch::new("preview.switch.read-only")
                    .checked(true)
                    .read_only(true)
                    .aria_label("Read-only setting"),
                "Read only",
            ))
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.switch_preview(window, cx).into_any_element()
}
