//! The toast catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn toast_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = window.use_keyed_state("notifications", cx, |_, cx| ToastState::new(cx));
        let trigger = state.clone();
        let sequence = window.use_keyed_state("notification-sequence", cx, |_, _| 0_usize);
        div()
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
                                    .description("Your settings are up to date."),
                                cx,
                            )
                        })
                    }),
            )
            .child(state)
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.toast_preview(window, cx).into_any_element()
}
