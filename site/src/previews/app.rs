//! The app catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn app_preview(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        use gpuicn::Appearance;
        let theme = UiTheme::read(cx).clone();
        let current = UiTheme::appearance(cx);
        let choice = |label: &'static str, appearance: Appearance| {
            Button::new(SharedString::from(format!("app-appearance-{label}")))
                .variant(if current == appearance {
                    ButtonVariant::Secondary
                } else {
                    ButtonVariant::Ghost
                })
                .size(ButtonSize::Sm)
                .child(label)
                .on_click(move |_, _, cx| UiTheme::set_appearance(cx, appearance))
        };
        div()
            .w(px(420.))
            .rounded(theme.radius.lg)
            .border_1()
            .border_color(theme.colors.border)
            .bg(theme.colors.card)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px(px(16.))
                    .py(px(10.))
                    .border_b_1()
                    .border_color(theme.colors.border)
                    .child(
                        div()
                            .text_color(theme.colors.card_foreground)
                            .font_weight(gpui_kit::FontWeight::MEDIUM)
                            .child("My App"),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(4.))
                            .child(choice("Light", Appearance::Light))
                            .child(choice("Dark", Appearance::Dark))
                            .child(choice("System", Appearance::System)),
                    ),
            )
            .child(
                div()
                    .px(px(16.))
                    .py(px(20.))
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(
                        div()
                            .text_color(theme.colors.muted_foreground)
                            .child("app::run opens the window, loads Geist, installs the light and dark themes and follows the OS appearance."),
                    )
                    .child(Button::new("app-button").child("Count: 0")),
            )
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.app_preview(window, cx).into_any_element()
}
