//! Nova-styled Markdown, rendered by GPUI Kit's selectable rich text view.
//!
//! Code blocks are syntax-highlighted when the application enables GPUI Kit's
//! `syntect` (or `tree-sitter`) feature; without it they render as plain
//! monospaced text with the same frame.
//!
//! Wrap the window's content in `selection_root` so a selection can span
//! several views, such as many messages in a transcript, and copy with
//! Cmd+C or Ctrl+C.
//!
//! Images never load over the network on their own: add `MarkdownImages` to
//! resolve each image URL through the application's media cache, and
//! `CustomEmoji` for `:emoji:` images sized to the text.

use std::{collections::HashMap, sync::Arc};

use gpui_kit::base::{SelectableText, TextSelection, TextSelectionLayer};
use gpui_kit::component::text::{
    InlineElement, InlineRenderContext, MarkdownNode, MarkdownParseContext, MarkdownPlugin,
    TextView, TextViewStyle, markdown_ast,
};
use gpui_kit::{
    App, ClipboardItem, ElementId, HighlightStyle, ImageSource, InteractiveElement as _,
    IntoElement, ObjectFit, ParentElement as _, SharedString, StatefulInteractiveElement as _,
    StyleRefinement, Styled, StyledImage as _, Window, div, img, prelude::FluentBuilder as _, px,
    relative, rems,
};

pub use gpui_kit::component::text::TextView as MarkdownView;

use super::embed::{ImagePlaceholder, placeholder_in_box};
use super::theme::UiTheme;

/// Renders Markdown with Nova typography, code blocks, quotes and tables.
///
/// The result is GPUI Kit's `TextView`: chain `plugin`, `on_link_click`,
/// `selectable` or `max_lines` onto it. Keep `id` stable for the message.
pub fn markdown(
    id: impl Into<ElementId>,
    source: impl Into<SharedString>,
    cx: &App,
) -> MarkdownView {
    let theme = UiTheme::read(cx);
    let colors = theme.colors;
    let mut code_block = StyleRefinement::default()
        .p(theme.space(3.))
        .rounded(theme.radius.lg)
        .border_1()
        .border_color(colors.border)
        .bg(colors.muted.opacity(0.5))
        .font_family(theme.fonts.mono.clone())
        .text_size(theme.text(13.));
    // A named code theme brings its own surface, so its colors stay legible.
    if let Some((background, foreground)) = theme.code_theme.surface() {
        code_block = code_block.bg(background).text_color(foreground);
    }
    code_block.overflow.x = Some(gpui_kit::Overflow::Scroll);
    let mut table = StyleRefinement::default()
        .rounded(theme.radius.lg)
        .border_1()
        .border_color(colors.border);
    table.overflow.x = Some(gpui_kit::Overflow::Scroll);
    let style = TextViewStyle::default()
        .paragraph_gap(rems(0.5))
        .code_block(code_block)
        .table(table)
        .table_cell(
            StyleRefinement::default()
                .px(theme.space(2.))
                .py(theme.space(1.5)),
        )
        .inline_code(HighlightStyle {
            background_color: Some(colors.muted.into()),
            ..Default::default()
        });
    TextView::markdown(id, source)
        .style(style)
        .inline_code_style(
            StyleRefinement::default()
                .font_family(theme.fonts.mono.clone())
                .rounded(theme.radius.sm)
                .px(px(3.)),
        )
        .blockquote_style(
            StyleRefinement::default()
                .border_l_2()
                .border_color(colors.border)
                .pl(theme.space(3.))
                .text_color(colors.muted_foreground),
        )
        .list_style(StyleRefinement::default().text_color(colors.muted_foreground))
        .selectable(true)
        .w_full()
        .font_family(theme.fonts.body.clone())
        .text_size(theme.text(14.))
        .line_height(relative(1.45))
        .text_color(colors.foreground)
}

type EntityClick = Arc<dyn Fn(&SharedString, &mut Window, &mut App) + Send + Sync + 'static>;
type EntityHighlight = Arc<dyn Fn(&str) -> bool + Send + Sync + 'static>;

/// Renders links with a custom URL scheme as Nova entity chips, for mentions
/// such as `[@arya](mention:arya)` or channels such as `[~town-square](channel:town-square)`.
///
/// The application rewrites its own syntax (`@arya`, `~town-square`) into these
/// links before rendering, so entity rules stay application-owned.
pub struct EntityLinks {
    scheme: SharedString,
    on_click: Option<EntityClick>,
    highlight: Option<EntityHighlight>,
}

