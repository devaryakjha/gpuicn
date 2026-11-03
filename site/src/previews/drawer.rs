//! The drawer catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn drawer_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let handle = window
            .use_keyed_state("drawer.handle", cx, |_, _| DialogHandle::new(false))
            .read(cx)
            .clone();
        let side = self.drawer_direction;
        let theme = UiTheme::read(cx).clone();
        let header = drawer_header(side, cx)
            .child(drawer_title("drawer.title", cx).child("Move goal"))
            .child(
                drawer_description("drawer.description", cx).child("Set your daily activity goal."),
            );
        let body = drawer_body("drawer.body", cx).pt_0().child(
            div()
                .mx_auto()
                .w_full()
                .max_w(theme.space(80.))
                .flex()
                .flex_col()
                .gap(theme.space(5.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap(theme.space(4.))
                        .child(
                            Button::new("goal-minus")
                                .variant(ButtonVariant::Outline)
                                .label("−")
                                .aria_label("Decrease daily goal")
                                .rounded_full()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.goal = (this.goal - 10).max(10);
                                    cx.notify();
                                })),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .child(
                                    div()
                                        .text_size(theme.text(48.))
                                        .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                                        .child(self.goal.to_string()),
                                )
                                .child(
                                    div()
                                        .text_color(theme.colors.muted_foreground)
                                        .text_size(theme.text(11.))
                                        .child("CALORIES / DAY"),
                                ),
                        )
                        .child(
                            Button::new("goal-plus")
                                .variant(ButtonVariant::Outline)
                                .label("+")
                                .aria_label("Increase daily goal")
                                .rounded_full()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.goal += 10;
                                    cx.notify();
                                })),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_end()
                        .gap(theme.space(1.5))
                        .h(theme.space(16.))
                        .children(
                            [40., 65., 45., 85., 55., 95., 70., 60., 90., 75., 50., 80.]
                                .into_iter()
                                .map(|height| {
                                    div()
                                        .flex_1()
                                        .h(theme.space(16.) * (height / 100.))
                                        .rounded_t(theme.radius.sm)
                                        .bg(theme.colors.foreground)
                                }),
                        ),
                ),
        );
        let footer = drawer_footer(cx)
            .mx_auto()
            .w_full()
            .max_w(theme.space(88.))
            .child(
                drawer_close("goal-save", &handle)
                    .variant(ButtonVariant::Default)
                    .label("Save goal"),
            )
            .child(drawer_close("goal-cancel", &handle).label("Cancel"));
        Drawer::new("drawer", &handle)
            .direction(side)
            .show_swipe_handle(true)
            .flex()
            .flex_col()
            .items_center()
            .gap(theme.space(4.))
            .child(drawer_trigger("drawer.trigger", &handle).label("Open drawer"))
            .child(
                div().flex().gap(theme.space(2.)).children(
                    [
                        ("top", DrawerSide::Top),
                        ("right", DrawerSide::Right),
                        ("bottom", DrawerSide::Bottom),
                        ("left", DrawerSide::Left),
                    ]
                    .into_iter()
                    .map(|(label, direction)| {
                        Button::new(label)
                            .variant(if side == direction {
                                ButtonVariant::Secondary
                            } else {
                                ButtonVariant::Ghost
                            })
                            .label(label)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.drawer_direction = direction;
                                cx.notify();
                            }))
                    }),
                ),
            )
            .content(
                DrawerContent::new("drawer.content", "Move goal")
                    .child(header)
                    .child(body)
                    .child(footer),
            )
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.drawer_preview(window, cx).into_any_element()
}
