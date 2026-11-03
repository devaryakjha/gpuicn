//! The spinner catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn spinner_preview(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(16.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(16.0))
                    .child(Spinner::new("preview.spinner"))
                    .child(Spinner::new("preview.spinner.large").size(px(24.)))
                    .child(
                        Spinner::new("preview.spinner.primary")
                            .size(px(32.))
                            .color(theme.colors.primary),
                    ),
            )
            .child(
                Button::new("preview.spinner.button")
                    .disabled(true)
                    .child(
                        Spinner::new("preview.spinner.inline")
                            .label("Sending")
                            .color(theme.colors.primary_foreground),
                    )
                    .label("Sending…"),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .text_size(px(13.))
                    .text_color(theme.colors.muted_foreground)
                    .child(Spinner::new("preview.spinner.history").label("Loading older messages"))
                    .child("Loading older messages"),
            )
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.spinner_preview(window, cx).into_any_element()
}