/// The target and label of one entity chip.
#[derive(Clone, Debug)]
struct Entity {
    target: SharedString,
    label: SharedString,
}

impl EntityLinks {
    /// Matches links whose URL starts with `scheme:`, such as `"mention"`.
    pub fn new(scheme: impl Into<SharedString>) -> Self {
        Self {
            scheme: scheme.into(),
            on_click: None,
            highlight: None,
        }
    }

    /// Runs with the link target (the part after `scheme:`) when a chip is clicked.
    pub fn on_click(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + Send + Sync + 'static,
    ) -> Self {
        self.on_click = Some(Arc::new(handler));
        self
    }

    /// Fills chips whose target matches, for example mentions of the reader.
    pub fn highlight(mut self, matches: impl Fn(&str) -> bool + Send + Sync + 'static) -> Self {
        self.highlight = Some(Arc::new(matches));
        self
    }
}

impl MarkdownPlugin for EntityLinks {
    fn name(&self) -> &str {
        &self.scheme
    }

    fn parse(
        &self,
        node: &markdown_ast::Node,
        _cx: &MarkdownParseContext<'_>,
    ) -> Option<MarkdownNode> {
        let markdown_ast::Node::Link(link) = node else {
            return None;
        };
        let target = link
            .url
            .strip_prefix(self.scheme.as_ref())?
            .strip_prefix(':')?;
        let label: String = link
            .children
            .iter()
            .map(|child| child.to_string())
            .collect();
        let entity = Entity {
            target: target.to_string().into(),
            label: label.clone().into(),
        };
        Some(
            MarkdownNode::new(self.scheme.clone(), entity)
                .text(label.clone())
                .accessibility_label(label),
        )
    }

    fn render(&self, node: &MarkdownNode, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let Some(entity) = node.data::<Entity>().cloned() else {
            return div().child(node.as_text().to_string()).into_any_element();
        };
        let theme = UiTheme::read(cx);
        let colors = theme.colors;
        let strong = self
            .highlight
            .as_ref()
            .is_some_and(|matches| matches(&entity.target));
        let on_click = self.on_click.clone();
        let target = entity.target.clone();
        div()
            .id(ElementId::Name(
                format!("{}:{}", self.scheme, entity.target).into(),
            ))
            .px(px(2.))
            .rounded(theme.radius.sm)
            .font_weight(gpui_kit::FontWeight::MEDIUM)
            .bg(if strong {
                colors.mention_highlight()
            } else {
                colors.link().opacity(0.1)
            })
            .text_color(if strong {
                colors.mention_highlight_foreground()
            } else {
                colors.link()
            })
            .when_some(on_click, move |chip, handler| {
                chip.cursor_pointer()
                    .on_click(move |_, window, cx| handler(&target, window, cx))
            })
            .child(entity.label)
            .into_any_element()
    }

    fn render_inline(
        &self,
        node: &MarkdownNode,
        _context: &InlineRenderContext,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<InlineElement> {
        Some(InlineElement::new(self.render(node, window, cx)))
    }
}

type ImageResolver =
    Arc<dyn Fn(&str, &mut Window, &mut App) -> Option<ImageSource> + Send + Sync + 'static>;
type ImageClick = Arc<dyn Fn(&SharedString, &mut Window, &mut App) + Send + Sync + 'static>;

/// One image the application resolves: its key and the text that stands in for it.
#[derive(Clone, Debug)]
struct ResolvedImage {
    key: SharedString,
    alt: SharedString,
}

/// Renders `![:tada:](emoji:tada)` images at the height of the surrounding
/// text, from images the application supplies.
///
/// The application rewrites its emoji syntax (`:tada:`) into these images
/// before rendering. Selection and copy keep the alt text, and an emoji the
/// resolver does not know renders as that text.
pub struct CustomEmoji {
    scheme: SharedString,
    resolve: ImageResolver,
}

impl CustomEmoji {
    /// Matches images whose URL starts with `scheme:`, such as `"emoji"`, and
    /// resolves the name after it, usually from a decoded-image cache.
    pub fn new(
        scheme: impl Into<SharedString>,
        resolve: impl Fn(&str, &mut Window, &mut App) -> Option<ImageSource> + Send + Sync + 'static,
    ) -> Self {
        Self {
            scheme: scheme.into(),
            resolve: Arc::new(resolve),
        }
    }
}

impl MarkdownPlugin for CustomEmoji {
    fn name(&self) -> &str {
        &self.scheme
    }

