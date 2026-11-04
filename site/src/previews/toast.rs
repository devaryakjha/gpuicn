//! The toast catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn toast_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = window.use_keyed_state("notifications", cx, |_, cx| ToastState::new(cx));
        let trigger = state.clone();
        let sequence = window.use_keyed_state("notification-sequence", cx, |_, _| 0_usize);
        let updater = state.clone();
        let last = window.use_keyed_state("notification-dismissal", cx, |_, _| SharedString::default());
        let heard = last.clone();
        let showing = state.read(cx).contains("update");
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(
                div()
                    .flex()
                    .gap(px(8.))
                    .child(
                Button::new("notify")
                    .variant(ButtonVariant::Outline)
                    .label("Show notification")
                    .on_click(move |_, _, cx| {
                        let id = sequence.update(cx, |next, _| {
                            *next += 1;
                            format!("saved-{}", *next)
                        });
                        trigger.update(cx, |state, cx| {
                            state.push(
                                id,
                                Notice::new("Changes saved")
                                    .description("Your settings are up to date.")
                                    .action("Undo", |_, _| {})
                                    .cancel("Dismiss"),
                                cx,
                            )
                        })
                    }),
            )
                    .child(
                        Button::new("notify-update")
                            .variant(ButtonVariant::Outline)
                            .label("Show update prompt")
                            .disabled(showing)
                            .on_click(move |_, _, cx| {
                                let heard = heard.clone();
                                updater.update(cx, |state, cx| {
                                    state.push(
                                        "update",
                                        Notice::new("Update ready")
                                            .description("Restart to finish installing 0.9.5.")
                                            .persistent()
                                            .action("Restart to update", |_, _| {})
                                            .cancel("Later")
                                            .on_dismiss(move |reason, cx| {
                                                heard.update(cx, |last, cx| {
                                                    *last = format!("Last dismissal: {reason:?}").into();
                                                    cx.notify();
                                                })
                                            }),
                                        cx,
                                    )
                                })
                            }),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(UiTheme::read(cx).colors.muted_foreground)
                    .child(last.read(cx).clone()),
            )
            .child(state)
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.toast_preview(window, cx).into_any_element()
}
