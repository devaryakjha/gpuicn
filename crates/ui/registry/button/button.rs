//! The shadcn Nova Button visual port.
//!
//! Visual source: shadcn/ui 4.19.0 `button.tsx` and `style-nova.css` at
//! `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`. Interaction comes from the
//! GPUI Kit headless button.

use std::rc::Rc;

use gpui_kit::base::Button as BaseButton;
use gpui_kit::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, IntoElement, ParentElement, RenderOnce,
    Rgba, SharedString, Styled, Window, px,
};

use super::theme::{
    UiRadius, UiTheme,
    locale::DirectionalStyled as _,
    primitives::{disabled as disabled_style, focus},
};

type ButtonClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
type ButtonStyleHandler = Rc<dyn Fn(BaseButton) -> BaseButton>;

/// The pinned shadcn Button visual variant.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonVariant {
    /// Primary action.
    #[default]
    Default,
    /// Bordered action.
    Outline,
    /// Secondary action.
    Secondary,
    /// Low-emphasis action.
    Ghost,
    /// Destructive action.
    Destructive,
    /// Link-styled action.
    Link,
}

/// The pinned shadcn Button size.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonSize {
    /// 24px-high extra-small button.
    Xs,
    /// 28px-high small button.
    Sm,
    /// 32px-high default button.
    #[default]
    Default,
    /// 36px-high large button.
    Lg,
    /// 24px square icon button.
    IconXs,
    /// 28px square icon button.
    IconSm,
    /// 32px square icon button.
    Icon,
    /// 36px square icon button.
    IconLg,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ButtonMetrics {
    height: f32,
    horizontal_padding: f32,
    gap: f32,
    radius: f32,
    text_size: f32,
    icon_only: bool,
}

impl ButtonSize {
    fn metrics(self, radii: UiRadius) -> ButtonMetrics {
        let radius = f32::from(radii.lg);
        let radius_md = f32::from(radii.md);
        match self {
            Self::Xs => ButtonMetrics::new(24.0, 8.0, 4.0, radius_md.min(10.0), 12.0, false),
            Self::Sm => ButtonMetrics::new(28.0, 10.0, 4.0, radius_md.min(12.0), 12.8, false),
            Self::Default => ButtonMetrics::new(32.0, 10.0, 6.0, radius, 14.0, false),
            Self::Lg => ButtonMetrics::new(36.0, 10.0, 6.0, radius, 14.0, false),
            Self::IconXs => ButtonMetrics::new(24.0, 0.0, 0.0, radius_md.min(10.0), 14.0, true),
            Self::IconSm => ButtonMetrics::new(28.0, 0.0, 0.0, radius_md.min(12.0), 14.0, true),
            Self::Icon => ButtonMetrics::new(32.0, 0.0, 0.0, radius, 14.0, true),
            Self::IconLg => ButtonMetrics::new(36.0, 0.0, 0.0, radius, 14.0, true),
        }
    }
}

impl ButtonMetrics {
    const fn new(
        height: f32,
        horizontal_padding: f32,
        gap: f32,
        radius: f32,
        text_size: f32,
        icon_only: bool,
    ) -> Self {
        Self {
            height,
            horizontal_padding,
            gap,
            radius,
            text_size,
            icon_only,
        }
    }
}

/// A styled Button that keeps GPUI Kit's pointer and keyboard behavior.
#[derive(IntoElement)]
pub struct Button {
    style: gpui_kit::StyleRefinement,
    id: ElementId,
    variant: ButtonVariant,
    size: ButtonSize,
    disabled: bool,
    aria_label: Option<SharedString>,
    on_click: Option<ButtonClickHandler>,
    style_with: Option<ButtonStyleHandler>,
    tint: Option<Rgba>,
    children: Vec<AnyElement>,
}

impl Button {
    /// Creates a Button with a caller-owned stable ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            style: gpui_kit::StyleRefinement::default(),
            id: id.into(),
            variant: ButtonVariant::default(),
            size: ButtonSize::default(),
            disabled: false,
            aria_label: None,
            on_click: None,
            style_with: None,
            tint: None,
            children: Vec::new(),
        }
    }

    /// Sets the visual variant.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the visual size.
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Prevents activation and removes the Button from tab order.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the accessible name for icon or custom content. Use `label` for plain text.
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// Adds visible text and uses it as the accessible name unless one was supplied.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.aria_label.get_or_insert_with(|| label.clone());
        self.children.push(label.into_any_element());
        self
    }

    /// Runs for pointer, Enter, or Space activation.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    /// Fills the button with one app-chosen color, such as an integration's
    /// `#3f51b5`. Hover moves the fill toward the text, and the text is the
    /// theme's foreground or background, whichever reads better on the fill,
    /// so it keeps its contrast in light and dark mode. Replaces the variant.
    pub fn tint(mut self, color: impl Into<Rgba>) -> Self {
        self.tint = Some(color.into());
        self
    }

    /// Replaces the visual styling while retaining button interaction.
    /// Include a visible keyboard focus treatment in custom styles.
    pub fn style_with(mut self, handler: impl Fn(BaseButton) -> BaseButton + 'static) -> Self {
        self.style_with = Some(Rc::new(handler));
        self
    }
}

