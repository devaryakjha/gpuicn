//! The toolbar catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn toolbar_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let input = demo_input("toolbar.search", "", "Find…", window, cx);
        Toolbar::new("tools", "Formatting", cx)
            .button(
                ToolbarButton::new("bold", "Bold", cx)
                    .child("Bold")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.demo_action = "Bold".into();
                        cx.notify();
                    })),
            )
            .button(ToolbarButton::new("italic", "Italic", cx).child("Italic"))
            .separator()
            .input(&input, "Find text", false)
            .button(ToolbarButton::new("more", "More options", cx).child("More"))
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.toolbar_preview(window, cx).into_any_element()
}
