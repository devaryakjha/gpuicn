//! Nova file and image attachment cards with upload progress, retry and remove.

use std::rc::Rc;

use gpui_icons::{LucideIcon, lucide};
use gpui_kit::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, ImageSource, InteractiveElement as _,
    IntoElement, ObjectFit, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, StyledImage as _, Window, div, img,
    prelude::FluentBuilder as _, px, relative,
};

use super::button::{Button, ButtonSize, ButtonVariant};
use super::theme::{UiTheme, apply_style};

type Handler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// Where an attachment is in its upload.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum AttachmentStatus {
    #[default]
    /// Uploaded, or a received file.
    Ready,
    /// Uploading, with progress from 0.0 to 1.0.
    Uploading(f32),
    /// The upload failed; a Retry button shows when `on_retry` is set.
    Failed,
}

/// A file card, or an image thumbnail when `image` is set.
#[derive(IntoElement)]
pub struct Attachment {
    style: gpui_kit::StyleRefinement,
    id: ElementId,
    name: SharedString,
    meta: Option<SharedString>,
    icon: LucideIcon,
    image: Option<ImageSource>,
    status: AttachmentStatus,
    on_click: Option<Handler>,
    on_remove: Option<Handler>,
    on_retry: Option<Handler>,
}

impl Attachment {
    /// Creates an attachment with a stable caller-owned ID and its file name.
    pub fn new(id: impl Into<ElementId>, name: impl Into<SharedString>) -> Self {
        Self {
            style: gpui_kit::StyleRefinement::default(),
            id: id.into(),
            name: name.into(),
            meta: None,
            icon: LucideIcon::File,
            image: None,
            status: AttachmentStatus::Ready,
            on_click: None,
            on_remove: None,
            on_retry: None,
        }
    }
    /// Sets the secondary line, such as "PDF · 2.4 MB".
    pub fn meta(mut self, meta: impl Into<SharedString>) -> Self {
        self.meta = Some(meta.into());
        self
    }
    /// Sets the file-type icon. Defaults to `File`.
    pub fn icon(mut self, icon: LucideIcon) -> Self {
        self.icon = icon;
        self
    }
    /// Shows an image thumbnail instead of a file card.
    pub fn image(mut self, source: impl Into<ImageSource>) -> Self {
        self.image = Some(source.into());
        self
    }
    /// Sets the upload state.
    pub fn status(mut self, status: AttachmentStatus) -> Self {
        self.status = status;
        self
    }
    /// Opens or previews the attachment on click or Enter.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
    /// Adds a remove button, announced as "Remove" plus the file name.
    pub fn on_remove(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_remove = Some(Rc::new(handler));
        self
    }
    /// Adds a Retry button while the upload has failed.
    pub fn on_retry(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_retry = Some(Rc::new(handler));
        self
    }
}

impl Styled for Attachment {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}

fn icon_button(
    id: ElementId,
    icon: LucideIcon,
    label: SharedString,
    handler: Handler,
    theme: &UiTheme,
) -> Button {
    Button::new(id)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::IconXs)
        .aria_label(label)
        .on_click(move |event, window, cx| handler(event, window, cx))
        .child(
            lucide(icon)
                .size(theme.space(3.5))
                .text_color(theme.colors.muted_foreground),
        )
}

impl RenderOnce for Attachment {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let failed = self.status == AttachmentStatus::Failed;
        let progress = match self.status {
            AttachmentStatus::Uploading(progress) => Some(progress.clamp(0., 1.)),
            _ => None,
        };
        let status_text: Option<SharedString> = match self.status {
            AttachmentStatus::Uploading(progress) => {
                Some(format!("Uploading… {}%", (progress.clamp(0., 1.) * 100.).round()).into())
            }
            AttachmentStatus::Failed => Some("Upload failed".into()),
            AttachmentStatus::Ready => None,
        };
        let description = status_text.clone().or(self.meta.clone());
        let accessible: SharedString = match &description {
            Some(description) => format!("{}, {}", self.name, description).into(),
            None => self.name.clone(),
        };
        let remove = self.on_remove.clone().map(|handler| {
            icon_button(
                (self.id.clone(), "remove").into(),
                LucideIcon::X,
                format!("Remove {}", self.name).into(),
                handler,
                &theme,
            )
        });
        let retry = self.on_retry.clone().filter(|_| failed).map(|handler| {
            icon_button(
                (self.id.clone(), "retry").into(),
                LucideIcon::RotateCw,
                format!("Retry uploading {}", self.name).into(),
                handler,
                &theme,
            )
        });
        let progress_bar = progress.map(|progress| {
            div()
                .absolute()
                .left_0()
                .right_0()
                .bottom_0()
                .h(px(2.))
                .bg(colors.muted)
                .child(div().h_full().w(relative(progress)).bg(colors.primary))
        });