impl ParentElement for Button {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let variant = self.variant;
        let size = self.size;
        let root = BaseButton::new(self.id).disabled(self.disabled);
        let root = match (self.style_with, self.tint) {
            (Some(style), _) => style(root),
            (None, Some(tint)) => style_tinted_button(root, self.disabled, tint, size, &theme),
            (None, None) => style_button(root, self.disabled, variant, size, &theme),
        };
        let mut root = super::theme::apply_style(root, &self.style).children(self.children);

        if let Some(label) = self.aria_label {
            root = root.accessibility_label(label);
        }
        if let Some(handler) = self.on_click {
            root = root.on_click(move |event, window, cx| handler(event, window, cx));
        }

        root
    }
}

#[doc(hidden)]
pub fn style_button<T: Styled + gpui_kit::InteractiveElement + gpui_kit::prelude::FluentBuilder>(
    base: T,
    disabled: bool,
    variant: ButtonVariant,
    size: ButtonSize,
    theme: &UiTheme,
) -> T {
    style_button_with(base, disabled, variant, None, size, theme)
}

fn style_button_with<
    T: Styled + gpui_kit::InteractiveElement + gpui_kit::prelude::FluentBuilder,
>(
    base: T,
    disabled: bool,
    variant: ButtonVariant,
    tint: Option<Rgba>,
    size: ButtonSize,
    theme: &UiTheme,
) -> T {
    let colors = theme.colors;
    let control = theme.control();
    let metrics = size.metrics(theme.radius);
    let invalid_ring = variant == ButtonVariant::Destructive;
    let focus_theme = theme.clone();

    let base = base
        .flex_row_in(theme.direction)
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .whitespace_nowrap()
        .h(theme.space(metrics.height / 4.))
        .gap(theme.space(metrics.gap / 4.))
        .rounded(px(metrics.radius))
        .border_1()
        .border_color(colors.background.opacity(0.0))
        .font_family(theme.fonts.body.clone())
        .font_weight(FontWeight::MEDIUM)
        .text_size(theme.text(metrics.text_size))
        .focus_visible(move |style| focus::ring_for(style, &focus_theme, invalid_ring))
        .when(metrics.icon_only, |base| {
            base.w(theme.space(metrics.height / 4.)).p_0()
        })
        .when(!metrics.icon_only, |base| {
            base.px(theme.space(metrics.horizontal_padding / 4.))
        });

    if let Some(tint) = tint {
        let (fill, hover, foreground) = tint_colors(tint, theme);
        let base = base
            .bg(fill)
            .text_color(foreground)
            .when(!disabled, |base| base.hover(move |style| style.bg(hover)));
        return disabled_style::interactive(base, disabled);
    }
    let base = match variant {
        ButtonVariant::Default => base
            .bg(colors.primary)
            .text_color(colors.primary_foreground)
            .when(!disabled, |base| {
                base.hover(move |style| style.bg(colors.primary.opacity(0.80)))
            }),
        ButtonVariant::Outline => base
            .bg(control.background)
            .text_color(colors.foreground)
            .border_color(control.border)
            .when(!disabled, |base| {
                base.hover(move |style| style.bg(control.hover).text_color(colors.foreground))
            }),
        ButtonVariant::Secondary => {
            let hover = colors
                .secondary
                .blend(colors.secondary_foreground.opacity(0.08));
            base.bg(colors.secondary)
                .text_color(colors.secondary_foreground)
                .when(!disabled, |base| base.hover(move |style| style.bg(hover)))
        }
        ButtonVariant::Ghost => base
            .bg(colors.background.opacity(0.0))
            .text_color(colors.foreground)
            .when(!disabled, |base| {
                base.hover(move |style| style.bg(control.ghost_hover).text_color(colors.foreground))
            }),
        ButtonVariant::Destructive => base
            .bg(control.destructive_background)
            .text_color(colors.destructive)
            .when(!disabled, |base| {
                base.hover(move |style| style.bg(control.destructive_hover))
            }),
        ButtonVariant::Link => base
            .bg(colors.background.opacity(0.0))
            .text_color(colors.primary)
            .when(!disabled, |base| base.hover(|style| style.underline())),
    };

    disabled_style::interactive(base, disabled)
}

