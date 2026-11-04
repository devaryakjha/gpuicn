//! Nova embed cards under a chat message: link previews (Open Graph) and
//! message attachments (the integration format with a colored bar, fields and
//! a footer).
//!
//! shadcn/ui has no embed card; both cards are drawn on the card tokens with
//! the same density steps as `Message`. Titles that open something are GPUI
//! Kit buttons with the link role, so they take focus and activate on Enter.
//! gpuicn never fetches: images arrive as `ImageSource` with their pixel size
//! so the card reserves its box before the bytes load.

use std::rc::Rc;

use gpui_icons::{LucideIcon, lucide};
use gpui_kit::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, ImageSource, InteractiveElement as _,
    IntoElement, ObjectFit, ParentElement, Pixels, RenderOnce, Rgba, Role, SharedString,
    StatefulInteractiveElement as _, Styled, Window, div, prelude::FluentBuilder as _, px,
    relative,
};

use super::button::{Button, ButtonSize, ButtonVariant};
pub use super::message::MessageDensity;
use super::theme::{
    UiTheme, apply_style,
    primitives::{clipped_image, focus, round_image, text},
};

type Handler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
type ToggleHandler = Rc<dyn Fn(&bool, &mut Window, &mut App) + 'static>;

/// Images narrower than this, in source pixels, show as a thumbnail beside
/// the text instead of a large image below it, as in Mattermost's webapp.
pub const LARGE_IMAGE_MIN_WIDTH: u32 = 150;

/// Where a link preview draws its image.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LinkPreviewImageLayout {
    #[default]
    /// A thumbnail when the image is narrower than [`LARGE_IMAGE_MIN_WIDTH`]
    /// or roughly square (or taller than wide); a large image otherwise.
    Auto,
    /// The full-width image below the text.
    Large,
    /// A small square on the right of the text.
    Thumbnail,
}

impl LinkPreviewImageLayout {
    /// The layout `Auto` resolves to for an image of `width` × `height` pixels.
    pub fn resolve(self, width: u32, height: u32) -> Self {
        match self {
            Self::Auto => {
                let square = height > 0 && (width as f32) < height as f32 * 1.2;
                if width < LARGE_IMAGE_MIN_WIDTH || square {
                    Self::Thumbnail
                } else {
                    Self::Large
                }
            }
            layout => layout,
        }
    }
}

#[derive(Clone)]
struct SizedImage {
    source: ImageSource,
    width: u32,
    height: u32,
}

/// Density-aware inner padding and spacing, matching `Message`.
fn metrics(theme: &UiTheme, density: MessageDensity) -> (Pixels, Pixels) {
    match density {
        MessageDensity::Standard => (theme.space(3.), theme.space(1.5)),
        MessageDensity::Compact => (theme.space(2.), theme.space(1.)),
    }
}

/// The box for an image of `width` × `height` pixels: its aspect ratio, scaled
/// down (never up) to fit the cap.
fn fit_within(width: f32, height: f32, max: (Pixels, Pixels)) -> (Pixels, Pixels) {
    let scale = (f32::from(max.0) / width)
        .min(f32::from(max.1) / height)
        .min(1.);
    (px((width * scale).round()), px((height * scale).round()))
}

/// A large image whose box is reserved from its pixel size. The width also
/// shrinks with a narrow card, and the aspect ratio keeps the height in step.
fn large_image(
    id: ElementId,
    image: SizedImage,
    on_click: Option<Handler>,
    theme: &UiTheme,
) -> AnyElement {
    let (width, height) = (image.width.max(1) as f32, image.height.max(1) as f32);
    let (box_width, _) = fit_within(width, height, (theme.space(114.), theme.space(72.)));
    div()
        .id(id)
        .debug_selector(|| "embed-image".into())
        .w(box_width)
        .max_w_full()
        .aspect_ratio(width / height)
        .flex_shrink_0()
        .child(clipped_image(image.source, theme.radius.md, ObjectFit::Cover, theme).size_full())
        .when_some(on_click, |frame, handler| {
            frame
                .cursor_pointer()
                .on_click(move |event, window, cx| handler(event, window, cx))
        })
        .into_any_element()
}

