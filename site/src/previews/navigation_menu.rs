//! The navigation menu catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn navigation_menu_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let getting_started = window.use_keyed_state("nav.start", cx, |_, cx| {
            MenuState::new(
                [
                    MenuItem::new("introduction", "Introduction")
                        .link()
                        .on_click(|_, _, cx| cx.open_url("https://ui.imajha.com/docs")),
                    MenuItem::new("installation", "Installation")
                        .link()
                        .on_click(|_, _, cx| {
                            cx.open_url("https://ui.imajha.com/docs/installation")
                        }),
                ],
                cx,
            )
        });
        let components = window.use_keyed_state("nav.components", cx, |_, cx| {
            MenuState::new(
                [
                    MenuItem::new("button", "Button")
                        .link()
                        .on_click(|_, _, cx| {
                            cx.open_url("https://ui.imajha.com/docs/components/button")
                        }),
                    MenuItem::new("dialog", "Dialog")
                        .link()
                        .on_click(|_, _, cx| {
                            cx.open_url("https://ui.imajha.com/docs/components/dialog")
                        }),
                ],
                cx,
            )
        });
        navigation_menu("documentation-nav", "Documentation")
            .menu(&getting_started, "Getting started")
            .menu(&components, "Components")
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.navigation_menu_preview(window, cx).into_any_element()
}
