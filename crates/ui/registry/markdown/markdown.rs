//! Nova-styled Markdown, rendered by GPUI Kit's selectable rich text view.
//!
//! Code blocks are syntax-highlighted when the application enables GPUI Kit's
//! `syntect` (or `tree-sitter`) feature; without it they render as plain
//! monospaced text with the same frame.

use std::sync::Arc;

use gpui_kit::component::text::{
    InlineElement, InlineRenderContext, MarkdownNode, MarkdownParseContext, MarkdownPlugin,
    TextView, TextViewStyle, markdown_ast,
};
use gpui_kit::{
    App, ElementId, HighlightStyle, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, StatefulInteractiveElement as _, StyleRefinement, Styled, Window, div,
    prelude::FluentBuilder as _, px, relative, rems,
};

pub use gpui_kit::component::text::TextView as MarkdownView;

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
                colors.primary
            } else {
                colors.primary.opacity(0.1)
            })
            .text_color(if strong {
                colors.primary_foreground
            } else {
                colors.primary
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