/// A small square image beside the text.
fn thumbnail(
    id: ElementId,
    source: ImageSource,
    on_click: Option<Handler>,
    theme: &UiTheme,
) -> AnyElement {
    div()
        .id(id)
        .debug_selector(|| "embed-thumbnail".into())
        .flex_shrink_0()
        .child(
            clipped_image(source, theme.radius.md, ObjectFit::Cover, theme).size(theme.space(20.)),
        )
        .when_some(on_click, |frame, handler| {
            frame
                .cursor_pointer()
                .on_click(move |event, window, cx| handler(event, window, cx))
        })
        .into_any_element()
}

/// A title clamped to `lines`; a focusable link when `on_click` is set.
fn title_text(
    id: ElementId,
    title: SharedString,
    lines: usize,
    on_click: Option<Handler>,
    theme: &UiTheme,
) -> AnyElement {
    match on_click {
        Some(handler) => {
            let theme = theme.clone();
            Button::new(id)
                .aria_label(title.clone())
                .style_with(move |base| {
                    let focus_theme = theme.clone();
                    let base = base.role(Role::Link);
                    text::sm(base, &theme)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.colors.primary)
                        .line_clamp(lines)
                        .rounded(theme.radius.sm)
                        // A block, so the title wraps at the card's width
                        // and clamps instead of growing past it.
                        .block()
                        .w_full()
                        .text_left()
                        .cursor_pointer()
                        .hover(|style| style.underline())
                        .focus_visible(move |style| focus::ring(style, &focus_theme))
                })
                .on_click(move |event, window, cx| handler(event, window, cx))
                .child(title)
                .into_any_element()
        }
        None => text::sm(div().id(id), theme)
            .font_weight(FontWeight::SEMIBOLD)
            .line_clamp(lines)
            .child(title)
            .into_any_element(),
    }
}

fn icon_button(
    id: ElementId,
    icon: LucideIcon,
    label: &'static str,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    theme: &UiTheme,
) -> Button {
    Button::new(id)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::IconXs)
        .aria_label(label)
        .on_click(on_click)
        .child(
            lucide(icon)
                .size(theme.space(3.5))
                .text_color(theme.colors.muted_foreground),
        )
}

/// The card frame both embeds share.
fn card_frame(
    id: ElementId,
    label: SharedString,
    theme: &UiTheme,
) -> gpui_kit::Stateful<gpui_kit::Div> {
    text::sm(div().id(id), theme)
        .role(Role::Group)
        .aria_label(label)
        .w_full()
        .max_w(theme.space(120.))
        .min_w_0()
        .text_color(theme.colors.card_foreground)
}

/// An Open Graph link preview: site name, linked title, description and an
/// optional image, with collapse and remove controls.
#[derive(IntoElement)]
pub struct LinkPreview {
    style: gpui_kit::StyleRefinement,
    id: ElementId,
    title: SharedString,
    site_name: Option<SharedString>,
    description: Option<SharedString>,
    image: Option<SizedImage>,
    image_layout: LinkPreviewImageLayout,
    collapsed: bool,
    density: MessageDensity,
    on_click: Option<Handler>,
    on_toggle: Option<ToggleHandler>,
    on_remove: Option<Handler>,
}