    fn parse(
        &self,
        node: &markdown_ast::Node,
        _cx: &MarkdownParseContext<'_>,
    ) -> Option<MarkdownNode> {
        let markdown_ast::Node::Image(image) = node else {
            return None;
        };
        let name = image
            .url
            .strip_prefix(self.scheme.as_ref())?
            .strip_prefix(':')?;
        let alt: SharedString = if image.alt.is_empty() {
            format!(":{name}:").into()
        } else {
            image.alt.clone().into()
        };
        Some(
            MarkdownNode::new(
                self.scheme.clone(),
                ResolvedImage {
                    key: name.to_string().into(),
                    alt: alt.clone(),
                },
            )
            .text(alt.clone())
            .accessibility_label(alt),
        )
    }

    fn render(&self, node: &MarkdownNode, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let size = UiTheme::read(cx).text(14.) * 1.25;
        emoji_element(&self.resolve, node, size, window, cx)
    }

    fn render_inline(
        &self,
        node: &MarkdownNode,
        context: &InlineRenderContext,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<InlineElement> {
        let size = context.font_size() * 1.25;
        let element = emoji_element(&self.resolve, node, size, window, cx);
        // Sit the image on the baseline, with the descent below it like text.
        Some(InlineElement::new(element).with_baseline(context.font_size() * 1.05))
    }
}

fn emoji_element(
    resolve: &ImageResolver,
    node: &MarkdownNode,
    size: gpui_kit::Pixels,
    window: &mut Window,
    cx: &mut App,
) -> gpui_kit::AnyElement {
    let Some(emoji) = node.data::<ResolvedImage>().cloned() else {
        return div().child(node.as_text().to_string()).into_any_element();
    };
    match resolve(&emoji.key, window, cx) {
        Some(source) => img(source)
            .size(size)
            .object_fit(ObjectFit::Contain)
            .into_any_element(),
        None => div().child(emoji.alt).into_any_element(),
    }
}

type PlaceholderFor =
    Arc<dyn Fn(&str, &mut Window, &mut App) -> Option<ImagePlaceholder> + Send + Sync>;

/// The widest an inline Markdown image shows.
pub const MARKDOWN_IMAGE_MAX_WIDTH: f32 = 360.;
/// The tallest an inline Markdown image shows, and the height of one whose
/// size is unknown.
pub const MARKDOWN_IMAGE_MAX_HEIGHT: f32 = 200.;

/// The box an inline image of `width` by `height` pixels takes: its own size,
/// scaled down to fit 360 by 200 with its aspect ratio kept.
pub fn markdown_image_box(width: u32, height: u32) -> (f32, f32) {
    let (width, height) = (width.max(1) as f32, height.max(1) as f32);
    let scale = (MARKDOWN_IMAGE_MAX_WIDTH / width)
        .min(MARKDOWN_IMAGE_MAX_HEIGHT / height)
        .min(1.);
    ((width * scale).round(), (height * scale).round())
}

/// Resolves every Markdown image through the application, so nothing is
/// fetched by the view itself.
///
/// The resolver gets the image URL and returns a source, such as a decoded
/// image from the application's media cache. An image it cannot supply yet
/// renders as the `placeholder` the application returns for it, such as a
/// click-to-load [`ImagePlaceholder`], or else as its alt text in a muted
/// frame. Images with a size from [`MarkdownImages::image_size`] take their
/// own box, scaled to fit 360 by 200; others show 200px tall, at most 360px
/// wide. Add `CustomEmoji` first so its images are not taken as ordinary
/// images.
pub struct MarkdownImages {
    resolve: ImageResolver,
    on_click: Option<ImageClick>,
    placeholder: Option<PlaceholderFor>,
    sizes: HashMap<SharedString, (u32, u32)>,
}

impl MarkdownImages {
    /// Resolves image URLs through `resolve`.
    pub fn new(
        resolve: impl Fn(&str, &mut Window, &mut App) -> Option<ImageSource> + Send + Sync + 'static,
    ) -> Self {
        Self {
            resolve: Arc::new(resolve),
            on_click: None,
            placeholder: None,
            sizes: HashMap::new(),
        }
    }

    /// Runs with the image URL when an image is clicked, for opening a lightbox.
    pub fn on_click(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + Send + Sync + 'static,
    ) -> Self {
        self.on_click = Some(Arc::new(handler));
        self
    }

