//! The alert catalog preview.

use crate::*;
use gpuicn::alert::{Alert, AlertVariant};

impl Showcase {
    pub(crate) fn alert_preview(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = UiTheme::read(cx);
        div()
            .w_full()
            .max_w(px(460.))
            .flex()
            .flex_col()
            .gap(theme.space(3.))
            .child(
                Alert::new("alert.error")
                    .variant(AlertVariant::Destructive)
                    .title("The integration refused this")
                    .description("Check the channel name and try again.")
                    .child(Button::new("alert.retry").size(ButtonSize::Sm).variant(ButtonVariant::Outline).label("Retry")),
            )
            .child(
                Alert::new("alert.warning")
                    .variant(AlertVariant::Warning)
                    .title("Guests can see this channel")
                    .description("Two guest accounts are members."),
            )
            .child(
                Alert::new("alert.info")
                    .variant(AlertVariant::Info)
                    .title("Theme sync is on")
                    .description("Matty follows the system appearance."),
            )
            .child(Alert::new("alert.success").variant(AlertVariant::Success).title("Settings saved"))
    }
}

pub(crate) fn preview(
    view: &Showcase,
    window: &mut Window,
    cx: &mut Context<Showcase>,
) -> gpui_kit::AnyElement {
    view.alert_preview(window, cx).into_any_element()
}