impl LinkPreview {
    /// Creates a preview with a stable caller-owned ID and the page title,
    /// which also names the card for assistive technology.
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            style: gpui_kit::StyleRefinement::default(),
            id: id.into(),
            title: title.into(),
            site_name: None,
            description: None,
            image: None,
            image_layout: LinkPreviewImageLayout::Auto,
            collapsed: false,
            density: MessageDensity::Standard,
            on_click: None,
            on_toggle: None,
            on_remove: None,
        }
    }
    /// Sets the small muted line above the title, such as "GitHub".
    pub fn site_name(mut self, site_name: impl Into<SharedString>) -> Self {
        self.site_name = Some(site_name.into());
        self
    }
    /// Sets the description, clamped to three lines.
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }
    /// Adds the page image with its pixel size from the server's metadata, so
    /// the card reserves the image's box before it loads.
    pub fn image(mut self, source: impl Into<ImageSource>, width: u32, height: u32) -> Self {
        self.image = Some(SizedImage {
            source: source.into(),
            width,
            height,
        });
        self
    }
    /// Chooses between a large image and a thumbnail. Defaults to `Auto`.
    pub fn image_layout(mut self, layout: LinkPreviewImageLayout) -> Self {
        self.image_layout = layout;
        self
    }
    /// Hides the image, as Mattermost's per-post "collapse preview". The
    /// text stays.
    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }
    /// Matches the padding to the message list's density.
    pub fn density(mut self, density: MessageDensity) -> Self {
        self.density = density;
        self
    }
    /// Runs when the title or the image is activated; open the link here.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
    /// Adds a chevron that collapses or expands the image, named "Collapse
    /// preview" or "Expand preview". Receives the new collapsed state; the
    /// caller stores it and passes it back through `collapsed`. Shown only
    /// when the preview has an image.
    pub fn on_toggle(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(handler));
        self
    }
    /// Adds a "Remove preview" button, for the post's author.
    pub fn on_remove(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_remove = Some(Rc::new(handler));
        self
    }
}

impl Styled for LinkPreview {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for LinkPreview {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let (padding, gap) = metrics(&theme, self.density);
        let id = self.id;
        let collapsed = self.collapsed;

        let toggle = self
            .on_toggle
            .filter(|_| self.image.is_some())
            .map(|handler| {
                let (icon, label) = if collapsed {
                    (LucideIcon::ChevronRight, "Expand preview")
                } else {
                    (LucideIcon::ChevronDown, "Collapse preview")
                };
                icon_button(
                    (id.clone(), "toggle").into(),
                    icon,
                    label,
                    move |_, window, cx| handler(&!collapsed, window, cx),
                    &theme,
                )
            });
        let remove = self.on_remove.map(|handler| {
            icon_button(
                (id.clone(), "remove").into(),
                LucideIcon::X,
                "Remove preview",
                move |event, window, cx| handler(event, window, cx),
                &theme,
            )
        });
        let header =
            (self.site_name.is_some() || toggle.is_some() || remove.is_some()).then(|| {
                div()
                    .flex()
                    .items_center()
                    .gap(theme.space(1.))
                    .min_h(theme.space(5.))
                    .child(
                        text::xs(div(), &theme)
                            .flex_1()
                            .min_w_0()
                            .text_color(colors.muted_foreground)
                            .overflow_hidden()
                            .text_ellipsis()
                            .whitespace_nowrap()
                            .children(self.site_name),
                    )
                    .children(
                        toggle.map(|toggle| {
                            div().debug_selector(|| "embed-toggle".into()).child(toggle)
                        }),
                    )
                    .children(remove)
            });

        let (large, thumb) = match self.image.filter(|_| !collapsed) {
            Some(image) => match self.image_layout.resolve(image.width, image.height) {
                LinkPreviewImageLayout::Thumbnail => (
                    None,
                    Some(thumbnail(
                        (id.clone(), "thumbnail").into(),
                        image.source,
                        self.on_click.clone(),
                        &theme,
                    )),
                ),
                _ => (
                    Some(large_image(
                        (id.clone(), "image").into(),
                        image,
                        self.on_click.clone(),
                        &theme,
                    )),
                    None,
                ),
            },
            None => (None, None),
        };

