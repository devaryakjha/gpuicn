//! The markdown catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn markdown_preview(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let image = "https://raw.githubusercontent.com/devaryakjha/devaryakjha/6526e3d7415b2fb573ba3da4523b5c6948aa5d08/avatar.png";
        let source = "**Release day.** gpuicn `v0.5.0` is tagged, thanks [@arya](mention:arya) ![:arya:](emoji:arya) and [@sam](mention:sam). Notes are in [~releases](channel:releases).\n\n> Matty pins a tagged gpuicn release before beta.\n\n- Message rows and reactions\n- A *virtualized* transcript\n- ~~Bubbles~~ left-aligned team chat\n\n```rust\nlet list = MessageList::new(\"channel\", &state, render_row);\n```\n\n| Component | Status |\n| --- | --- |\n| Command | Ready |\n| Markdown | Ready |\n\n![Arya's avatar](avatar.png)";
        selection_root("markdown.selection")
            .flex()
            .items_center()
            .justify_center()
            .child(
                div().w_full().max_w(px(520.)).child(
                    gpuicn::markdown::markdown("preview.markdown", source, cx)
                        .plugin(EntityLinks::new("mention").highlight(|user| user == "arya"))
                        .plugin(EntityLinks::new("channel"))
                        .plugin(CustomEmoji::new("emoji", move |name, _, _| {
                            (name == "arya").then(|| image.into())
                        }))
                        .plugin(MarkdownImages::new(move |url, _, _| {
                            (url == "avatar.png").then(|| image.into())
                        })),
                ),
            )
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.markdown_preview(window, cx).into_any_element()
}
