//! The textarea catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn textarea_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bio = window.use_keyed_state("textarea.bio", cx, |window, cx| {
            TextareaState::new(window, cx)
                .rows(5)
                .placeholder("Tell us about your work")
        });

        div().w(px(420.)).child(
            Field::from_control("textarea.bio.field", textarea(&bio))
                .label("Bio")
                .description("A short introduction for your profile."),
        )
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.textarea_preview(window, cx).into_any_element()
}