        let card = card_frame(id.clone(), self.title.clone(), &theme)
            .flex()
            .flex_col()
            .gap(gap)
            .p(padding)
            .rounded(theme.radius.lg)
            .border_1()
            .border_color(colors.border)
            .bg(colors.card)
            .children(header)
            .child(
                div()
                    .flex()
                    .items_start()
                    .gap(theme.space(3.))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap(theme.space(1.))
                            .child(title_text(
                                (id.clone(), "title").into(),
                                self.title,
                                2,
                                self.on_click,
                                &theme,
                            ))
                            .children(self.description.map(|description| {
                                text::muted(div(), &theme).line_clamp(3).child(description)
                            })),
                    )
                    .children(thumb),
            )
            .children(large);
        apply_style(card, &self.style)
    }
}

#[derive(Clone, Copy)]
enum EmbedColor {
    Custom(Rgba),
    Good,
    Warning,
    Danger,
}

struct EmbedField {
    label: SharedString,
    value: AnyElement,
    short: bool,
}

/// A message attachment card, Mattermost's and Slack's integration format: a
/// colored bar, then pretext, author, title, body, fields, image and footer.
/// Every part is optional.
#[derive(IntoElement)]
pub struct MessageEmbed {
    style: gpui_kit::StyleRefinement,
    id: ElementId,
    color: Option<EmbedColor>,
    pretext: Option<AnyElement>,
    author: Option<SharedString>,
    author_icon: Option<ImageSource>,
    on_author_click: Option<Handler>,
    title: Option<SharedString>,
    on_title_click: Option<Handler>,
    body: Option<AnyElement>,
    fields: Vec<EmbedField>,
    image: Option<SizedImage>,
    on_image_click: Option<Handler>,
    thumbnail: Option<ImageSource>,
    footer: Option<SharedString>,
    footer_icon: Option<ImageSource>,
    timestamp: Option<SharedString>,
    density: MessageDensity,
}

