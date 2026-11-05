//! Nova embed cards under a chat message: link previews (Open Graph) and
//! message attachments (the integration format with a colored bar, fields and
//! a footer) and quoted posts for permalinks and forwards.
//!
//! shadcn/ui has no embed card; both cards are drawn on the card tokens with
//! the same density steps as `Message`. Titles that open something are GPUI
//! Kit buttons with the link role, so they take focus and activate on Enter.
//! gpuicn never fetches: images arrive as `ImageSource` with their pixel size
//! so the card reserves its box before the bytes load. When the application
//! asks before loading remote images, an [`ImagePlaceholder`] takes the
//! image's box until the person chooses to load it.

use std::rc::Rc;

use gpui_icons::{LucideIcon, lucide};
use gpui_kit::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, ImageSource, InteractiveElement as _,
    IntoElement, ObjectFit, ParentElement, Pixels, RenderOnce, Rgba, Role, SharedString,
    StatefulInteractiveElement as _, Styled, Window, div, prelude::FluentBuilder as _, px,
    relative,
};

use gpui_kit::base::ElementExt as _;

use super::avatar::{Avatar, AvatarSize};
use super::button::{Button, ButtonSize, ButtonVariant};
pub use super::message::MessageDensity;
use super::theme::{
    UiText, UiTheme, apply_style,
    locale::{self, DirectionalStyled as _},
    primitives::{clipped_image, focus, icon, round_image, text},
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

/// A click-to-load stand-in for a remote image, for applications that ask
/// before loading one. It takes the box the image would take, so nothing
/// moves when the image arrives, and shows an image icon, "Load image" and
/// the image's host. It is a button named "Load image from {host}" that
/// runs `on_load` once on click, Enter or Space; the caller then fetches the
/// image and passes it through `image` on the next render.
#[derive(Clone)]
pub struct ImagePlaceholder {
    host: SharedString,
    size: Option<(u32, u32)>,
    label: Option<SharedString>,
    aria_label: Option<SharedString>,
    on_load: Handler,
}

impl ImagePlaceholder {
    /// Creates a placeholder for an image served from `host`, such as
    /// "github.com". `on_load` runs once, when the person asks to load it.
    pub fn new(
        host: impl Into<SharedString>,
        on_load: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            host: host.into(),
            size: None,
            label: None,
            aria_label: None,
            on_load: Rc::new(on_load),
        }
    }
    /// The image's pixel size from the server's metadata, so the placeholder
    /// takes the image's box and layout. Without it the box is 16:9 and large.
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.size = Some((width, height));
        self
    }
    /// Replaces the visible "Load image" text, such as for a translation.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
    /// Replaces the accessible name, "Load image from {host}" by default.
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }
    /// The pixel size layouts resolve against; 16:9 when unknown.
    fn layout_size(&self) -> (u32, u32) {
        self.size.unwrap_or((1600, 900))
    }
}

/// What fills a card's image slot.
#[derive(Clone)]
enum ImageSlot {
    Image(SizedImage),
    Placeholder(ImagePlaceholder),
}

/// What fills a card's thumbnail square.
enum ThumbnailSlot {
    Image(ImageSource),
    Placeholder(ImagePlaceholder),
}

