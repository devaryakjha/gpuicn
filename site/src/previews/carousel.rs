//! The carousel catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn carousel_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let horizontal = window.use_keyed_state("carousel.horizontal", cx, |_, _| {
            CarouselState::new(3).with_looping(true)
        });
        let vertical = window.use_keyed_state("carousel.vertical", cx, |_, _| {
            CarouselState::new(3).with_axis(gpui_kit::Axis::Vertical)
        });
        let theme = UiTheme::read(cx).clone();
        let colors = [
            theme.colors.chart_1,
            theme.colors.chart_2,
            theme.colors.chart_3,
        ];
        let card = move |index: usize| {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .rounded(theme.radius.xl)
                .border_1()
                .border_color(theme.colors.border)
                .bg(colors[index].opacity(0.16))
                .font_weight(gpui_kit::FontWeight::MEDIUM)
                .text_size(theme.text(24.))
                .child(format!("Slide {}", index + 1))
        };
        let vertical_theme = UiTheme::read(cx).clone();
        let vertical_colors = [
            vertical_theme.colors.chart_3,
            vertical_theme.colors.chart_4,
            vertical_theme.colors.chart_5,
        ];
        let vertical_card = move |index: usize| {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .rounded(vertical_theme.radius.xl)
                .border_1()
                .border_color(vertical_theme.colors.border)
                .bg(vertical_colors[index].opacity(0.16))
                .font_weight(gpui_kit::FontWeight::MEDIUM)
                .child(format!("{} / 3", index + 1))
        };

        div()
            .flex()
            .items_center()
            .gap(px(72.))
            .child(
                Carousel::new("carousel.horizontal", &horizontal)
                    .aria_label("Featured components")
                    .w(px(300.))
                    .child(
                        CarouselContent::new(&horizontal)
                            .h(px(180.))
                            .children((0..3).map(|index| {
                                CarouselItem::new(
                                    ("carousel.horizontal.slide", index),
                                    index,
                                    &horizontal,
                                )
                                .child(card(index))
                            })),
                    )
                    .child(CarouselPrevious::new(&horizontal))
                    .child(CarouselNext::new(&horizontal))
                    .child(CarouselPagination::new().children((0..3).map(|index| {
                        CarouselPaginationItem::new(
                            ("carousel.horizontal.page", index),
                            index,
                            &horizontal,
                        )
                    }))),
            )
            .child(
                Carousel::new("carousel.vertical", &vertical)
                    .aria_label("Vertical component tour")
                    .w(px(160.))
                    .child(
                        CarouselContent::new(&vertical)
                            .h(px(180.))
                            .children((0..3).map(|index| {
                                CarouselItem::new(
                                    ("carousel.vertical.slide", index),
                                    index,
                                    &vertical,
                                )
                                .child(vertical_card(index))
                            })),
                    )
                    .child(CarouselPrevious::new(&vertical))
                    .child(CarouselNext::new(&vertical)),
            )
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.carousel_preview(window, cx).into_any_element()
}