impl MessageEmbed {
    /// Creates an empty card with a stable caller-owned ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            style: gpui_kit::StyleRefinement::default(),
            id: id.into(),
            color: None,
            pretext: None,
            author: None,
            author_icon: None,
            on_author_click: None,
            title: None,
            on_title_click: None,
            body: None,
            fields: Vec::new(),
            image: None,
            on_image_click: None,
            thumbnail: None,
            footer: None,
            footer_icon: None,
            timestamp: None,
            density: MessageDensity::Standard,
        }
    }
    /// Colors the bar. Defaults to the border color.
    pub fn color(mut self, color: impl Into<Rgba>) -> Self {
        self.color = Some(EmbedColor::Custom(color.into()));
        self
    }
    /// Colors the bar from the integration's string: `#rgb`, `#rrggbb` (the
    /// `#` is optional), or `good`, `warning` and `danger`, which map to the
    /// theme's status colors. Anything else keeps the default.
    pub fn color_str(mut self, color: &str) -> Self {
        let color = color.trim();
        self.color = match color.to_ascii_lowercase().as_str() {
            "good" => Some(EmbedColor::Good),
            "warning" => Some(EmbedColor::Warning),
            "danger" => Some(EmbedColor::Danger),
            _ => {
                let hex = color.strip_prefix('#').unwrap_or(color);
                let valid =
                    matches!(hex.len(), 3 | 6) && hex.chars().all(|c| c.is_ascii_hexdigit());
                valid
                    .then(|| Rgba::try_from(format!("#{hex}").as_str()).ok())
                    .flatten()
                    .map(EmbedColor::Custom)
                    .or(self.color)
            }
        };
        self
    }
    /// Content above the card, such as rendered Markdown pretext.
    pub fn pretext(mut self, pretext: impl IntoElement) -> Self {
        self.pretext = Some(pretext.into_any_element());
        self
    }
    /// Sets the author line's name.
    pub fn author(mut self, name: impl Into<SharedString>) -> Self {
        self.author = Some(name.into());
        self
    }
    /// Sets the small round icon before the author's name.
    pub fn author_icon(mut self, source: impl Into<ImageSource>) -> Self {
        self.author_icon = Some(source.into());
        self
    }
    /// Makes the author's name a link.
    pub fn on_author_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_author_click = Some(Rc::new(handler));
        self
    }
    /// Sets the title, which also names the card for assistive technology.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }
    /// Makes the title a link.
    pub fn on_title_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_title_click = Some(Rc::new(handler));
        self
    }
    /// The card's text; render the attachment's Markdown into it.
    pub fn body(mut self, body: impl IntoElement) -> Self {
        self.body = Some(body.into_any_element());
        self
    }
    /// Adds a field. Short fields sit two to a row; a long one takes the row.
    pub fn field(
        mut self,
        label: impl Into<SharedString>,
        value: impl IntoElement,
        short: bool,
    ) -> Self {
        self.fields.push(EmbedField {
            label: label.into(),
            value: value.into_any_element(),
            short,
        });
        self
    }
    /// Adds an image below the fields with its pixel size, so the card
    /// reserves the image's box before it loads.
    pub fn image(mut self, source: impl Into<ImageSource>, width: u32, height: u32) -> Self {
        self.image = Some(SizedImage {
            source: source.into(),
            width,
            height,
        });
        self
    }
    /// Runs when the image is clicked, such as to open a preview.
    pub fn on_image_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_image_click = Some(Rc::new(handler));
        self
    }
    /// Adds a small square image on the right of the card.
    pub fn thumbnail(mut self, source: impl Into<ImageSource>) -> Self {
        self.thumbnail = Some(source.into());
        self
    }
    /// Sets the footer text.
    pub fn footer(mut self, footer: impl Into<SharedString>) -> Self {
        self.footer = Some(footer.into());
        self
    }
    /// Sets the small round icon before the footer.
    pub fn footer_icon(mut self, source: impl Into<ImageSource>) -> Self {
        self.footer_icon = Some(source.into());
        self
    }
    /// Sets the footer's time, already formatted by the caller.
    pub fn timestamp(mut self, timestamp: impl Into<SharedString>) -> Self {
        self.timestamp = Some(timestamp.into());
        self
    }
    /// Matches the padding to the message list's density.
    pub fn density(mut self, density: MessageDensity) -> Self {
        self.density = density;
        self
    }
}

impl Styled for MessageEmbed {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for MessageEmbed {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let (padding, gap) = metrics(&theme, self.density);
        let id = self.id;
        let bar = match self.color {
            Some(EmbedColor::Custom(color)) => color,
            Some(EmbedColor::Good) => colors.success,
            Some(EmbedColor::Warning) => colors.warning,
            Some(EmbedColor::Danger) => colors.destructive,
            None => colors.border,
        };
        let label = self
            .title
            .clone()
            .or_else(|| self.author.clone())
            .unwrap_or_else(|| "Attachment".into());
        let small_icon = theme.space(4.);