    /// Shows a placeholder for an image the resolver cannot supply yet, such
    /// as `ImagePlaceholder::new(host, load)` when remote images load on
    /// request. Return `None` to show the alt text instead. The placeholder
    /// takes the image's box from [`MarkdownImages::image_size`].
    pub fn placeholder(
        mut self,
        placeholder: impl Fn(&str, &mut Window, &mut App) -> Option<ImagePlaceholder>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        self.placeholder = Some(Arc::new(placeholder));
        self
    }

    /// Gives an image URL's pixel size, such as from Mattermost's
    /// `metadata.images`, so the image and its placeholder reserve the right
    /// box before it loads.
    pub fn image_size(mut self, url: impl Into<SharedString>, width: u32, height: u32) -> Self {
        self.sizes.insert(url.into(), (width, height));
        self
    }

    /// Gives several image sizes; see [`MarkdownImages::image_size`].
    pub fn image_sizes(
        mut self,
        sizes: impl IntoIterator<Item = (impl Into<SharedString>, u32, u32)>,
    ) -> Self {
        for (url, width, height) in sizes {
            self.sizes.insert(url.into(), (width, height));
        }
        self
    }
}

impl MarkdownPlugin for MarkdownImages {
    fn name(&self) -> &str {
        "image"
    }

    fn parse(
        &self,
        node: &markdown_ast::Node,
        _cx: &MarkdownParseContext<'_>,
    ) -> Option<MarkdownNode> {
        let markdown_ast::Node::Image(image) = node else {
            return None;
        };
        let alt: SharedString = if image.alt.is_empty() {
            "Image".into()
        } else {
            image.alt.clone().into()
        };
        Some(
            MarkdownNode::new(
                "image",
                ResolvedImage {
                    key: image.url.clone().into(),
                    alt: alt.clone(),
                },
            )
            .text(alt.clone())
            .accessibility_label(alt),
        )
    }

