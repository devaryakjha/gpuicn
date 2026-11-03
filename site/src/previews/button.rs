//! The button catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn button_preview(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let variants = [
            (ButtonVariant::Default, "Default"),
            (ButtonVariant::Secondary, "Secondary"),
            (ButtonVariant::Outline, "Outline"),
            (ButtonVariant::Ghost, "Ghost"),
            (ButtonVariant::Destructive, "Destructive"),
            (ButtonVariant::Link, "Link"),
        ];

        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(20.0))
            .child(
                Button::new("preview.button.interactive")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.count += 1;
                        cx.notify();
                    }))
                    .label(format!("Clicked {} times", self.count)),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .justify_center()
                    .gap(px(8.0))
                    .children(variants.into_iter().map(|(variant, label)| {
                        Button::new(format!("preview.button.{}", label.to_lowercase()))
                            .variant(variant)
                            .label(label)
                    })),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        Button::new("preview.button.small")
                            .size(ButtonSize::Xs)
                            .label("Extra small"),
                    )
                    .child(
                        Button::new("preview.button.large")
                            .size(ButtonSize::Lg)
                            .label("Large"),
                    )
                    .child(
                        Button::new("preview.button.icon")
                            .size(ButtonSize::Icon)
                            .variant(ButtonVariant::Outline)
                            .aria_label("Add one click")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.count += 1;
                                cx.notify();
                            }))
                            .child(
                                lucide(LucideIcon::Plus)
                                    .size(px(16.))
                                    .text_color(UiTheme::read(cx).colors.foreground),
                            ),
                    )
                    .child(
                        Button::new("preview.button.disabled")
                            .disabled(true)
                            .label("Disabled"),
                    ),
            )
    }
}

pub(crate) fn preview(
    view: &Showcase,
    window: &mut Window,
    cx: &mut Context<Showcase>,
) -> gpui_kit::AnyElement {
    view.button_preview(window, cx).into_any_element()
}