        let author = self.author.map(|name| {
            div()
                .flex()
                .items_center()
                .gap(theme.space(1.5))
                .children(
                    self.author_icon
                        .map(|source| round_image(source, small_icon, &theme)),
                )
                .child(match self.on_author_click {
                    Some(handler) => {
                        let theme = theme.clone();
                        Button::new((id.clone(), "author"))
                            .aria_label(name.clone())
                            .style_with(move |base| {
                                let focus_theme = theme.clone();
                                text::xs(base.role(Role::Link), &theme)
                                    .font_weight(FontWeight::MEDIUM)
                                    .rounded(theme.radius.sm)
                                    .justify_start()
                                    .cursor_pointer()
                                    .hover(|style| style.underline())
                                    .focus_visible(move |style| focus::ring(style, &focus_theme))
                            })
                            .on_click(move |event, window, cx| handler(event, window, cx))
                            .child(name)
                            .into_any_element()
                    }
                    None => text::xs(div(), &theme)
                        .font_weight(FontWeight::MEDIUM)
                        .child(name)
                        .into_any_element(),
                })
        });
        let title = self.title.map(|title| {
            title_text(
                (id.clone(), "title").into(),
                title,
                2,
                self.on_title_click,
                &theme,
            )
        });
        let fields = (!self.fields.is_empty()).then(|| {
            div()
                .flex()
                .flex_wrap()
                .children(self.fields.into_iter().map(|field| {
                    div()
                        .when(field.short, |cell| cell.w(relative(0.5)))
                        .when(!field.short, |cell| cell.w_full())
                        .min_w_0()
                        .pr(theme.space(3.))
                        .py(theme.space(1.))
                        .flex()
                        .flex_col()
                        .gap(theme.space(0.5))
                        .child(
                            text::xs(div(), &theme)
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(field.label),
                        )
                        .child(text::sm(div(), &theme).child(field.value))
                }))
        });
        let image = self.image.map(|image| {
            large_image(
                (id.clone(), "image").into(),
                image,
                self.on_image_click,
                &theme,
            )
        });
        let footer = (self.footer.is_some() || self.timestamp.is_some()).then(|| {
            let line: SharedString = match (self.footer, self.timestamp) {
                (Some(footer), Some(time)) => format!("{footer} · {time}").into(),
                (Some(footer), None) => footer,
                (None, Some(time)) => time,
                (None, None) => SharedString::default(),
            };
            div()
                .flex()
                .items_center()
                .gap(theme.space(1.5))
                .children(
                    self.footer_icon
                        .map(|source| round_image(source, small_icon, &theme)),
                )
                .child(
                    text::xs(div(), &theme)
                        .text_color(colors.muted_foreground)
                        .child(line),
                )
        });

        let card = card_frame(id.clone(), label, &theme)
            .flex()
            .flex_col()
            .gap(theme.space(1.))
            .children(self.pretext.map(|pretext| div().child(pretext)))
            .child(
                div()
                    .flex()
                    .child(
                        // GPUI clips to rectangles, so the bar rounds its own
                        // left corners and the content rounds the right ones.
                        div()
                            .debug_selector(|| "embed-bar".into())
                            .w(theme.space(1.))
                            .flex_shrink_0()
                            .rounded_l(theme.radius.lg)
                            .bg(bar),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .items_start()
                            .gap(theme.space(3.))
                            .p(padding)
                            .rounded_r(theme.radius.lg)
                            .border_1()
                            .border_l_0()
                            .border_color(colors.border)
                            .bg(colors.card)
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .flex()
                                    .flex_col()
                                    .gap(gap)
                                    .children(author)
                                    .children(title)
                                    .children(self.body.map(|body| div().child(body)))
                                    .children(fields)
                                    .children(image)
                                    .children(footer),
                            )
                            .children(self.thumbnail.map(|source| {
                                thumbnail((id.clone(), "thumbnail").into(), source, None, &theme)
                            })),
                    ),
            );
        apply_style(card, &self.style)
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use super::*;
    use gpui_kit::{Context, Render, TestAppContext, VisualTestContext};

    #[test]
    fn auto_layout_puts_small_and_square_images_beside_the_text() {
        use LinkPreviewImageLayout::*;
        assert_eq!(Auto.resolve(1200, 630), Large);
        assert_eq!(Auto.resolve(120, 60), Thumbnail);
        assert_eq!(Auto.resolve(400, 400), Thumbnail);
        assert_eq!(Auto.resolve(300, 600), Thumbnail);
        assert_eq!(Large.resolve(100, 100), Large);
        assert_eq!(Thumbnail.resolve(1200, 630), Thumbnail);
    }

