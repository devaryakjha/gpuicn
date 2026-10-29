//! Nova-styled badges for labels and counts.

use gpui_icons::{LucideIcon, lucide};
use gpui_kit::{
    App, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::FluentBuilder as _, px, relative,
};

use super::theme::UiTheme;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
/// Nova badge color treatments.
pub enum BadgeVariant {
    #[default]
    /// Primary fill, for the most important count or label.
    Default,
    /// Neutral secondary fill.
    Secondary,
    /// Tinted destructive treatment, for errors and urgent counts.
    Destructive,
    /// Transparent with a border.
    Outline,
    /// Text only, with no fill or border.
    Ghost,
}

/// A small, noninteractive pill that labels or counts something.
#[derive(IntoElement)]
pub struct Badge {
    style: gpui_kit::StyleRefinement,
    label: SharedString,
    icon: Option<LucideIcon>,
    variant: BadgeVariant,
}

impl Badge {
    /// Creates a `Badge` showing `label`.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            style: gpui_kit::StyleRefinement::default(),
            label: label.into(),
            icon: None,
            variant: BadgeVariant::Default,
        }
    }

    /// Creates a count `Badge`, showing `max+` once `count` exceeds `max`.
    pub fn count(count: usize, max: usize) -> Self {
        Self::new(format_count(count, max))
    }

    /// Selects the Nova color treatment.
    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Adds a leading Lucide icon.
    pub fn icon(mut self, icon: LucideIcon) -> Self {
        self.icon = Some(icon);
        self
    }
}

/// Formats an unread or mention count, capping it at `max+`.
pub fn format_count(count: usize, max: usize) -> SharedString {
    if count > max {
        format!("{max}+").into()
    } else {
        count.to_string().into()
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx);
        let colors = theme.colors;
        let (background, foreground, border) = match self.variant {
            BadgeVariant::Default => (colors.primary, colors.primary_foreground, None),
            BadgeVariant::Secondary => (colors.secondary, colors.secondary_foreground, None),
            BadgeVariant::Destructive => {
                (colors.destructive.opacity(0.1), colors.destructive, None)
            }
            BadgeVariant::Outline => (
                colors.background.opacity(0.),
                colors.foreground,
                Some(colors.border),
            ),
            BadgeVariant::Ghost => (colors.background.opacity(0.), colors.foreground, None),
        };
        let root = div()
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .gap(theme.space(1.))
            .h(theme.space(5.))
            .min_w(theme.space(5.))
            .px(theme.space(2.))
            .rounded(px(9999.))
            .border_1()
            .border_color(border.unwrap_or(background.opacity(0.)))
            .bg(background)
            .text_color(foreground)
            .font_family(theme.fonts.body.clone())
            .font_weight(FontWeight::MEDIUM)
            .text_size(theme.text(12.))
            .line_height(relative(1.))
            .whitespace_nowrap()
            .overflow_hidden()
            .when_some(self.icon, |root, icon| {
                root.child(
                    lucide(icon)
                        .size(theme.space(3.))
                        .flex_shrink_0()
                        .text_color(foreground),
                )
            })
            .child(self.label);
        super::theme::apply_style(root, &self.style)
    }
}

impl Styled for Badge {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}

#[cfg(test)]
mod tests {
    use super::format_count;

    #[test]
    fn counts_cap_at_max() {
        assert_eq!(format_count(0, 99).as_ref(), "0");
        assert_eq!(format_count(99, 99).as_ref(), "99");
        assert_eq!(format_count(100, 99).as_ref(), "99+");
    }
}
