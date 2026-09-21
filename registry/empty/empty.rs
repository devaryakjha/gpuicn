//! Nova empty-state layout with named media, copy, and action slots.

use gpui_kit::{
    AnyElement, App, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
    prelude::FluentBuilder as _, relative,
};

use super::theme::UiTheme;

/// A centered empty state with optional header and content slots.
#[derive(IntoElement)]
pub struct Empty {
    style: gpui_kit::StyleRefinement,
    header: Option<EmptyHeader>,
    content: Option<EmptyContent>,
    children: Vec<AnyElement>,
}

impl Empty {
    /// Creates an empty state without a visible background or border.
    pub fn new() -> Self {
        Self {
            style: Default::default(),
            header: None,
            content: None,
            children: vec![],
        }
    }

    /// Sets the media and copy shown above the content.
    pub fn header(mut self, header: EmptyHeader) -> Self {
        self.header = Some(header);
        self
    }

    /// Sets application-owned actions or other supporting content.
    pub fn content(mut self, content: EmptyContent) -> Self {
        self.content = Some(content);
        self
    }
}

impl Default for Empty {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Empty {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Styled for Empty {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Empty {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx);
        let root = div()
            .w_full()
            .min_w_0()
            .flex_1()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(theme.space(4.))
            .p(theme.space(6.))
            .rounded(theme.radius.xl)
            .border_dashed()
            .border_color(theme.colors.border)
            .font_family(theme.fonts.body.clone())
            .text_center()
            .text_color(theme.colors.foreground);

        super::theme::apply_style(root, &self.style)
            .when_some(self.header, |root, header| root.child(header))
            .when_some(self.content, |root, content| root.child(content))
            .children(self.children)
    }
}

/// The media, title, and description of an [`Empty`] state.
#[derive(IntoElement)]
pub struct EmptyHeader {
    style: gpui_kit::StyleRefinement,
    media: Option<EmptyMedia>,
    title: Option<EmptyTitle>,
    description: Option<EmptyDescription>,
}

impl EmptyHeader {
    /// Creates a centered header with no slots configured.
    pub fn new() -> Self {
        Self {
            style: Default::default(),
            media: None,
            title: None,
            description: None,
        }
    }

    /// Sets the optional image, icon, or custom media.
    pub fn media(mut self, media: EmptyMedia) -> Self {
        self.media = Some(media);
        self
    }

    /// Sets the primary message.
    pub fn title(mut self, title: EmptyTitle) -> Self {
        self.title = Some(title);
        self
    }

    /// Sets the supporting message.
    pub fn description(mut self, description: EmptyDescription) -> Self {
        self.description = Some(description);
        self
    }
}

impl Default for EmptyHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl Styled for EmptyHeader {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for EmptyHeader {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx);
        let root = div()
            .w_full()
            .max_w(theme.space(96.))
            .min_w_0()
            .flex()
            .flex_col()
            .items_center()
            .gap(theme.space(2.));

        super::theme::apply_style(root, &self.style)
            .when_some(self.media, |root, media| root.child(media))
            .when_some(self.title, |root, title| root.child(title))
            .when_some(self.description, |root, description| {
                root.child(description)
            })
    }
}

/// The visual treatment for [`EmptyMedia`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum EmptyMediaVariant {
    /// Leaves images, avatars, and other custom media unframed.
    #[default]
    Default,
    /// Places an icon in a compact muted frame.
    Icon,
}

/// A media slot for an icon, image, avatar, or custom element.
#[derive(IntoElement)]
pub struct EmptyMedia {
    style: gpui_kit::StyleRefinement,
    variant: EmptyMediaVariant,
    children: Vec<AnyElement>,
}

impl EmptyMedia {
    /// Creates an unframed media slot.
    pub fn new() -> Self {
        Self {
            style: Default::default(),
            variant: EmptyMediaVariant::Default,
            children: vec![],
        }
    }

    /// Sets the media treatment without changing its content.
    pub fn with_variant(mut self, variant: EmptyMediaVariant) -> Self {
        self.variant = variant;
        self
    }
}

impl Default for EmptyMedia {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for EmptyMedia {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Styled for EmptyMedia {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for EmptyMedia {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx);
        let root = div()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .mb(theme.space(2.))
            .when(self.variant == EmptyMediaVariant::Icon, |root| {
                root.size(theme.space(8.))
                    .rounded(theme.radius.lg)
                    .bg(theme.colors.muted)
                    .text_color(theme.colors.foreground)
                    .text_size(theme.text(16.))
            });

        super::theme::apply_style(root, &self.style).children(self.children)
    }
}

/// The primary message in an [`Empty`] state.
#[derive(IntoElement)]
pub struct EmptyTitle {
    style: gpui_kit::StyleRefinement,
    children: Vec<AnyElement>,
}

impl EmptyTitle {
    /// Creates an empty title slot.
    pub fn new() -> Self {
        Self {
            style: Default::default(),
            children: vec![],
        }
    }
}

impl Default for EmptyTitle {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for EmptyTitle {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Styled for EmptyTitle {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for EmptyTitle {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx);
        let root = div()
            .max_w_full()
            .min_w_0()
            .font_weight(gpui_kit::FontWeight::MEDIUM)
            .text_size(theme.text(14.))
            .line_height(theme.text(20.))
            .whitespace_normal();

        super::theme::apply_style(root, &self.style).children(self.children)
    }
}

/// Supporting text or rich content for an [`Empty`] state.
#[derive(IntoElement)]
pub struct EmptyDescription {
    style: gpui_kit::StyleRefinement,
    children: Vec<AnyElement>,
}

impl EmptyDescription {
    /// Creates a muted description slot.
    pub fn new() -> Self {
        Self {
            style: Default::default(),
            children: vec![],
        }
    }
}

impl Default for EmptyDescription {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for EmptyDescription {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Styled for EmptyDescription {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for EmptyDescription {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx);
        let root = div()
            .w_full()
            .min_w_0()
            .text_size(theme.text(14.))
            .line_height(relative(1.625))
            .text_color(theme.colors.muted_foreground)
            .whitespace_normal();

        super::theme::apply_style(root, &self.style).children(self.children)
    }
}

/// Application-owned actions or supporting content below an [`EmptyHeader`].
#[derive(IntoElement)]
pub struct EmptyContent {
    style: gpui_kit::StyleRefinement,
    children: Vec<AnyElement>,
}

impl EmptyContent {
    /// Creates a centered content column.
    pub fn new() -> Self {
        Self {
            style: Default::default(),
            children: vec![],
        }
    }
}

impl Default for EmptyContent {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for EmptyContent {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Styled for EmptyContent {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for EmptyContent {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx);
        let root = div()
            .w_full()
            .max_w(theme.space(96.))
            .min_w_0()
            .flex()
            .flex_col()
            .items_center()
            .gap(theme.space(2.5))
            .text_size(theme.text(14.));

        super::theme::apply_style(root, &self.style).children(self.children)
    }
}
