//! The context menu catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn context_menu_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let click = cx.listener(|this, _, _, cx| {
            this.demo_action = "Copied".into();
            cx.notify();
        });
        let menu = window.use_keyed_state("context-menu", cx, |_, cx| {
            MenuState::new(
                [
                    MenuItem::new("copy", "Copy").on_click(click),
                    MenuItem::new("paste", "Paste").disabled(true),
                    MenuItem::separator(),
                    MenuItem::new("show-bookmarks", "Show bookmarks").checked(true),
                    MenuItem::submenu(
                        "share",
                        "Share",
                        [MenuItem::new("documentation", "Documentation")
                            .link()
                            .on_click(|_, _, cx| cx.open_url("https://ui.imajha.com"))],
                    ),
                ],
                cx,
            )
        });
        gpuicn::context_menu::context_menu(
            &menu,
            "File actions",
            div()
                .w(px(280.))
                .h(px(160.))
                .border_1()
                .border_color(UiTheme::read(cx).colors.border)
                .rounded(px(8.))
                .flex()
                .items_center()
                .justify_center()
                .child("Right click here"),
        )
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.context_menu_preview(window, cx).into_any_element()
}
