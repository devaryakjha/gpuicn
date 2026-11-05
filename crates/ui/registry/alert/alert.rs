//! Nova Alert: an inline callout with an icon, title and description.
//!
//! Visual source: shadcn/ui 4.19.0 `alert.tsx`, Nova style, plus warning,
//! info and success treatments that read the theme's status tokens. It is
//! plain layout, not a GPUI Kit control: nothing in it takes focus. Use it
//! above a form for an error the server returned, or for a settings note.

use gpui_icons::{LucideIcon, lucide};
use gpui_kit::{
    AnyElement, App, ElementId, FontWeight, InteractiveElement as _, IntoElement, ParentElement,
    RenderOnce, Rgba, Role, SharedString, StatefulInteractiveElement as _, StyleRefinement, Styled,
    Window, div, prelude::FluentBuilder as _,
};

use gpui_kit::base::TestSupportExt as _;

use super::theme::{UiTheme, apply_style, locale::DirectionalStyled as _, primitives::text};

/// The Alert's color treatment.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AlertVariant {
    /// Card colors, for a neutral note.
    #[default]
    Default,
    /// Destructive text and icon, for an error. Announced as an alert.
    Destructive,
    /// The `warning` color, for a caution. Announced as an alert.
    Warning,
    /// The `info` color, for a hint.
    Info,
    /// The `success` color, for a confirmation.
    Success,
}

impl AlertVariant {
    /// The icon shown when none is set: none for `Default`.
    pub fn default_icon(self) -> Option<LucideIcon> {
        match self {
            Self::Default => None,
            Self::Destructive => Some(LucideIcon::CircleAlert),
            Self::Warning => Some(LucideIcon::TriangleAlert),
            Self::Info => Some(LucideIcon::Info),
            Self::Success => Some(LucideIcon::CircleCheck),
        }
    }

    fn accent(self, theme: &UiTheme) -> Option<Rgba> {
        let colors = theme.colors;
        match self {
            Self::Default => None,
            Self::Destructive => Some(colors.destructive),
            Self::Warning => Some(colors.warning),
            Self::Info => Some(colors.info),
            Self::Success => Some(colors.success),
        }
    }
}

/// An inline, non-modal callout: an icon, a one-line title, a description
/// and optional children such as a retry button, on the card surface.
///
/// Destructive and warning alerts are announced when they appear; the
/// others are read as a status. The title is the accessible name and the
/// description its description.
#[derive(IntoElement)]
pub struct Alert {
    id: ElementId,
    variant: AlertVariant,
    icon: Option<Option<LucideIcon>>,
    title: Option<SharedString>,
    description: Option<SharedString>,
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl Alert {
    /// Creates an alert with a stable caller-owned ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            variant: AlertVariant::Default,
            icon: None,
            title: None,
            description: None,
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }

    /// Selects the color treatment, and the icon unless one is set.
    pub fn variant(mut self, variant: AlertVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Shows a Lucide icon instead of the variant's.
    pub fn icon(mut self, icon: LucideIcon) -> Self {
        self.icon = Some(Some(icon));
        self
    }

    /// Shows no icon, whatever the variant.
    pub fn no_icon(mut self) -> Self {
        self.icon = Some(None);
        self
    }

    /// Sets the one-line title in medium weight.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the muted description under the title.
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }
}

impl ParentElement for Alert {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Styled for Alert {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Alert {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let accent = self.variant.accent(&theme);
        let icon = self.icon.unwrap_or(self.variant.default_icon());
        let announced = matches!(
            self.variant,
            AlertVariant::Destructive | AlertVariant::Warning
        );
        let description_color = match accent {
            Some(accent) if self.variant == AlertVariant::Destructive => accent.opacity(0.9),
            _ => colors.muted_foreground,
        };
        let body = div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .gap(theme.space(0.5))
            .when_some(self.title.clone(), |body, title| {
                body.child(
                    div()
                        .min_h(theme.space(4.))
                        .overflow_hidden()
                        .text_ellipsis()
                        .whitespace_nowrap()
                        .font_weight(FontWeight::MEDIUM)
                        .child(title),
                )
            })
            .when_some(self.description.clone(), |body, description| {
                body.child(div().text_color(description_color).child(description))
            })
            .when(!self.children.is_empty(), |body| {
                body.child(
                    div()
                        .pt(theme.space(1.5))
                        .flex_row_in(theme.direction)
                        .flex_wrap()
                        .items_center()
                        .gap(theme.space(2.))
                        .children(self.children),
                )
            });
        let root = text::sm(div(), &theme)
            .id(self.id)
            .test_support()
            .role(if announced { Role::Alert } else { Role::Status })
            .when_some(self.title, |root, title| root.aria_label(title))
            .when_some(self.description, |root, description| {
                root.aria_description(description)
            })
            .w_full()
            .flex_row_in(theme.direction)
            .items_start()
            .gap(theme.space(3.))
            .px(theme.space(4.))
            .py(theme.space(3.))
            .rounded(theme.radius.lg)
            .border_1()
            .border_color(colors.border)
            .bg(colors.card)
            .text_color(accent.unwrap_or(colors.card_foreground))
            .when_some(icon, |root, icon| {
                root.child(
                    lucide(icon)
                        .flex_shrink_0()
                        .mt(theme.space(0.5))
                        .size(theme.space(4.))
                        .text_color(accent.unwrap_or(colors.card_foreground)),
                )
            })
            .child(body);
        apply_style(root, &self.style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{Context, Render, TestAppContext, VisualTestContext, test::TestQueryExt as _};

    struct Probe;

    impl Render for Probe {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .child(
                    Alert::new("refused")
                        .variant(AlertVariant::Destructive)
                        .title("The integration refused this")
                        .description("Check the channel name and try again."),
                )
                .child(Alert::new("note").title("Theme sync is on"))
        }
    }

    #[gpui_kit::test]
    fn reads_as_an_alert_or_a_status_in_both_themes(cx: &mut TestAppContext) {
        cx.update(super::super::theme::init);
        let window = cx.add_window(|_, _| Probe);
        let mut cx = VisualTestContext::from_window(window.into(), cx);
        for mode in [
            super::super::theme::ThemeMode::Light,
            super::super::theme::ThemeMode::Dark,
        ] {
            cx.update(|_, cx| UiTheme::switch(cx, mode));
            cx.update(|window, cx| window.draw(cx).clear(cx));
            cx.update(|window, _| {
                let refused = window.find_by_label("The integration refused this");
                assert_eq!(refused.role(), Some(Role::Alert));
                assert_eq!(
                    refused.description(),
                    Some("Check the channel name and try again.")
                );
                assert_eq!(
                    window.find_by_label("Theme sync is on").role(),
                    Some(Role::Status)
                );
            });
        }
    }

    #[test]
    fn variants_pick_their_icon() {
        assert_eq!(AlertVariant::Default.default_icon(), None);
        assert_eq!(
            AlertVariant::Warning.default_icon(),
            Some(LucideIcon::TriangleAlert)
        );
    }
}