    #[test]
    fn bar_colors_parse_hex_and_status_names() {
        let custom = |embed: MessageEmbed| match embed.color {
            Some(EmbedColor::Custom(color)) => Some(color),
            _ => None,
        };
        assert_eq!(
            custom(MessageEmbed::new("a").color_str("#ff0000")),
            Some(Rgba::try_from("#ff0000").unwrap())
        );
        assert_eq!(
            custom(MessageEmbed::new("a").color_str("0f0")),
            Some(Rgba::try_from("#00ff00").unwrap())
        );
        assert!(matches!(
            MessageEmbed::new("a").color_str("Danger").color,
            Some(EmbedColor::Danger)
        ));
        assert!(MessageEmbed::new("a").color_str("blue").color.is_none());
        assert!(MessageEmbed::new("a").color_str("#12345").color.is_none());
    }

    struct Probe {
        collapsed: bool,
        toggled: Rc<Cell<Option<bool>>>,
    }

    impl Render for Probe {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let toggled = self.toggled.clone();
            div()
                .w(px(600.))
                .flex()
                .flex_col()
                .gap(px(8.))
                .child(
                    LinkPreview::new("preview", "gpuicn 0.9.1 released")
                        .site_name("GitHub")
                        .description("Seven component hooks for Matty.")
                        .image("missing.png", 1200, 630)
                        .collapsed(self.collapsed)
                        .on_click(|_, _, _| {})
                        .on_toggle(move |collapsed, _, _| toggled.set(Some(*collapsed)))
                        .on_remove(|_, _, _| {}),
                )
                .child(
                    MessageEmbed::new("embed")
                        .color_str("good")
                        .pretext("Build finished")
                        .author("CI")
                        .title("verify #812")
                        .on_title_click(|_, _, _| {})
                        .body("All checks passed.")
                        .field("Branch", "main", true)
                        .field("Duration", "4m 12s", true)
                        .field("Commit", "Fix the dialog scroll", false)
                        .footer("GitHub Actions")
                        .timestamp("10:42"),
                )
        }
    }

    fn probe(
        cx: &mut TestAppContext,
        collapsed: bool,
    ) -> (Rc<Cell<Option<bool>>>, VisualTestContext) {
        cx.update(super::super::theme::init);
        let toggled = Rc::new(Cell::new(None));
        let window = cx.add_window({
            let toggled = toggled.clone();
            move |_, _| Probe { collapsed, toggled }
        });
        (toggled, VisualTestContext::from_window(window.into(), cx))
    }

    #[gpui_kit::test]
    fn draws_in_both_themes(cx: &mut TestAppContext) {
        let (_, mut cx) = probe(cx, false);
        for mode in [
            super::super::theme::ThemeMode::Light,
            super::super::theme::ThemeMode::Dark,
        ] {
            cx.update(|_, cx| UiTheme::switch(cx, mode));
            cx.update(|window, cx| window.draw(cx).clear(cx));
        }
        assert!(cx.debug_bounds("embed-bar").is_some());
    }

    #[gpui_kit::test]
    fn large_images_reserve_their_box_before_loading(cx: &mut TestAppContext) {
        let (_, mut cx) = probe(cx, false);
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let image = cx.debug_bounds("embed-image").unwrap();
        // 1200 × 630 fills the card's content width (480px less padding and
        // border) and keeps its aspect ratio, so the height is known up front.
        assert_eq!(image.size.width, px(454.));
        let expected = 454. * 630. / 1200.;
        assert!((f32::from(image.size.height) - expected).abs() < 1.);
    }

    #[gpui_kit::test]
    fn the_toggle_reports_the_new_state_and_collapsing_hides_the_image(cx: &mut TestAppContext) {
        let (toggled, mut cx) = probe(cx, true);
        cx.update(|window, cx| window.draw(cx).clear(cx));
        assert!(cx.debug_bounds("embed-image").is_none());
        assert!(cx.debug_bounds("embed-thumbnail").is_none());
        let toggle = cx.debug_bounds("embed-toggle").unwrap();
        cx.simulate_click(toggle.center(), Default::default());
        assert_eq!(
            toggled.get(),
            Some(false),
            "expanding reports collapsed = false"
        );
    }
}