    fn render(&self, node: &MarkdownNode, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let Some(image) = node.data::<ResolvedImage>().cloned() else {
            return div().child(node.as_text().to_string()).into_any_element();
        };
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let source = (self.resolve)(&image.key, window, cx);
        let known = self.sizes.get(&image.key).copied();
        let id = ElementId::Name(format!("image:{}", image.key).into());
        if source.is_none()
            && let Some(placeholder) = self
                .placeholder
                .as_ref()
                .and_then(|placeholder| placeholder(&image.key, window, cx))
        {
            // 16:9 at the usual height when nothing gives the size.
            let (width, height) = known
                .or(placeholder.known_size())
                .map(|(w, h)| markdown_image_box(w, h))
                .unwrap_or((356., MARKDOWN_IMAGE_MAX_HEIGHT));
            return div()
                .my(theme.space(1.))
                .child(placeholder_in_box(
                    (id, "placeholder"),
                    placeholder,
                    px(width),
                    px(height),
                    window,
                    cx,
                ))
                .into_any_element();
        }
        let on_click = self.on_click.clone();
        let key = image.key.clone();
        div()
            .id(id)
            .my(theme.space(1.))
            .rounded(theme.radius.lg)
            .overflow_hidden()
            .border_1()
            .border_color(colors.border)
            .when_some(on_click, move |frame, handler| {
                frame
                    .cursor_pointer()
                    .on_click(move |_, window, cx| handler(&key, window, cx))
            })
            .map(|frame| match (source, known) {
                (Some(source), Some((width, height))) => {
                    let (width, height) = markdown_image_box(width, height);
                    frame.child(
                        img(source)
                            .debug_selector(|| "markdown-image".into())
                            .w(px(width))
                            .h(px(height))
                            .max_w_full()
                            .rounded(theme.radius.lg - px(1.))
                            .object_fit(ObjectFit::Contain),
                    )
                }
                (Some(source), None) => frame.child(
                    img(source)
                        .debug_selector(|| "markdown-image".into())
                        .h(px(MARKDOWN_IMAGE_MAX_HEIGHT))
                        .max_w(px(MARKDOWN_IMAGE_MAX_WIDTH))
                        .rounded(theme.radius.lg - px(1.))
                        .object_fit(ObjectFit::Contain),
                ),
                (None, _) => frame
                    .px(theme.space(3.))
                    .py(theme.space(2.))
                    .bg(colors.muted)
                    .text_size(theme.text(13.))
                    .text_color(colors.muted_foreground)
                    .child(image.alt),
            })
            .into_any_element()
    }
}

/// The window's root for text selection. Render it once around the window's
/// content; Markdown views and `selectable_text` inside it then share one
/// selection, and Cmd+C (Ctrl+C elsewhere) copies it in reading order.
///
/// A focused text field keeps its own copy shortcut.
pub fn selection_root(id: impl Into<ElementId>) -> gpui_kit::Stateful<gpui_kit::Div> {
    div()
        .id(id)
        .key_context("SelectionRoot")
        .size_full()
        .relative()
        // GPUI Kit's views copy the raw selection themselves; a selection
        // across messages with copy prefixes is copied here first.
        .capture_action(|_: &gpui_kit::base::input::Copy, window, cx| {
            let text = TextSelection::selected_text(window, cx);
            if super::message::has_copy_groups(&text) && copy_selection(window, cx) {
                cx.stop_propagation();
            }
        })
        .on_key_down(|event, window, cx| {
            let keystroke = &event.keystroke;
            if keystroke.key != "c"
                || !keystroke.modifiers.secondary(cx)
                || keystroke.modifiers.shift
                || keystroke.modifiers.alt
            {
                return;
            }
            if copy_selection(window, cx) {
                cx.stop_propagation();
            }
        })
        .child(TextSelectionLayer)
}

/// Copies the window's text selection to the clipboard. Returns `false` when
/// nothing is selected.
///
/// When the selection covers several messages with a
/// [`Message::copy_prefix`](super::message::Message::copy_prefix), each
/// message is one line that starts with its prefix.
pub fn copy_selection(window: &mut Window, cx: &mut App) -> bool {
    let text = super::message::apply_copy_prefixes(&TextSelection::selected_text(window, cx));
    let text = text.trim();
    if text.is_empty() {
        return false;
    }
    cx.write_to_clipboard(ClipboardItem::new_string(text.to_string()));
    true
}

/// Plain text that joins the window selection, for message bodies that are
/// not Markdown. Keep `id` stable for the message.
pub fn selectable_text(
    id: impl Into<ElementId>,
    text: impl Into<SharedString>,
    cx: &App,
) -> impl IntoElement {
    let theme = UiTheme::read(cx);
    div()
        .w_full()
        .text_size(theme.text(14.))
        .text_color(theme.colors.foreground)
        .child(SelectableText::new(id, text))
}

#[cfg(test)]
mod image_tests {
    use super::*;
    use gpui_kit::{Context, Render, TestAppContext, VisualTestContext, test::TestQueryExt as _};
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn known_sizes_scale_to_fit_and_keep_their_shape() {
        assert_eq!(markdown_image_box(1200, 600), (360., 180.));
        assert_eq!(markdown_image_box(400, 800), (100., 200.));
        assert_eq!(markdown_image_box(64, 48), (64., 48.));
    }

    struct Post {
        loads: Arc<AtomicUsize>,
    }

    impl Render for Post {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let loads = self.loads.clone();
            div().w(px(480.)).child(
                markdown(
                    "post",
                    "Look:\n\n![Chart](https://cdn.example.com/chart.png)",
                    cx,
                )
                .plugin(
                    MarkdownImages::new(|_, _, _| None)
                        .image_size("https://cdn.example.com/chart.png", 1200, 600)
                        .placeholder(move |url, _, _| {
                            let loads = loads.clone();
                            url.starts_with("https://cdn.example.com/").then(|| {
                                ImagePlaceholder::new("cdn.example.com", move |_, _, _| {
                                    loads.fetch_add(1, Ordering::SeqCst);
                                })
                            })
                        }),
                ),
            )
        }
    }

    #[gpui_kit::test]
    fn unloaded_images_show_a_placeholder_in_their_box(cx: &mut TestAppContext) {
        cx.update(super::super::theme::init);
        let loads = Arc::new(AtomicUsize::new(0));
        let window = cx.add_window({
            let loads = loads.clone();
            move |_, _| Post { loads }
        });
        let mut cx = VisualTestContext::from_window(window.into(), cx);
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let frame = cx
            .debug_bounds("markdown-image-placeholder")
            .expect("the placeholder is drawn");
        assert_eq!(frame.size.width, px(360.));
        assert_eq!(frame.size.height, px(180.));
        let button = cx.update(|window, _| window.find_by_label("Load image from cdn.example.com"));
        cx.simulate_click(button.bounds().center(), gpui_kit::Modifiers::default());
        assert_eq!(loads.load(Ordering::SeqCst), 1);
    }
}
