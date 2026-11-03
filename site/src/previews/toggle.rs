//! The toggle catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn toggle_preview(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity().downgrade();
        let icon_color = UiTheme::read(cx).colors.foreground;
        div()
            .flex()
            .items_center()
            .gap(px(8.0))
            .child(
                Toggle::new("preview.toggle")
                    .pressed(self.pressed)
                    .variant(ToggleVariant::Outline)
                    .aria_label("Bold")
                    .on_change(move |pressed, _, _, cx| {
                        view.update(cx, |this, cx| {
                            this.pressed = pressed;
                            cx.notify();
                        })
                        .ok();
                    })
                    .child(
                        lucide(LucideIcon::Bold)
                            .size(px(16.0))
                            .text_color(icon_color),
                    ),
            )
            .child(
                Toggle::new("preview.toggle.pressed")
                    .pressed(true)
                    .aria_label("Italic")
                    .child(
                        lucide(LucideIcon::Italic)
                            .size(px(16.0))
                            .text_color(icon_color),
                    ),
            )
            .child(
                Toggle::new("preview.toggle.disabled")
                    .disabled(true)
                    .aria_label("Underline")
                    .child(
                        lucide(LucideIcon::Underline)
                            .size(px(16.0))
                            .text_color(icon_color),
                    ),
            )
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.toggle_preview(window, cx).into_any_element()
}
