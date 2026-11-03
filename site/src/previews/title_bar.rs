//! The title bar catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn title_bar_preview(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        div()
            .w_full()
            .max_w(px(560.))
            .rounded(theme.radius.xl)
            .border_1()
            .border_color(theme.colors.border)
            .overflow_hidden()
            .bg(theme.colors.background)
            .child(
                // GPUI clips children to a rectangle, so the bar takes the
                // frame's inner radius.
                TitleBar::new()
                    .rounded_t(theme.radius.xl - px(1.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.))
                            .children((0..3).map(|index| {
                                div()
                                    .size(px(12.))
                                    .rounded_full()
                                    .bg(theme.colors.border)
                                    .id(("traffic", index as usize))
                            })),
                    )
                    .child(
                        div()
                            .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                            .child("Matty"),
                    )
                    .child(div().flex_1())
                    .child(
                        Button::new("title-bar.search")
                            .variant(ButtonVariant::Outline)
                            .size(ButtonSize::Xs)
                            .w(px(200.))
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.))
                                    .text_color(theme.colors.muted_foreground)
                                    .child(
                                        lucide(LucideIcon::Search)
                                            .size(px(12.))
                                            .text_color(theme.colors.muted_foreground),
                                    )
                                    .child("Search"),
                            )
                            .child(Kbd::shortcut("secondary-k")),
                    )
                    .child(div().flex_1())
                    .child(
                        Avatar::new("title-bar.avatar")
                            .size(AvatarSize::Sm)
                            .fallback("AJ"),
                    )
                    .pr(px(12.)),
            )
            .child(
                div()
                    .h(px(120.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(13.))
                    .text_color(theme.colors.muted_foreground)
                    .child("Drag the bar to move the window; double-click to zoom."),
            )
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.title_bar_preview(window, cx).into_any_element()
}
