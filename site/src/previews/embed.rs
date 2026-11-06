//! The embed catalog preview.

use crate::*;
use gpuicn::embed::{ImagePlaceholder, LinkPreview, MessageEmbed, QuotedMessage};

impl Showcase {
    pub(crate) fn embed_preview(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let image = "https://raw.githubusercontent.com/devaryakjha/devaryakjha/6526e3d7415b2fb573ba3da4523b5c6948aa5d08/avatar.png";
        let collapsed = self.checked;
        // The attachment's image waits for a click, as with "Load remote
        // images: ask", and then loads into the same box.
        let image_loaded = self.pressed;
        div()
            .w_full()
            .max_w(px(480.))
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(
                LinkPreview::new("embed.link", "karyalabs/gpuicn: shadcn-style components for GPUI")
                    .site_name("GitHub")
                    .description(
                        "Copy-in components for GPUI applications: dialogs, menus, chat \
                         messages and more, themed from one set of tokens.",
                    )
                    .image(image, 460, 460)
                    .collapsed(collapsed)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.demo_action = "Opened github.com/karyalabs/gpuicn".into();
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
            .child({
                let embed = MessageEmbed::new("embed.attachment")
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
                    .footer("gpuicn")
                    .footer_icon(image)
                    .timestamp("Today at 10:42")
                    .action(
                        Button::new("embed.action.approve")
                            .size(ButtonSize::Sm)
                            .tint(gpui_kit::rgb(0x3f51b5))
                            .label("Approve")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.demo_action = "Approved the release".into();
                                cx.notify();
                            })),
                    )
                    .action(
                        Button::new("embed.action.logs")
                            .size(ButtonSize::Sm)
                            .variant(ButtonVariant::Outline)
                            .label("View logs"),
                    );
                if image_loaded {
                    embed.image(image, 460, 460)
                } else {
                    embed.image_placeholder(
                        ImagePlaceholder::new(
                            "raw.githubusercontent.com",
                            cx.listener(|this, _, _, cx| {
                                this.pressed = true;
                                this.demo_action = "Loaded the image".into();
                                cx.notify();
                            }),
                        )
                        .size(460, 460),
                    )
                }
            })
            .child(
                QuotedMessage::new("embed.quote", "Margaret Hamilton")
                    .avatar(image)
                    .initials("MH")
                    .timestamp("Yesterday at 4:12 PM")
                    .channel("~town-square")
                    .body(
                        "The landing software flagged an overflow, and the computer shed its \
                         lowest-priority work to keep guidance running. That was the design: the \
                         scheduler restarted the jobs that mattered, and the crew carried on with \
                         the descent while Houston confirmed the alarm was safe to ignore.",
                    )
                    .on_jump(cx.listener(|this, _, _, cx| {
                        this.demo_action = "Jumped to the original message".into();
                        cx.notify();
                    })),
            )
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.embed_preview(window, cx).into_any_element()
}