        let root = div()
            .id(self.id.clone())
            .aria_label(accessible)
            .relative()
            .flex_shrink_0()
            .overflow_hidden()
            .rounded(theme.radius.lg)
            .border_1()
            .border_color(if failed {
                colors.destructive.opacity(0.5)
            } else {
                colors.border
            })
            .bg(colors.card)
            .text_color(colors.card_foreground)
            .font_family(theme.fonts.body.clone())
            .when_some(self.on_click.clone(), |root, handler| {
                root.cursor_pointer()
                    .hover(|root| root.bg(colors.muted.opacity(0.5)))
                    .on_click(move |event, window, cx| handler(event, window, cx))
            });

        let root: AnyElement = match self.image {
            Some(source) => root
                .size(theme.space(16.))
                .child(
                    img(source)
                        .size_full()
                        .object_fit(ObjectFit::Cover)
                        .when(progress.is_some() || failed, |image| image.opacity(0.5)),
                )
                .when(failed, |root| {
                    root.child(
                        div()
                            .absolute()
                            .inset_0()
                            .flex()
                            .items_center()
                            .justify_center()
                            .bg(colors.background.opacity(0.6))
                            .children(retry.map(|retry| retry.bg(colors.background)))
                            .when(self.on_retry.is_none(), |overlay| {
                                overlay.child(
                                    lucide(LucideIcon::CircleAlert)
                                        .size(theme.space(5.))
                                        .text_color(colors.destructive),
                                )
                            }),
                    )
                })
                .children(remove.map(|remove| {
                    div()
                        .absolute()
                        .top(theme.space(1.))
                        .right(theme.space(1.))
                        .rounded_md()
                        .bg(colors.background.opacity(0.9))
                        .child(remove)
                }))
                .children(progress_bar)
                .into_any_element(),
            None => root
                .w(theme.space(60.))
                .h(theme.space(14.))
                .flex()
                .items_center()
                .gap(theme.space(3.))
                .pl(theme.space(2.))
                .pr(theme.space(1.5))
                .child(
                    div()
                        .size(theme.space(10.))
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(theme.radius.md)
                        .bg(if failed {
                            colors.destructive.opacity(0.1)
                        } else {
                            colors.muted
                        })
                        .child(
                            lucide(self.icon)
                                .size(theme.space(5.))
                                .text_color(if failed {
                                    colors.destructive
                                } else {
                                    colors.muted_foreground
                                }),
                        ),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .gap(theme.space(0.5))
                        .child(
                            div()
                                .text_size(theme.text(14.))
                                .font_weight(FontWeight::MEDIUM)
                                .overflow_hidden()
                                .text_ellipsis()
                                .whitespace_nowrap()
                                .child(self.name.clone()),
                        )
                        .children(description.map(|description| {
                            div()
                                .text_size(theme.text(12.))
                                .text_color(if failed {
                                    colors.destructive
                                } else {
                                    colors.muted_foreground
                                })
                                .overflow_hidden()
                                .text_ellipsis()
                                .whitespace_nowrap()
                                .child(description)
                        })),
                )
                .children(retry)
                .children(remove)
                .children(progress_bar)
                .into_any_element(),
        };
        apply_style(div().flex_shrink_0().child(root), &self.style)
    }
}

/// A horizontally scrolling row of attachments, such as a composer's upload tray.
pub fn attachment_group(
    id: impl Into<ElementId>,
    attachments: impl IntoIterator<Item = Attachment>,
    cx: &App,
) -> gpui_kit::Stateful<gpui_kit::Div> {
    let theme = UiTheme::read(cx);
    div()
        .id(id)
        .w_full()
        .flex()
        .items_end()
        .gap(theme.space(2.))
        .overflow_x_scroll()
        .children(attachments)
}
