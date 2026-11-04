//! The embed catalog preview.

use crate::*;
use gpuicn::embed::{LinkPreview, MessageEmbed};

impl Showcase {
    pub(crate) fn embed_preview(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let image = "https://raw.githubusercontent.com/devaryakjha/devaryakjha/6526e3d7415b2fb573ba3da4523b5c6948aa5d08/avatar.png";
        let collapsed = self.checked;
        div()
            .w_full()
            .max_w(px(480.))
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(
                LinkPreview::new("embed.link", "devaryakjha/gpuicn: shadcn-style components for GPUI")
                    .site_name("GitHub")
                    .description(
                        "Copy-in components for GPUI applications: dialogs, menus, chat \
                         messages and more, themed from one set of tokens.",
                    )
                    .image(image, 460, 460)
                    .collapsed(collapsed)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.demo_action = "Opened github.com/devaryakjha/gpuicn".into();
                        cx.notify();
                    }))
                    .on_toggle(cx.listener(|this, collapsed: &bool, _, cx| {
                        this.checked = *collapsed;
                        this.demo_action =
                            if *collapsed { "Collapsed the preview" } else { "Expanded the preview" }.into();
                        cx.notify();
                    }))
                    .on_remove(cx.listener(|this, _, _, cx| {
                        this.demo_action = "Removed the preview".into();
                        cx.notify();
                    })),
            )
            .child(
                MessageEmbed::new("embed.attachment")
                    .color_str("good")
                    .pretext("Release pipeline finished")
                    .author("GitHub Actions")
                    .author_icon(image)
                    .title("release · v0.9.2")
                    .on_title_click(cx.listener(|this, _, _, cx| {
                        this.demo_action = "Opened the release run".into();
                        cx.notify();
                    }))
                    .body("Published the crates, the desktop apps and the website.")
                    .field("Branch", "main", true)
                    .field("Duration", "6m 41s", true)
                    .field("Commit", "Embed cards and scrolling dialogs", false)
                    .footer("gpuicn")
                    .footer_icon(image)
                    .timestamp("Today at 10:42"),
            )
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.embed_preview(window, cx).into_any_element()
}