/// WCAG relative luminance of an opaque color.
fn luminance(color: Rgba) -> f32 {
    let channel = |c: f32| {
        if c <= 0.039_28 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(color.r) + 0.7152 * channel(color.g) + 0.0722 * channel(color.b)
}

/// The WCAG contrast ratio between two opaque colors, from 1 to 21.
pub fn contrast_ratio(a: Rgba, b: Rgba) -> f32 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// The fill, hover fill and text of a button tinted with `tint`.
pub fn tint_colors(tint: Rgba, theme: &UiTheme) -> (Rgba, Rgba, Rgba) {
    let fill = theme.opaque(tint);
    let colors = theme.colors;
    let foreground =
        if contrast_ratio(fill, colors.foreground) >= contrast_ratio(fill, colors.background) {
            colors.foreground
        } else {
            colors.background
        };
    let hover = fill.blend(foreground.opacity(0.14));
    (fill, hover, foreground)
}

#[doc(hidden)]
pub fn style_tinted_button<
    T: Styled + gpui_kit::InteractiveElement + gpui_kit::prelude::FluentBuilder,
>(
    base: T,
    disabled: bool,
    tint: Rgba,
    size: ButtonSize,
    theme: &UiTheme,
) -> T {
    style_button_with(
        base,
        disabled,
        ButtonVariant::Default,
        Some(tint),
        size,
        theme,
    )
}

impl gpui_kit::Styled for Button {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tints_pick_readable_text_in_both_themes() {
        let indigo = Rgba::try_from("#3f51b5").unwrap();
        let lime = Rgba::try_from("#cddc39").unwrap();
        for theme in [UiTheme::neutral_light(), UiTheme::neutral_dark()] {
            for tint in [indigo, lime] {
                let (fill, hover, text) = tint_colors(tint, &theme);
                assert!(contrast_ratio(fill, text) >= 4.5, "{tint:?} reads");
                assert_ne!(fill, hover);
            }
            let (_, _, text) = tint_colors(indigo, &theme);
            assert!(luminance(text) > 0.5, "indigo takes light text");
            let (_, _, text) = tint_colors(lime, &theme);
            assert!(luminance(text) < 0.1, "lime takes dark text");
        }
    }

    #[test]
    fn all_variants_expose_clickable_and_disabled_cursors() {
        for theme in [UiTheme::neutral_light(), UiTheme::neutral_dark()] {
            for variant in [
                ButtonVariant::Default,
                ButtonVariant::Outline,
                ButtonVariant::Secondary,
                ButtonVariant::Ghost,
                ButtonVariant::Destructive,
                ButtonVariant::Link,
            ] {
                for disabled in [false, true] {
                    let mut button = style_button(
                        gpui_kit::div(),
                        disabled,
                        variant,
                        ButtonSize::Default,
                        &theme,
                    );
                    assert_eq!(
                        button.style().mouse_cursor,
                        Some(if disabled {
                            gpui_kit::CursorStyle::OperationNotAllowed
                        } else {
                            gpui_kit::CursorStyle::PointingHand
                        })
                    );
                }
            }
        }
    }

    #[test]
    fn nova_sizes_are_exact() {
        let sizes = [
            (ButtonSize::Xs, 24.0, 8.0, 4.0, 8.0),
            (ButtonSize::Sm, 28.0, 10.0, 4.0, 8.0),
            (ButtonSize::Default, 32.0, 10.0, 6.0, 10.0),
            (ButtonSize::Lg, 36.0, 10.0, 6.0, 10.0),
            (ButtonSize::IconXs, 24.0, 0.0, 0.0, 8.0),
            (ButtonSize::IconSm, 28.0, 0.0, 0.0, 8.0),
            (ButtonSize::Icon, 32.0, 0.0, 0.0, 10.0),
            (ButtonSize::IconLg, 36.0, 0.0, 0.0, 10.0),
        ];

        for (size, height, padding, gap, radius) in sizes {
            let metrics = size.metrics(UiRadius::new(px(10.0)));
            assert_eq!(metrics.height, height);
            assert_eq!(metrics.horizontal_padding, padding);
            assert_eq!(metrics.gap, gap);
            assert_eq!(metrics.radius, radius);
        }
    }
}
