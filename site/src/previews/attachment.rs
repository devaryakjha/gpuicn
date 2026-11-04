//! The attachment catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn attachment_preview(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let image = "https://raw.githubusercontent.com/devaryakjha/devaryakjha/6526e3d7415b2fb573ba3da4523b5c6948aa5d08/avatar.png";
        let removed = self.checked;
        div()
            .w_full()
            .max_w(px(540.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(8.))
                    .child(
                        Attachment::new("attachment.ready", "matty-plan.pdf")
                            .icon(LucideIcon::FileText)
                            .meta("PDF · 2.4 MB")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.demo_action = "Opened matty-plan.pdf".into();
                                cx.notify();
                            })),
                    )
                    .child(
                        Attachment::new("attachment.uploading", "recording.mov")
                            .icon(LucideIcon::FilePlay)
                            .status(AttachmentStatus::Uploading(0.64))
                            .on_remove(|_, _, _| {}),
                    )
                    .child(
                        Attachment::new("attachment.failed", "design.fig")
                            .status(AttachmentStatus::Failed)
                            .on_retry(cx.listener(|this, _, _, cx| {
                                this.demo_action = "Retrying design.fig".into();
                                cx.notify();
                            }))
                            .on_remove(|_, _, _| {}),
                    ),
            )
            .child(attachment_group(
                "attachment.images",
                [
                    Some(Attachment::new("attachment.image", "avatar.png").image(image)),
                    Some(
                        Attachment::new("attachment.image.sized", "avatar-large.png")
                            .image(image)
                            .dimensions(460, 460)
                            .max_size(px(120.), px(120.)),
                    ),
                    Some(
                        Attachment::new("attachment.image.uploading", "avatar-2.png")
                            .image(image)
                            .status(AttachmentStatus::Uploading(0.3)),
                    ),
                    (!removed).then(|| {
                        Attachment::new("attachment.image.removable", "avatar-3.png")
                            .image(image)
                            .on_remove(cx.listener(|this, _, _, cx| {
                                this.checked = true;
                                this.demo_action = "Removed avatar-3.png".into();
                                cx.notify();
                            }))
                    }),
                ]
                .into_iter()
                .flatten(),
                cx,
            ))
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.attachment_preview(window, cx).into_any_element()
}
