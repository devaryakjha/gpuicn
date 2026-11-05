//! Nova file and image attachment cards with upload progress, retry and remove.

use std::rc::Rc;

use gpui_icons::{LucideIcon, lucide};
use gpui_kit::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, ImageSource, InteractiveElement as _,
    IntoElement, ObjectFit, ParentElement, Pixels, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, Window, div, prelude::FluentBuilder as _, px,
    relative,
};

use super::button::{Button, ButtonSize, ButtonVariant};
use super::theme::{
    UiTheme, UiText, apply_style,
    locale::{self, DirectionalStyled as _},
    primitives::{clipped_image, disabled},
};

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
    dimensions: Option<(f32, f32)>,
    max_size: Option<(Pixels, Pixels)>,
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
            dimensions: None,
            max_size: None,
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
    /// Sizes the image thumbnail from the file's pixel size, such as the
    /// width and height in the server's file metadata. The box keeps the
    /// image's aspect ratio, scaled down to fit `max_size`, and holds that size
    /// while the image loads so a transcript does not jump. Without it, the
    /// thumbnail is a 64px square.
    pub fn dimensions(mut self, width: u32, height: u32) -> Self {
        self.dimensions = (width > 0 && height > 0).then_some((width as f32, height as f32));
        self
    }
    /// Caps a thumbnail sized by `dimensions`. Defaults to 360 × 240 at the
    /// default spacing scale.
    pub fn max_size(mut self, width: impl Into<Pixels>, height: impl Into<Pixels>) -> Self {
        self.max_size = Some((width.into(), height.into()));
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

/// The thumbnail box for an image of `width` × `height` pixels: its aspect
/// ratio, scaled down (never up) to fit the cap.
fn fit_within(width: f32, height: f32, max: (Pixels, Pixels)) -> (Pixels, Pixels) {
    let scale = (f32::from(max.0) / width)
        .min(f32::from(max.1) / height)
        .min(1.);
    (px((width * scale).round()), px((height * scale).round()))
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
        let dir = theme.direction;
        let failed = self.status == AttachmentStatus::Failed;
        let progress = match self.status {
            AttachmentStatus::Uploading(progress) => Some(progress.clamp(0., 1.)),
            _ => None,
        };
        let status_text: Option<SharedString> = match self.status {
            AttachmentStatus::Uploading(progress) => Some(locale::text(
                cx,
                UiText::Uploading((progress.clamp(0., 1.) * 100.).round() as u32),
            )),
            AttachmentStatus::Failed => Some(locale::text(cx, UiText::UploadFailed)),
            AttachmentStatus::Ready => None,
        };
        let description = status_text.clone().or(self.meta.clone());
        let accessible: SharedString = match &description {
            Some(description) => {
                format!("{}, {}", locale::isolate(&self.name), description).into()
            }
            None => self.name.clone(),
        };
        let remove = self.on_remove.clone().map(|handler| {
            icon_button(
                (self.id.clone(), "remove").into(),
                LucideIcon::X,
                locale::text(cx, UiText::Remove(locale::isolate(&self.name))),
                handler,
                &theme,
            )
        });
        let retry = self.on_retry.clone().filter(|_| failed).map(|handler| {
            icon_button(
                (self.id.clone(), "retry").into(),
                LucideIcon::RotateCw,
                locale::text(cx, UiText::RetryUpload(locale::isolate(&self.name))),
                handler,
                &theme,
            )
        });
        // GPUI clips children to a rectangle, not to the card's rounded
        // corners, so the bar stays clear of them and images carry the inner radius.
        let inner_radius = theme.radius.lg - px(1.);
        let is_image = self.image.is_some();
        let progress_bar = progress.map(|progress| {
            let (inset, bottom, height) = if is_image {
                (theme.space(1.5), theme.space(1.5), px(3.))
            } else {
                (theme.radius.lg, px(0.), px(2.))
            };
            div()
                .absolute()
                .left(inset)
                .right(inset)
                .bottom(bottom)
                .h(height)
                .rounded_full()
                .overflow_hidden()
                .bg(colors.muted)
                .flex_row_in(dir)
                .child(
                    div()
                        .h_full()
                        .w(relative(progress))
                        .rounded_full()
                        .bg(colors.primary),
                )
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

        let sized = self.dimensions.map(|(width, height)| {
            let max = self
                .max_size
                .unwrap_or((theme.space(90.), theme.space(60.)));
            fit_within(width, height, max)
        });
        let root: AnyElement = match self.image {
            Some(source) => root
                .map(|root| match sized {
                    // The muted fill shows until the bytes arrive.
                    Some((width, height)) => root.w(width).h(height).bg(colors.muted),
                    None => root.size(theme.space(16.)),
                })
                .child(disabled::dim(
                    clipped_image(
                        source,
                        inner_radius,
                        if sized.is_some() {
                            ObjectFit::Contain
                        } else {
                            ObjectFit::Cover
                        },
                        &theme,
                    )
                    .size_full(),
                    progress.is_some() || failed,
                ))
                .when(failed, |root| {
                    root.child(
                        div()
                            .absolute()
                            .inset_0()
                            .rounded(inner_radius)
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
                        .inset_end(dir, theme.space(1.))
                        .rounded_md()
                        .bg(colors.background.opacity(0.9))
                        .child(remove)
                }))
                .children(progress_bar)
                .into_any_element(),
            None => root
                .w(theme.space(60.))
                .h(theme.space(14.))
                .flex_row_in(dir)
                .items_center()
                .gap(theme.space(3.))
                .padding_start(dir, theme.space(2.))
                .padding_end(dir, theme.space(1.5))
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
    let mut attachments: Vec<_> = attachments.into_iter().collect();
    // A reversed row would overflow past the scroll origin, so right to
    // left reverses the children instead.
    if theme.direction.is_rtl() {
        attachments.reverse();
    }
    div()
        .id(id)
        .w_full()
        .flex()
        .items_end()
        .gap(theme.space(2.))
        .overflow_x_scroll()
        .children(attachments)
}

#[cfg(test)]
mod tests {
    use super::fit_within;
    use gpui_kit::px;

    #[test]
    fn thumbnails_keep_the_aspect_ratio_inside_the_cap() {
        let cap = (px(360.), px(240.));
        // A wide screenshot is limited by the width, a tall one by the height.
        assert_eq!(fit_within(1920., 1080., cap), (px(360.), px(203.)));
        assert_eq!(fit_within(1080., 1920., cap), (px(135.), px(240.)));
        // Small images keep their own size.
        assert_eq!(fit_within(120., 80., cap), (px(120.), px(80.)));
    }
}
