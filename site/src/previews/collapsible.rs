//! The collapsible catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn collapsible_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let project = |name: &'static str| {
            div()
                .w_full()
                .rounded(theme.radius.lg)
                .border_1()
                .border_color(theme.colors.border)
                .px(theme.space(4.))
                .py(theme.space(2.5))
                .font_family(theme.fonts.mono.clone())
                .text_size(theme.text(13.))
                .child(name)
        };
        div().w(px(340.)).h(px(240.)).child(
            collapsible("preview.projects", self.collapsible_open, window, cx)
                .gap(theme.space(2.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .font_weight(gpui_kit::FontWeight::MEDIUM)
                                .child("Recent projects"),
                        )
                        .child(
                            collapsible_trigger("preview.disclosure", self.collapsible_open, cx)
                                .accessibility_label("Toggle recent projects")
                                .child(
                                    lucide(LucideIcon::ChevronsUpDown)
                                        .size(theme.space(4.))
                                        .text_color(theme.colors.foreground),
                                )
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.collapsible_open = !this.collapsible_open;
                                    cx.notify();
                                })),
                        ),
                )
                .child(project("Design system"))
                .content(
                    collapsible_content(cx)
                        .pt(px(0.))
                        .flex()
                        .flex_col()
                        .gap(theme.space(2.))
                        .child(project("Documentation"))
                        .child(project("Website")),
                ),
        )
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.collapsible_preview(window, cx).into_any_element()
}
