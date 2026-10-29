//! Nova-styled tags for statuses, roles and removable filters.

use std::rc::Rc;

use gpui_icons::{LucideIcon, lucide};
use gpui_kit::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement as _, IntoElement, ParentElement,
    RenderOnce, Rgba, SharedString, Styled, Window, div, prelude::FluentBuilder as _, relative,
};

use super::button::{Button, ButtonSize, ButtonVariant};
use super::theme::UiTheme;

type RemoveHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
/// Nova tag color treatments.
pub enum TagVariant {
    #[default]
    /// Muted fill with a hairline border.
    Secondary,
    /// Transparent with a border.
    Outline,
    /// Tinted destructive treatment.
    Destructive,
}

/// A compact rectangular chip for a status, role or filter.
#[derive(IntoElement)]
pub struct Tag {
    style: gpui_kit::StyleRefinement,
    id: ElementId,
    label: SharedString,
    variant: TagVariant,
    icon: Option<LucideIcon>,
    dot: Option<Rgba>,
    on_remove: Option<RemoveHandler>,
}

impl Tag {
    /// Creates a `Tag` with a stable caller-owned ID.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            style: gpui_kit::StyleRefinement::default(),
            id: id.into(),
            label: label.into(),
            variant: TagVariant::Secondary,
            icon: None,
            dot: None,
            on_remove: None,
        }
    }

    /// Selects the Nova color treatment.
    pub fn variant(mut self, variant: TagVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Adds a leading Lucide icon.
    pub fn icon(mut self, icon: LucideIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Adds a leading status dot in `color`, for presence or state.
    pub fn dot(mut self, color: impl Into<Rgba>) -> Self {
        self.dot = Some(color.into());
        self
    }

    /// Adds a trailing remove button, announced as "Remove" plus the label.
    pub fn on_remove(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_remove = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Tag {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let (background, foreground, border) = match self.variant {
            TagVariant::Secondary => (colors.muted, colors.foreground, colors.border),
            TagVariant::Outline => (
                colors.background.opacity(0.),
                colors.foreground,
                colors.border,
            ),
            TagVariant::Destructive => (
                colors.destructive.opacity(0.1),
                colors.destructive,
                colors.destructive.opacity(0.2),
            ),
        };
        let remove_label: SharedString = format!("Remove {}", self.label).into();
        let root = div()
            .id(self.id.clone())
            .flex()
            .flex_shrink_0()
            .items_center()
            .gap(theme.space(1.5))
            .h(theme.space(6.))
            .px(theme.space(2.))
            .when(self.on_remove.is_some(), |root| root.pr(theme.space(1.)))
            .rounded(theme.radius.md)
            .border_1()
            .border_color(border)
            .bg(background)
            .text_color(foreground)
            .font_family(theme.fonts.body.clone())
            .font_weight(FontWeight::MEDIUM)
            .text_size(theme.text(12.))
            .line_height(relative(1.))
            .whitespace_nowrap()
            .when_some(self.dot, |root, color| {
                root.child(
                    div()
                        .size(theme.space(1.5))
                        .flex_shrink_0()
                        .rounded_full()
                        .bg(color),
                )
            })
            .when_some(self.icon, |root, icon| {
                root.child(
                    lucide(icon)
                        .size(theme.space(3.5))
                        .flex_shrink_0()
                        .text_color(foreground),
                )
            })
            .child(self.label)
            .when_some(self.on_remove, |root, handler| {
                root.child(
                    Button::new((self.id, "remove"))
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconXs)
                        .w(theme.space(4.))
                        .h(theme.space(4.))
                        .rounded(theme.radius.sm)
                        .aria_label(remove_label)
                        .on_click(move |event, window, cx| handler(event, window, cx))
                        .child(
                            lucide(LucideIcon::X)
                                .size(theme.space(3.))
                                .text_color(foreground),
                        ),
                )
            });
        super::theme::apply_style(root, &self.style)
    }
}

impl Styled for Tag {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}