impl ImageSlot {
    fn size(&self) -> (u32, u32) {
        match self {
            Self::Image(image) => (image.width, image.height),
            Self::Placeholder(placeholder) => placeholder.layout_size(),
        }
    }
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

/// The box of a large image of `width` × `height` pixels. The width also
/// shrinks with a narrow card, and the aspect ratio keeps the height in step.
fn large_box(
    id: ElementId,
    width: u32,
    height: u32,
    theme: &UiTheme,
) -> gpui_kit::Stateful<gpui_kit::Div> {
    let (width, height) = (width.max(1) as f32, height.max(1) as f32);
    let (box_width, _) = fit_within(width, height, (theme.space(114.), theme.space(72.)));
    div()
        .id(id)
        .w(box_width)
        .max_w_full()
        .aspect_ratio(width / height)
        .flex_shrink_0()
}

/// A large image whose box is reserved from its pixel size.
fn large_image(
    id: ElementId,
    image: SizedImage,
    on_click: Option<Handler>,
    theme: &UiTheme,
) -> AnyElement {
    large_box(id, image.width, image.height, theme)
        .debug_selector(|| "embed-image".into())
        .child(clipped_image(image.source, theme.radius.md, ObjectFit::Cover, theme).size_full())
        .when_some(on_click, |frame, handler| {
            frame
                .cursor_pointer()
                .on_click(move |event, window, cx| handler(event, window, cx))
        })
        .into_any_element()
}

/// The box of a small square image beside the text.
fn thumbnail_box(id: ElementId, theme: &UiTheme) -> gpui_kit::Stateful<gpui_kit::Div> {
    div().id(id).flex_shrink_0().size(theme.space(20.))
}

/// A small square image beside the text.
fn thumbnail(
    id: ElementId,
    source: ImageSource,
    on_click: Option<Handler>,
    theme: &UiTheme,
) -> AnyElement {
    thumbnail_box(id, theme)
        .debug_selector(|| "embed-thumbnail".into())
        .child(clipped_image(source, theme.radius.md, ObjectFit::Cover, theme).size_full())
        .when_some(on_click, |frame, handler| {
            frame
                .cursor_pointer()
                .on_click(move |event, window, cx| handler(event, window, cx))
        })
        .into_any_element()
}

/// The placeholder's button filling an image box: a muted surface with the
/// image icon, the label and the host, which runs `on_load` only once.
fn placeholder_button(
    id: ElementId,
    placeholder: ImagePlaceholder,
    window: &mut Window,
    cx: &mut App,
    theme: &UiTheme,
) -> Button {
    let colors = theme.colors;
    let loaded = window.use_keyed_state((id.clone(), "loaded"), cx, |_, _| false);
    let aria_label = placeholder
        .aria_label
        .unwrap_or_else(|| locale::text(cx, UiText::LoadImageFrom(placeholder.host.clone())));
    let label = placeholder
        .label
        .unwrap_or_else(|| locale::text(cx, UiText::LoadImage));
    let on_load = placeholder.on_load;
    let style_theme = theme.clone();
    Button::new(id)
        .aria_label(aria_label)
        .style_with(move |base| {
            let theme = style_theme.clone();
            let focus_theme = theme.clone();
            base.size_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(theme.space(1.))
                .p(theme.space(2.))
                .overflow_hidden()
                .rounded(theme.radius.md)
                .bg(theme.colors.muted)
                .cursor_pointer()
                .hover(move |style| style.bg(theme.colors.accent))
                .focus_visible(move |style| focus::ring(style, &focus_theme))
        })
        .on_click(move |event, window, cx| {
            if *loaded.read(cx) {
                return;
            }
            loaded.update(cx, |loaded, _| *loaded = true);
            on_load(event, window, cx);
        })
        .child(
            lucide(LucideIcon::Image)
                .flex_shrink_0()
                .size(icon::lg(theme))
                .text_color(colors.muted_foreground),
        )
        .child(
            text::xs(div(), theme)
                .max_w_full()
                .font_weight(FontWeight::MEDIUM)
                .overflow_hidden()
                .text_ellipsis()
                .whitespace_nowrap()
                .child(label),
        )
        .child(
            text::xs(div(), theme)
                .max_w_full()
                .text_color(colors.muted_foreground)
                .overflow_hidden()
                .text_ellipsis()
                .whitespace_nowrap()
                .child(placeholder.host),
        )
}

/// A placeholder in a large image's box.
fn large_placeholder(
    id: ElementId,
    placeholder: ImagePlaceholder,
    window: &mut Window,
    cx: &mut App,
    theme: &UiTheme,
) -> AnyElement {
    let (width, height) = placeholder.layout_size();
    large_box((id.clone(), "frame").into(), width, height, theme)
        .debug_selector(|| "embed-image-placeholder".into())
        .child(placeholder_button(id, placeholder, window, cx, theme))
        .into_any_element()
}

/// A placeholder in a box of exactly `width` by `height`, for images laid
/// out by another component, such as inline Markdown images.
#[doc(hidden)]
pub fn placeholder_in_box(
    id: impl Into<ElementId>,
    placeholder: ImagePlaceholder,
    width: gpui_kit::Pixels,
    height: gpui_kit::Pixels,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let theme = UiTheme::read(cx).clone();
    let id = id.into();
    div()
        .w(width)
        .h(height)
        .max_w_full()
        .debug_selector(|| "markdown-image-placeholder".into())
        .child(placeholder_button(id, placeholder, window, cx, &theme))
        .into_any_element()
}

impl ImagePlaceholder {
    /// The size set with [`ImagePlaceholder::size`], if any.
    pub fn known_size(&self) -> Option<(u32, u32)> {
        self.size
    }
}

/// A placeholder in a thumbnail's box.
fn thumbnail_placeholder(
    id: ElementId,
    placeholder: ImagePlaceholder,
    window: &mut Window,
    cx: &mut App,
    theme: &UiTheme,
) -> AnyElement {
    thumbnail_box((id.clone(), "frame").into(), theme)
        .debug_selector(|| "embed-thumbnail-placeholder".into())
        .child(placeholder_button(id, placeholder, window, cx, theme))
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
    label: SharedString,
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
    image: Option<ImageSlot>,
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
        self.image = Some(ImageSlot::Image(SizedImage {
            source: source.into(),
            width,
            height,
        }));
        self
    }
    /// Shows a click-to-load placeholder in the image's place, with the same
    /// box and layout the image would take. Replaces `image`; pass the image
    /// through `image` once it has loaded.
    pub fn image_placeholder(mut self, placeholder: ImagePlaceholder) -> Self {
        self.image = Some(ImageSlot::Placeholder(placeholder));
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
    /// when the preview has an image or an image placeholder.
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
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
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
                    (
                        theme.direction.icon(
                            true,
                            LucideIcon::ChevronLeft,
                            LucideIcon::ChevronRight,
                        ),
                        locale::text(cx, UiText::ExpandPreview),
                    )
                } else {
                    (
                        LucideIcon::ChevronDown,
                        locale::text(cx, UiText::CollapsePreview),
                    )
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
                locale::text(cx, UiText::RemovePreview),
                move |event, window, cx| handler(event, window, cx),
                &theme,
            )
        });
        let header =
            (self.site_name.is_some() || toggle.is_some() || remove.is_some()).then(|| {
                div()
                    .flex_row_in(theme.direction)
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
            Some(slot) => {
                let (width, height) = slot.size();
                let as_thumbnail =
                    self.image_layout.resolve(width, height) == LinkPreviewImageLayout::Thumbnail;
                match (slot, as_thumbnail) {
                    (ImageSlot::Image(image), true) => (
                        None,
                        Some(thumbnail(
                            (id.clone(), "thumbnail").into(),
                            image.source,
                            self.on_click.clone(),
                            &theme,
                        )),
                    ),
                    (ImageSlot::Image(image), false) => (
                        Some(large_image(
                            (id.clone(), "image").into(),
                            image,
                            self.on_click.clone(),
                            &theme,
                        )),
                        None,
                    ),
                    (ImageSlot::Placeholder(placeholder), true) => (
                        None,
                        Some(thumbnail_placeholder(
                            (id.clone(), "thumbnail-placeholder").into(),
                            placeholder,
                            window,
                            cx,
                            &theme,
                        )),
                    ),
                    (ImageSlot::Placeholder(placeholder), false) => (
                        Some(large_placeholder(
                            (id.clone(), "image-placeholder").into(),
                            placeholder,
                            window,
                            cx,
                            &theme,
                        )),
                        None,
                    ),
                }
            }
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
                    .flex_row_in(theme.direction)
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
    image: Option<ImageSlot>,
    on_image_click: Option<Handler>,
    thumbnail: Option<ThumbnailSlot>,
    footer: Option<SharedString>,
    footer_icon: Option<ImageSource>,
    timestamp: Option<SharedString>,
    actions: Vec<AnyElement>,
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
            actions: Vec::new(),
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
        self.image = Some(ImageSlot::Image(SizedImage {
            source: source.into(),
            width,
            height,
        }));
        self
    }
    /// Shows a click-to-load placeholder in the image's place, in the box the
    /// image would take. Replaces `image`; pass the image through `image` once
    /// it has loaded.
    pub fn image_placeholder(mut self, placeholder: ImagePlaceholder) -> Self {
        self.image = Some(ImageSlot::Placeholder(placeholder));
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
        self.thumbnail = Some(ThumbnailSlot::Image(source.into()));
        self
    }
    /// Shows a click-to-load placeholder in the thumbnail's square. Replaces
    /// `thumbnail`; pass the image through `thumbnail` once it has loaded.
    pub fn thumbnail_placeholder(mut self, placeholder: ImagePlaceholder) -> Self {
        self.thumbnail = Some(ThumbnailSlot::Placeholder(placeholder));
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
    /// Adds an interactive action, such as a `Button` or a `Select`, to the
    /// row at the bottom of the card, after the fields and footer. Actions
    /// wrap onto more rows when the card is narrow.
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }
    /// Adds several actions; see [`MessageEmbed::action`].
    pub fn actions(mut self, actions: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.actions
            .extend(actions.into_iter().map(IntoElement::into_any_element));
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
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
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
            .unwrap_or_else(|| locale::text(cx, UiText::Attachment));
        let small_icon = theme.space(4.);
        let dir = theme.direction;

        let author = self.author.map(|name| {
            div()
                .flex_row_in(dir)
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
                .flex_row_in(dir)
                .flex_wrap()
                .children(self.fields.into_iter().map(|field| {
                    div()
                        .when(field.short, |cell| cell.w(relative(0.5)))
                        .when(!field.short, |cell| cell.w_full())
                        .min_w_0()
                        .padding_end(dir, theme.space(3.))
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
        let image = self.image.map(|slot| match slot {
            ImageSlot::Image(image) => large_image(
                (id.clone(), "image").into(),
                image,
                self.on_image_click,
                &theme,
            ),
            ImageSlot::Placeholder(placeholder) => large_placeholder(
                (id.clone(), "image-placeholder").into(),
                placeholder,
                window,
                cx,
                &theme,
            ),
        });
        let thumb = self.thumbnail.map(|slot| match slot {
            ThumbnailSlot::Image(source) => {
                thumbnail((id.clone(), "thumbnail").into(), source, None, &theme)
            }
            ThumbnailSlot::Placeholder(placeholder) => thumbnail_placeholder(
                (id.clone(), "thumbnail-placeholder").into(),
                placeholder,
                window,
                cx,
                &theme,
            ),
        });
        let footer = (self.footer.is_some() || self.timestamp.is_some()).then(|| {
            let line: SharedString = match (self.footer, self.timestamp) {
                (Some(footer), Some(time)) => format!("{footer} · {time}").into(),
                (Some(footer), None) => footer,
                (None, Some(time)) => time,
                (None, None) => SharedString::default(),
            };
            div()
                .flex_row_in(dir)
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

        let actions = self.actions;
        let card = card_frame(id.clone(), label, &theme)
            .flex()
            .flex_col()
            .gap(theme.space(1.))
            .children(self.pretext.map(|pretext| div().child(pretext)))
            .child(
                div()
                    .flex_row_in(dir)
                    .child(
                        // GPUI clips to rectangles, so the bar rounds its own
                        // start corners and the content rounds the end ones.
                        div()
                            .debug_selector(|| "embed-bar".into())
                            .w(theme.space(1.))
                            .flex_shrink_0()
                            .rounded_start(dir, theme.radius.lg)
                            .bg(bar),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex_row_in(dir)
                            .items_start()
                            .gap(theme.space(3.))
                            .p(padding)
                            .rounded_end(dir, theme.radius.lg)
                            .border_1()
                            .border_start_0(dir)
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
                                    .children(footer)
                                    .when(!actions.is_empty(), |content| {
                                        content.child(
                                            div()
                                                .id((id.clone(), "actions"))
                                                .pt(theme.space(1.))
                                                .flex_row_in(dir)
                                                .flex_wrap()
                                                .items_center()
                                                .gap(theme.space(2.))
                                                .children(actions),
                                        )
                                    }),
                            )
                            .children(thumb),
                    ),
            );
        apply_style(card, &self.style)
    }
}

/// How many lines of a quoted body show before "Show more".
pub const QUOTED_MESSAGE_LINES: usize = 4;

/// A quoted post: a pasted permalink or a forwarded message, drawn as its own
/// card next to [`LinkPreview`] and [`MessageEmbed`].
///
/// One line holds the author's avatar, name and time with a trailing Jump
/// link; a muted line says where it was first posted; the body is clamped
/// to a few lines with a "Show more" toggle, and an optional strip below it
/// holds the quoted post's files or images. The card is named "Quoted
/// message from <author>" for assistive technology.
#[derive(IntoElement)]
pub struct QuotedMessage {
    style: gpui_kit::StyleRefinement,
    id: ElementId,
    author: SharedString,
    avatar: Option<ImageSource>,
    initials: Option<SharedString>,
    timestamp: Option<SharedString>,
    channel: Option<SharedString>,
    body: Option<AnyElement>,
    attachments: Option<AnyElement>,
    max_lines: usize,
    density: MessageDensity,
    on_jump: Option<Handler>,
}

impl QuotedMessage {
    /// Creates a quoted post by `author` with a stable caller-owned ID.
    pub fn new(id: impl Into<ElementId>, author: impl Into<SharedString>) -> Self {
        Self {
            style: gpui_kit::StyleRefinement::default(),
            id: id.into(),
            author: author.into(),
            avatar: None,
            initials: None,
            timestamp: None,
            channel: None,
            body: None,
            attachments: None,
            max_lines: QUOTED_MESSAGE_LINES,
            density: MessageDensity::Standard,
            on_jump: None,
        }
    }
    /// Sets the author's avatar image; the initials show while it loads.
    pub fn avatar(mut self, source: impl Into<ImageSource>) -> Self {
        self.avatar = Some(source.into());
        self
    }
    /// Sets the initials drawn when there is no avatar image. Defaults to the
    /// first letter of the author's name.
    pub fn initials(mut self, initials: impl Into<SharedString>) -> Self {
        self.initials = Some(initials.into());
        self
    }
    /// Sets the time shown after the author's name, already formatted.
    pub fn timestamp(mut self, timestamp: impl Into<SharedString>) -> Self {
        self.timestamp = Some(timestamp.into());
        self
    }
    /// Names the channel the post came from, shown as "Originally posted in
    /// <channel>". Leave it out for direct messages.
    pub fn channel(mut self, channel: impl Into<SharedString>) -> Self {
        self.channel = Some(channel.into());
        self
    }
    /// Sets the quoted text, usually rendered Markdown.
    pub fn body(mut self, body: impl IntoElement) -> Self {
        self.body = Some(body.into_any_element());
        self
    }
    /// Sets the strip of the quoted post's files or images under the body.
    pub fn attachments(mut self, attachments: impl IntoElement) -> Self {
        self.attachments = Some(attachments.into_any_element());
        self
    }
    /// Sets how many body lines show before "Show more"; at least one.
    pub fn max_lines(mut self, lines: usize) -> Self {
        self.max_lines = lines.max(1);
        self
    }
    /// Matches the density of the message list around the card.
    pub fn density(mut self, density: MessageDensity) -> Self {
        self.density = density;
        self
    }
    /// Shows a Jump link that opens the original post.
    pub fn on_jump(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_jump = Some(Rc::new(handler));
        self
    }
}

impl Styled for QuotedMessage {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}

/// Whether the body is expanded, and whether it is taller than the clamp.
#[derive(Default)]
struct QuoteState {
    expanded: bool,
    overflows: bool,
}

/// A small link-styled button, as Jump and Show more are.
fn link_button(
    id: ElementId,
    label: SharedString,
    aria_label: SharedString,
    expanded: Option<bool>,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    theme: &UiTheme,
) -> Button {
    let theme = theme.clone();
    Button::new(id)
        .aria_label(aria_label)
        .style_with(move |base| {
            let focus_theme = theme.clone();
            let base = match expanded {
                Some(expanded) => base.aria_expanded(expanded),
                None => base.role(Role::Link),
            };
            text::xs(base, &theme)
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.colors.primary)
                .rounded(theme.radius.sm)
                .flex_shrink_0()
                .cursor_pointer()
                .hover(|style| style.underline())
                .focus_visible(move |style| focus::ring(style, &focus_theme))
        })
        .on_click(on_click)
        .child(label)
}

impl RenderOnce for QuotedMessage {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let (padding, gap) = metrics(&theme, self.density);
        let id = self.id;
        let state = window.use_keyed_state((id.clone(), "quote"), cx, |_, _| QuoteState::default());
        let (expanded, overflows) = {
            let state = state.read(cx);
            (state.expanded, state.overflows)
        };
        let line_height = theme.text_style(theme.typography.sm).line_height;
        let clamp = line_height * self.max_lines as f32;
        let show_more = locale::text(cx, UiText::ShowMore);
        let show_less = locale::text(cx, UiText::ShowLess);

        let initials = self.initials.clone().unwrap_or_else(|| {
            self.author
                .chars()
                .next()
                .map(|c| c.to_uppercase().to_string())
                .unwrap_or_default()
                .into()
        });
        let avatar = Avatar::new((id.clone(), "avatar"))
            .size(AvatarSize::Sm)
            .fallback(initials)
            .when_some(self.avatar, |avatar, source| avatar.image(source));
        let dir = theme.direction;
        let author = self.author.clone();
        let jump_text = locale::text(cx, UiText::Jump);
        let jump_label = locale::text(cx, UiText::JumpToMessageFrom(author.clone()));
        let jump = self.on_jump.map(|handler| {
            div()
                .debug_selector(|| "quote-jump".into())
                .child(link_button(
                    (id.clone(), "jump").into(),
                    jump_text,
                    jump_label,
                    None,
                    move |event, window, cx| handler(event, window, cx),
                    &theme,
                ))
        });
        let header = div()
            .flex_row_in(dir)
            .items_center()
            .gap(theme.space(2.))
            .min_w_0()
            .child(avatar)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex_row_in(dir)
                    .items_baseline()
                    .gap(theme.space(2.))
                    .overflow_hidden()
                    .child(
                        text::sm(div(), &theme)
                            .font_weight(FontWeight::SEMIBOLD)
                            .flex_shrink_0()
                            .child(self.author.clone()),
                    )
                    .children(self.timestamp.map(|time| {
                        text::xs(div(), &theme)
                            .text_color(colors.muted_foreground)
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(time)
                    })),
            )
            .children(jump);
        let origin = self.channel.map(|channel| {
            text::xs(div(), &theme)
                .debug_selector(|| "quote-origin".into())
                .text_color(colors.muted_foreground)
                .overflow_hidden()
                .text_ellipsis()
                .whitespace_nowrap()
                .child(locale::text(
                    cx,
                    UiText::OriginallyPostedIn(locale::isolate(&channel)),
                ))
        });
        let body = self.body.map(|body| {
            let measured = state.clone();
            div()
                .flex()
                .flex_col()
                .gap(theme.space(1.))
                .child(
                    text::sm(div(), &theme)
                        .debug_selector(|| "quote-body".into())
                        .when(!expanded, |frame| frame.max_h(clamp).overflow_hidden())
                        .child(
                            div()
                                .on_prepaint(move |bounds, window, cx| {
                                    // The text's natural height, measured
                                    // inside the clamp, decides whether
                                    // there is more to show.
                                    let overflows = bounds.size.height > clamp + px(0.5);
                                    if measured.read(cx).overflows != overflows {
                                        measured.update(cx, |state, _| state.overflows = overflows);
                                        window.refresh();
                                    }
                                })
                                .child(body),
                        ),
                )
                .when(overflows, |column| {
                    let toggle = state.clone();
                    let label = if expanded {
                        show_less.clone()
                    } else {
                        show_more.clone()
                    };
                    column.child(div().debug_selector(|| "quote-toggle".into()).flex().child(
                        link_button(
                            (id.clone(), "more").into(),
                            label.clone(),
                            label,
                            Some(expanded),
                            move |_, window, cx| {
                                toggle.update(cx, |state, _| state.expanded = !state.expanded);
                                window.refresh();
                            },
                            &theme,
                        ),
                    ))
                })
        });

        let card = card_frame(
            id.clone(),
            locale::text(cx, UiText::QuotedMessageFrom(author.clone())),
            &theme,
        )
        .flex()
        .flex_col()
        .gap(gap)
        .p(padding)
        .rounded(theme.radius.lg)
        .border_1()
        .border_color(colors.border)
        .bg(colors.card)
        .child(header)
        .children(origin)
        .children(body)
        .children(
            self.attachments
                .map(|strip| div().flex().flex_wrap().gap(gap).child(strip)),
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

    struct ActionsProbe;

    impl Render for ActionsProbe {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w(px(480.)).child(
                MessageEmbed::new("deploy")
                    .color_str("#3f51b5")
                    .title("Deploy to production?")
                    .footer("Release bot")
                    .action(
                        Button::new("approve")
                            .tint(Rgba::try_from("#3f51b5").unwrap())
                            .label("Approve"),
                    )
                    .action(Button::new("reject").label("Reject")),
            )
        }
    }

    #[gpui_kit::test]
    fn actions_sit_inside_the_card_after_the_footer(cx: &mut TestAppContext) {
        use gpui_kit::test::TestQueryExt as _;
        cx.update(super::super::theme::init);
        let window = cx.add_window(|_, _| ActionsProbe);
        let mut cx = VisualTestContext::from_window(window.into(), cx);
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let bar = cx.debug_bounds("embed-bar").unwrap();
        cx.update(|window, _| {
            let approve = window.find_by_label("Approve").bounds();
            window.find_by_label("Reject");
            assert!(
                approve.left() > bar.right(),
                "inside the card, past the bar"
            );
            assert!(approve.bottom() <= bar.bottom(), "within the bar's height");
        });
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

    struct PlaceholderProbe {
        loads: Rc<Cell<usize>>,
        size: Option<(u32, u32)>,
    }

    impl Render for PlaceholderProbe {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let loads = self.loads.clone();
            let mut placeholder =
                ImagePlaceholder::new("github.com", move |_, _, _| loads.set(loads.get() + 1));
            if let Some((width, height)) = self.size {
                placeholder = placeholder.size(width, height);
            }
            div().w(px(600.)).child(
                LinkPreview::new("preview", "gpuicn 0.9.3 released")
                    .site_name("GitHub")
                    .image_placeholder(placeholder),
            )
        }
    }

    fn placeholder_probe(
        cx: &mut TestAppContext,
        size: Option<(u32, u32)>,
    ) -> (Rc<Cell<usize>>, VisualTestContext) {
        cx.update(super::super::theme::init);
        let loads = Rc::new(Cell::new(0));
        let window = cx.add_window({
            let loads = loads.clone();
            move |_, _| PlaceholderProbe { loads, size }
        });
        let mut cx = VisualTestContext::from_window(window.into(), cx);
        cx.update(|window, cx| window.draw(cx).clear(cx));
        (loads, cx)
    }

    #[gpui_kit::test]
    fn placeholders_take_the_images_box(cx: &mut TestAppContext) {
        // The same 1200 × 630 box the loaded image takes.
        let (_, mut sized) = placeholder_probe(cx, Some((1200, 630)));
        let frame = sized.debug_bounds("embed-image-placeholder").unwrap();
        assert_eq!(frame.size.width, px(454.));
        assert!((f32::from(frame.size.height) - 454. * 630. / 1200.).abs() < 1.);
    }

    #[gpui_kit::test]
    fn placeholders_without_a_size_are_16_by_9(cx: &mut TestAppContext) {
        let (_, mut cx) = placeholder_probe(cx, None);
        let frame = cx.debug_bounds("embed-image-placeholder").unwrap();
        assert_eq!(frame.size.width, px(454.));
        assert!((f32::from(frame.size.height) - 454. * 9. / 16.).abs() < 1.);
    }

    #[gpui_kit::test]
    fn square_placeholders_sit_in_the_thumbnail(cx: &mut TestAppContext) {
        let (_, mut cx) = placeholder_probe(cx, Some((400, 400)));
        assert!(cx.debug_bounds("embed-image-placeholder").is_none());
        let frame = cx.debug_bounds("embed-thumbnail-placeholder").unwrap();
        assert_eq!(frame.size.width, frame.size.height);
    }

    #[gpui_kit::test]
    fn the_placeholder_loads_once(cx: &mut TestAppContext) {
        let (loads, mut cx) = placeholder_probe(cx, None);
        let frame = cx.debug_bounds("embed-image-placeholder").unwrap();
        cx.simulate_click(frame.center(), Default::default());
        cx.simulate_click(frame.center(), Default::default());
        assert_eq!(loads.get(), 1);
    }

    #[gpui_kit::test]
    fn enter_loads_the_focused_placeholder(cx: &mut TestAppContext) {
        cx.update(super::super::theme::init);
        let loads = Rc::new(Cell::new(0));
        let window = cx.add_window({
            let loads = loads.clone();
            move |window, _| {
                window.activate_window();
                PlaceholderProbe { loads, size: None }
            }
        });
        let mut cx = VisualTestContext::from_window(window.into(), cx);
        cx.run_until_parked();
        for keys in ["tab", "enter", "space"] {
            cx.update(|window, cx| window.draw(cx).clear(cx));
            cx.simulate_keystrokes(keys);
            cx.update(|window, cx| {
                window.dispatch_event(
                    gpui_kit::PlatformInput::KeyUp(gpui_kit::KeyUpEvent {
                        keystroke: gpui_kit::Keystroke::parse(keys).unwrap(),
                    }),
                    cx,
                );
            });
            cx.run_until_parked();
            if keys == "enter" {
                assert_eq!(loads.get(), 1, "Enter runs on_load");
            }
        }
        assert_eq!(loads.get(), 1, "on_load runs once");
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

    struct Quote {
        jumps: Rc<Cell<usize>>,
        body: &'static str,
    }

    impl Render for Quote {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let jumps = self.jumps.clone();
            div().w(px(420.)).child(
                QuotedMessage::new("quote", "Margaret Hamilton")
                    .timestamp("Yesterday at 4:12 PM")
                    .channel("~town-square")
                    .max_lines(2)
                    .body(div().child(self.body))
                    .on_jump(move |_, _, _| jumps.set(jumps.get() + 1)),
            )
        }
    }

    fn quote(body: &'static str, cx: &mut TestAppContext) -> (Rc<Cell<usize>>, VisualTestContext) {
        cx.update(super::super::theme::init);
        let jumps = Rc::new(Cell::new(0));
        let window = cx.add_window({
            let jumps = jumps.clone();
            move |_, _| Quote { jumps, body }
        });
        let mut cx = VisualTestContext::from_window(window.into(), cx);
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            window.draw(cx).clear(cx);
        });
        (jumps, cx)
    }

    const LONG: &str = "The landing software flagged an overflow, and the computer \
        shed its lowest-priority work to keep the guidance running. That was the \
        design: the priority scheduler restarted the jobs that mattered, and the \
        crew carried on with the descent while Houston confirmed the alarm was safe.";

    #[gpui_kit::test]
    fn a_long_quote_clamps_and_show_more_expands_it(cx: &mut TestAppContext) {
        let (_, mut cx) = quote(LONG, cx);
        let clamped = cx.debug_bounds("quote-body").unwrap();
        let line = UiTheme::default()
            .text_style(UiTheme::default().typography.sm)
            .line_height;
        assert_eq!(clamped.size.height, line * 2., "two lines");
        assert!(cx.debug_bounds("quote-origin").is_some());
        let toggle = cx.debug_bounds("quote-toggle").expect("Show more appears");
        let on_label = toggle.origin + gpui_kit::point(px(8.), toggle.size.height / 2.);
        cx.simulate_click(on_label, Default::default());
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let expanded = cx.debug_bounds("quote-body").unwrap();
        assert!(expanded.size.height > clamped.size.height, "{expanded:?}");
    }

    #[gpui_kit::test]
    fn a_short_quote_has_no_show_more_and_jump_reports(cx: &mut TestAppContext) {
        let (jumps, mut cx) = quote("Lunch?", cx);
        assert!(cx.debug_bounds("quote-toggle").is_none());
        cx.update(|window, _| {
            use gpui_kit::test::TestQueryExt as _;
            window.find_by_label("Jump to the message from Margaret Hamilton");
        });
        let jump = cx.debug_bounds("quote-jump").unwrap();
        cx.simulate_click(jump.center(), Default::default());
        assert_eq!(jumps.get(), 1);
    }
}
