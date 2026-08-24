//! The shadcn Nova Button visual port.
//!
//! Visual source: shadcn/ui 4.19.0 `button.tsx` and `style-nova.css` at
//! `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`. Interaction comes from the
//! pinned Base GPUI `ButtonRoot`.

use std::rc::Rc;

use base_gpui::button::ButtonRoot;
use gpui::{
    AnyElement, App, BoxShadow, ClickEvent, Div, ElementId, FontWeight, InteractiveElement as _,
    IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    prelude::FluentBuilder as _, px,
};

use super::theme::{ThemeMode, UiTheme, neutral};

type ButtonClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

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
    fn metrics(self, radius: f32) -> ButtonMetrics {
        let radius_md = radius * 0.8;
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

/// A styled Button that keeps Base GPUI's pointer and keyboard behavior.
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    variant: ButtonVariant,
    size: ButtonSize,
    disabled: bool,
    aria_label: Option<SharedString>,
    on_click: Option<ButtonClickHandler>,
    children: Vec<AnyElement>,
}

impl Button {
    /// Creates a Button with a caller-owned stable ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            variant: ButtonVariant::default(),
            size: ButtonSize::default(),
            disabled: false,
            aria_label: None,
            on_click: None,
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

    /// Sets the accessible name, required for icon-only Buttons.
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
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
        let mut root = ButtonRoot::new()
            .id(self.id)
            .disabled(self.disabled)
            .style_with_state(move |state, base| {
                style_button(base, state.disabled, variant, size, &theme)
            })
            .children(self.children);

        if let Some(label) = self.aria_label {
            root = root.aria_label(label);
        }
        if let Some(handler) = self.on_click {
            root = root.on_click(move |event, window, cx| handler(event, window, cx));
        }

        root
    }
}

pub(super) fn style_button(
    base: Div,
    disabled: bool,
    variant: ButtonVariant,
    size: ButtonSize,
    theme: &UiTheme,
) -> Div {
    let colors = theme.colors;
    let metrics = size.metrics(f32::from(theme.radius.base));
    let focus_border = match variant {
        ButtonVariant::Destructive => colors.destructive.alpha(0.40),
        _ => colors.ring,
    };
    let focus_alpha = match (variant, theme.mode) {
        (ButtonVariant::Destructive, ThemeMode::Dark) => 0.40,
        (ButtonVariant::Destructive, ThemeMode::Light) => 0.20,
        _ => 0.50,
    };
    let focus_ring = match variant {
        ButtonVariant::Destructive => colors.destructive.alpha(focus_alpha),
        _ => colors.ring.alpha(focus_alpha),
    };
    let focus_background =
        match variant {
            ButtonVariant::Default => colors.primary,
            ButtonVariant::Outline => match theme.mode {
                ThemeMode::Light => colors.background,
                ThemeMode::Dark => colors.background.blend(colors.input.alpha(0.30)),
            },
            ButtonVariant::Secondary => colors.secondary,
            ButtonVariant::Ghost | ButtonVariant::Link => colors.background,
            ButtonVariant::Destructive => colors.background.blend(colors.destructive.alpha(
                if theme.mode == ThemeMode::Light {
                    0.10
                } else {
                    0.20
                },
            )),
        };

    let base = base
        .flex()
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .whitespace_nowrap()
        .h(px(metrics.height))
        .gap(px(metrics.gap))
        .rounded(px(metrics.radius))
        .border_1()
        .border_color(colors.background.alpha(0.0))
        .font_family(theme.fonts.body.clone())
        .font_weight(FontWeight::MEDIUM)
        .text_size(px(metrics.text_size))
        .focus_visible(move |style| {
            style
                .bg(focus_background)
                .border_color(focus_border)
                .shadow(vec![
                    BoxShadow::new(px(0.0), px(0.0), focus_ring.into()).spread_radius(px(3.0)),
                ])
        })
        .when(metrics.icon_only, |base| base.w(px(metrics.height)).p_0())
        .when(!metrics.icon_only, |base| {
            base.px(px(metrics.horizontal_padding))
        });

    let base = match variant {
        ButtonVariant::Default => base
            .bg(colors.primary)
            .text_color(colors.primary_foreground)
            .when(!disabled, |base| {
                base.hover(move |style| style.bg(colors.primary.alpha(0.80)))
            }),
        ButtonVariant::Outline => {
            let background = match theme.mode {
                ThemeMode::Light => colors.background,
                ThemeMode::Dark => colors.input.alpha(0.30),
            };
            let border = match theme.mode {
                ThemeMode::Light => colors.border,
                ThemeMode::Dark => colors.input,
            };
            let hover = match theme.mode {
                ThemeMode::Light => colors.muted,
                ThemeMode::Dark => colors.input.alpha(0.50),
            };
            base.bg(background)
                .text_color(colors.foreground)
                .border_color(border)
                .when(!disabled, |base| {
                    base.hover(move |style| style.bg(hover).text_color(colors.foreground))
                })
        }
        ButtonVariant::Secondary => {
            let hover = neutral(match theme.mode {
                ThemeMode::Light => 0.928_75,
                ThemeMode::Dark => 0.304_8,
            });
            base.bg(colors.secondary)
                .text_color(colors.secondary_foreground)
                .when(!disabled, |base| base.hover(move |style| style.bg(hover)))
        }
        ButtonVariant::Ghost => {
            let hover = match theme.mode {
                ThemeMode::Light => colors.muted,
                ThemeMode::Dark => colors.muted.alpha(0.50),
            };
            base.bg(colors.background.alpha(0.0))
                .text_color(colors.foreground)
                .when(!disabled, |base| {
                    base.hover(move |style| style.bg(hover).text_color(colors.foreground))
                })
        }
        ButtonVariant::Destructive => {
            let background_alpha = match theme.mode {
                ThemeMode::Light => 0.10,
                ThemeMode::Dark => 0.20,
            };
            let hover_alpha = match theme.mode {
                ThemeMode::Light => 0.20,
                ThemeMode::Dark => 0.30,
            };
            base.bg(colors.destructive.alpha(background_alpha))
                .text_color(colors.destructive)
                .when(!disabled, |base| {
                    base.hover(move |style| style.bg(colors.destructive.alpha(hover_alpha)))
                })
        }
        ButtonVariant::Link => base
            .bg(colors.background.alpha(0.0))
            .text_color(colors.primary)
            .when(!disabled, |base| base.hover(|style| style.underline())),
    };

    base.when(disabled, |base| base.opacity(0.50))
}

#[cfg(test)]
mod tests {
    use super::*;

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
            let metrics = size.metrics(10.0);
            assert_eq!(metrics.height, height);
            assert_eq!(metrics.horizontal_padding, padding);
            assert_eq!(metrics.gap, gap);
            assert_eq!(metrics.radius, radius);
        }
    }
}
