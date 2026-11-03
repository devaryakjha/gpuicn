//! The alert dialog catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn alert_dialog_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let handle = window
            .use_keyed_state("alert_dialog_preview.handle", cx, |_, _| {
                DialogHandle::new(false)
            })
            .read(cx)
            .clone();
        let popup = dialog_popup("alert_dialog_preview.popup", "Are you sure?", cx)
            .child(dialog_title("alert_dialog_preview.title", cx).child("Are you sure?"))
            .child(
                dialog_description("alert_dialog_preview.description", cx)
                    .child("This action cannot be undone."),
            )
            .child(
                dialog_footer(cx)
                    .child(alert_dialog_cancel("cancel").label("Cancel"))
                    .child(alert_dialog_action("confirm").label("Continue")),
            );
        div()
            .child(dialog_trigger("alert_dialog_preview.trigger", &handle).label("Delete account"))
            .child(alert_dialog(
                "alert_dialog_preview",
                &handle,
                popup,
                window,
                cx,
            ))
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.alert_dialog_preview(window, cx).into_any_element()
}
