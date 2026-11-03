//! The otp field catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn otp_field_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = window.use_keyed_state("otp", cx, |window, cx| OtpState::new(6, window, cx));
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child("Enter the six-digit code")
            .child(OtpField::new(&state).aria_label("Verification code"))
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.otp_field_preview(window, cx).into_any_element()
}
