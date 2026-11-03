//! The dialog catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn dialog_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let handle = window
            .use_keyed_state("dialog_preview.handle", cx, |_, _| DialogHandle::new(false))
            .read(cx)
            .clone();
        let name = demo_input("dialog_preview.name", "Ada Lovelace", "Name", window, cx);
        let popup = dialog_popup("dialog_preview.popup", "Edit profile", cx)
            .child(dialog_title("dialog_preview.title", cx).child("Edit profile"))
            .child(
                dialog_description("dialog_preview.description", cx)
                    .child("Update the name shown on your profile."),
            )
            .child(Input::new(&name).aria_label("Name"))
            .child(dialog_footer(cx).child(dialog_action("save", &handle).label("Save changes")));
        div()
            .child(dialog_trigger("dialog_preview.trigger", &handle).label("Edit profile"))
            .child(dialog("dialog_preview", &handle, popup, window, cx))
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.dialog_preview(window, cx).into_any_element()
}
