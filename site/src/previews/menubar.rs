//! The menubar catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn menubar_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let file = window.use_keyed_state("menubar.file", cx, |_, cx| {
            MenuState::new(
                [
                    MenuItem::new("new-tab", "New tab"),
                    MenuItem::new("new-window", "New window"),
                    MenuItem::separator(),
                    MenuItem::new("quit", "Quit"),
                ],
                cx,
            )
        });
        let edit = window.use_keyed_state("menubar.edit", cx, |_, cx| {
            MenuState::new(
                [
                    MenuItem::new("undo", "Undo"),
                    MenuItem::new("redo", "Redo").disabled(true),
                    MenuItem::new("copy", "Copy"),
                ],
                cx,
            )
        });
        Menubar::new("app-menu", "Application menu")
            .menu(&file, "File")
            .menu(&edit, "Edit")
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.menubar_preview(window, cx).into_any_element()
}
