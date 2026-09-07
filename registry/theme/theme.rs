//! The app-owned shadcn Neutral theme for gpuicn.
//!
//! Source: shadcn/ui 4.19.0 at
//! `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`, Nova style.

use gpui::{
    App, BoxShadow, Corners, Div, Global, ParentElement as _, Pixels, Rgba, SharedString, Styled,
    black, px,
};

/// The active color mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThemeMode {
    /// The Neutral light palette.
    Light,
    /// The Neutral dark palette.
    Dark,
}

/// Semantic colors consumed by installed components.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiColors {
    /// App background.
    pub background: Rgba,
    /// Default foreground.
    pub foreground: Rgba,
    /// Card background.
    pub card: Rgba,
    /// Card foreground.
    pub card_foreground: Rgba,
    /// Popover background.
    pub popover: Rgba,
    /// Popover foreground.
    pub popover_foreground: Rgba,
    /// Primary control background.
    pub primary: Rgba,
    /// Primary control foreground.
    pub primary_foreground: Rgba,
    /// Secondary control background.
    pub secondary: Rgba,
    /// Secondary control foreground.
    pub secondary_foreground: Rgba,
    /// Muted background.
    pub muted: Rgba,
    /// Muted foreground.
    pub muted_foreground: Rgba,
    /// Accent background.
    pub accent: Rgba,
    /// Accent foreground.
    pub accent_foreground: Rgba,
    /// Destructive action color.
    pub destructive: Rgba,
    /// Default border.
    pub border: Rgba,
    /// Input border.
    pub input: Rgba,
    /// Focus ring.
    pub ring: Rgba,
    /// First chart color.
    pub chart_1: Rgba,
    /// Second chart color.
    pub chart_2: Rgba,
    /// Third chart color.
    pub chart_3: Rgba,
    /// Fourth chart color.
    pub chart_4: Rgba,
    /// Fifth chart color.
    pub chart_5: Rgba,
    /// Sidebar background.
    pub sidebar: Rgba,
    /// Sidebar foreground.
    pub sidebar_foreground: Rgba,
    /// Sidebar primary background.
    pub sidebar_primary: Rgba,
    /// Sidebar primary foreground.
    pub sidebar_primary_foreground: Rgba,
    /// Sidebar accent background.
    pub sidebar_accent: Rgba,
    /// Sidebar accent foreground.
    pub sidebar_accent_foreground: Rgba,
    /// Sidebar border.
    pub sidebar_border: Rgba,
    /// Sidebar focus ring.
    pub sidebar_ring: Rgba,
}

/// Theme font family names. Applications own font loading.
#[derive(Clone, Debug, PartialEq)]
pub struct UiFonts {
    /// Default body family.
    pub body: SharedString,
    /// Default heading family.
    pub heading: SharedString,
    /// Default monospace family.
    pub mono: SharedString,
}

/// Corner radii derived from shadcn's 10px base radius.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiRadius {
    /// Backward-compatible alias for [`Self::lg`].
    pub base: Pixels,
    /// Six pixels (`radius * 0.6`).
    pub sm: Pixels,
    /// Eight pixels (`radius * 0.8`).
    pub md: Pixels,
    /// Ten pixels (the configured base radius).
    pub lg: Pixels,
    /// Fourteen pixels (`radius * 1.4`).
    pub xl: Pixels,
    /// Eighteen pixels (`radius * 1.8`).
    pub two_xl: Pixels,
    /// Twenty-two pixels (`radius * 2.2`).
    pub three_xl: Pixels,
    /// Twenty-six pixels (`radius * 2.6`).
    pub four_xl: Pixels,
}

/// Shared shadcn elevation tokens.
#[derive(Clone, Debug, PartialEq)]
pub struct UiShadows {
    /// Tailwind `shadow-sm`.
    pub sm: Vec<BoxShadow>,
    /// Tailwind `shadow-md`.
    pub md: Vec<BoxShadow>,
    /// Tailwind `shadow-lg`.
    pub lg: Vec<BoxShadow>,
}

/// Base spacing unit from the pinned Nova style.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiSpacing {
    /// Four pixels.
    pub unit: Pixels,
}

/// The application-owned theme stored in GPUI global state.
#[derive(Clone, Debug, PartialEq)]
pub struct UiTheme {
    /// Active color mode.
    pub mode: ThemeMode,
    /// Active semantic colors.
    pub colors: UiColors,
    /// Font family names.
    pub fonts: UiFonts,
    /// Base corner radius.
    pub radius: UiRadius,
    /// Base spacing unit.
    pub spacing: UiSpacing,
    /// Shared elevation and focus-ring tokens.
    pub shadows: UiShadows,
}

impl Global for UiTheme {}

impl Default for UiTheme {
    fn default() -> Self {
        Self::neutral_light()
    }
}

impl UiTheme {
    /// Creates the pinned Neutral light theme.
    pub fn neutral_light() -> Self {
        Self::new(ThemeMode::Light, neutral_light_colors())
    }

    /// Creates the pinned Neutral dark theme.
    pub fn neutral_dark() -> Self {
        Self::new(ThemeMode::Dark, neutral_dark_colors())
    }

    /// Installs a theme into the application.
    pub fn set(cx: &mut App, theme: Self) {
        cx.set_global(theme);
    }

    /// Reads the installed theme.
    ///
    /// Panics when the application has not installed a theme.
    pub fn read(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    /// Replaces the installed theme with the pinned palette for `mode`.
    pub fn switch(cx: &mut App, mode: ThemeMode) {
        Self::set(
            cx,
            match mode {
                ThemeMode::Light => Self::neutral_light(),
                ThemeMode::Dark => Self::neutral_dark(),
            },
        );
    }

    /// Builds shadcn's three-pixel focus ring from the active semantic ring color.
    pub fn focus_ring(&self) -> Vec<BoxShadow> {
        vec![
            BoxShadow::new(px(0.), px(0.), self.colors.ring.opacity(0.50).into())
                .spread_radius(px(3.)),
        ]
    }

    /// Builds shadcn's invalid focus ring from the active destructive color.
    pub fn destructive_focus_ring(&self) -> Vec<BoxShadow> {
        let alpha = match self.mode {
            ThemeMode::Light => 0.20,
            ThemeMode::Dark => 0.40,
        };
        vec![
            BoxShadow::new(
                px(0.),
                px(0.),
                self.colors.destructive.opacity(alpha).into(),
            )
            .spread_radius(px(3.)),
        ]
    }

    fn new(mode: ThemeMode, colors: UiColors) -> Self {
        Self {
            mode,
            colors,
            fonts: UiFonts {
                body: "Geist".into(),
                heading: "Geist".into(),
                mono: "Geist Mono".into(),
            },
            radius: UiRadius {
                base: px(10.),
                sm: px(6.),
                md: px(8.),
                lg: px(10.),
                xl: px(14.),
                two_xl: px(18.),
                three_xl: px(22.),
                four_xl: px(26.),
            },
            spacing: UiSpacing { unit: px(4.) },
            shadows: UiShadows {
                sm: vec![
                    BoxShadow::new(px(0.), px(1.), black().alpha(0.10)).blur_radius(px(3.)),
                    BoxShadow::new(px(0.), px(1.), black().alpha(0.10))
                        .blur_radius(px(2.))
                        .spread_radius(px(-1.)),
                ],
                md: vec![
                    BoxShadow::new(px(0.), px(4.), black().alpha(0.10))
                        .blur_radius(px(6.))
                        .spread_radius(px(-1.)),
                    BoxShadow::new(px(0.), px(2.), black().alpha(0.10))
                        .blur_radius(px(4.))
                        .spread_radius(px(-2.)),
                ],
                lg: vec![
                    BoxShadow::new(px(0.), px(10.), black().alpha(0.10))
                        .blur_radius(px(15.))
                        .spread_radius(px(-3.)),
                    BoxShadow::new(px(0.), px(4.), black().alpha(0.10))
                        .blur_radius(px(6.))
                        .spread_radius(px(-4.)),
                ],
            },
        }
    }
}

fn neutral_light_colors() -> UiColors {
    UiColors {
        background: neutral(1.0),
        foreground: neutral(0.145),
        card: neutral(1.0),
        card_foreground: neutral(0.145),
        popover: neutral(1.0),
        popover_foreground: neutral(0.145),
        primary: neutral(0.205),
        primary_foreground: neutral(0.985),
        secondary: neutral(0.97),
        secondary_foreground: neutral(0.205),
        muted: neutral(0.97),
        muted_foreground: neutral(0.556),
        accent: neutral(0.97),
        accent_foreground: neutral(0.205),
        destructive: srgb(0xdc, 0x26, 0x26),
        border: neutral(0.922),
        input: neutral(0.922),
        ring: neutral(0.708),
        chart_1: neutral(0.87),
        chart_2: neutral(0.556),
        chart_3: neutral(0.439),
        chart_4: neutral(0.371),
        chart_5: neutral(0.269),
        sidebar: neutral(0.985),
        sidebar_foreground: neutral(0.145),
        sidebar_primary: neutral(0.205),
        sidebar_primary_foreground: neutral(0.985),
        sidebar_accent: neutral(0.97),
        sidebar_accent_foreground: neutral(0.205),
        sidebar_border: neutral(0.922),
        sidebar_ring: neutral(0.708),
    }
}

fn neutral_dark_colors() -> UiColors {
    UiColors {
        background: neutral(0.145),
        foreground: neutral(0.985),
        card: neutral(0.205),
        card_foreground: neutral(0.985),
        popover: neutral(0.205),
        popover_foreground: neutral(0.985),
        primary: neutral(0.922),
        primary_foreground: neutral(0.205),
        secondary: neutral(0.269),
        secondary_foreground: neutral(0.985),
        muted: neutral(0.269),
        muted_foreground: neutral(0.708),
        accent: neutral(0.269),
        accent_foreground: neutral(0.985),
        destructive: srgb(0xff, 0x64, 0x67),
        border: neutral_alpha(1.0, 0.10),
        input: neutral_alpha(1.0, 0.15),
        ring: neutral(0.556),
        chart_1: neutral(0.87),
        chart_2: neutral(0.556),
        chart_3: neutral(0.439),
        chart_4: neutral(0.371),
        chart_5: neutral(0.269),
        sidebar: neutral(0.205),
        sidebar_foreground: neutral(0.985),
        sidebar_primary: srgb(0x14, 0x47, 0xe6),
        sidebar_primary_foreground: neutral(0.985),
        sidebar_accent: neutral(0.269),
        sidebar_accent_foreground: neutral(0.985),
        sidebar_border: neutral_alpha(1.0, 0.10),
        sidebar_ring: neutral(0.556),
    }
}

pub(super) fn neutral(l: f64) -> Rgba {
    neutral_alpha(l, 1.0)
}

fn neutral_alpha(l: f64, alpha: f64) -> Rgba {
    let channel = srgb_channel(l * l * l) as f32;

    Rgba {
        r: channel,
        g: channel,
        b: channel,
        a: alpha.clamp(0.0, 1.0) as f32,
    }
}

const fn srgb(red: u8, green: u8, blue: u8) -> Rgba {
    Rgba {
        r: red as f32 / 255.0,
        g: green as f32 / 255.0,
        b: blue as f32 / 255.0,
        a: 1.0,
    }
}

fn srgb_channel(channel: f64) -> f64 {
    if channel >= 0.003_130_8 {
        (1.055 * channel.powf(1.0 / 2.4) - 0.055).clamp(0.0, 1.0)
    } else {
        (12.92 * channel).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_pinned_neutral_tokens() {
        assert_eq!(
            neutral(1.0),
            Rgba {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }
        );
        assert_eq!(
            neutral(0.0),
            Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }
        );

        assert_eq!(neutral_light_colors().destructive, srgb(0xdc, 0x26, 0x26));
        assert_eq!(neutral_dark_colors().destructive, srgb(0xff, 0x64, 0x67));
        assert_eq!(
            neutral_dark_colors().sidebar_primary,
            srgb(0x14, 0x47, 0xe6)
        );
    }

    #[test]
    fn preserves_pinned_alpha_tokens() {
        let dark = neutral_dark_colors();
        assert_eq!(dark.border.a, 0.10);
        assert_eq!(dark.input.a, 0.15);
        assert_eq!(dark.sidebar_border.a, 0.10);
        assert!((dark.input.opacity(0.30).a - 0.045).abs() < f32::EPSILON);
    }

    #[test]
    fn derives_the_shadcn_radius_and_shadow_scales() {
        let theme = UiTheme::neutral_light();
        assert_eq!(theme.radius.sm, px(6.));
        assert_eq!(theme.radius.md, px(8.));
        assert_eq!(theme.radius.lg, px(10.));
        assert_eq!(theme.radius.xl, px(14.));
        assert_eq!(theme.shadows.sm.len(), 2);
        assert_eq!(theme.shadows.md.len(), 2);
        assert_eq!(theme.shadows.lg.len(), 2);
        assert_eq!(theme.focus_ring()[0].spread_radius, px(3.));
    }
}

/// Centers the Base GPUI single-line editor independently of inherited typography.
/// The editor sizes its text and caret from the line height, so padding alone
/// cannot keep both centered across bordered and borderless controls.
pub(crate) fn input_text_layout(base: Div) -> Div {
    base.flex().items_center().line_height(px(20.))
}

/// Draws concentric focus corners; GPUI spread shadows retain the inner radius.
pub(crate) fn focus_outline(mut base: Div, color: Rgba, radii: Corners<Pixels>) -> Div {
    let borders = base.style().border_widths.clone();
    base.child(gpui::deferred(
        gpui::canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                let rem = window.rem_size();
                let borders = gpui::Edges {
                    left: borders.left.unwrap_or_default().to_pixels(rem),
                    top: borders.top.unwrap_or_default().to_pixels(rem),
                    right: borders.right.unwrap_or_default().to_pixels(rem),
                    bottom: borders.bottom.unwrap_or_default().to_pixels(rem),
                };
                window.paint_quad(focus_outline_quad(bounds, color, radii, borders));
            },
        )
        .absolute()
        .inset_0(),
    ))
}

fn focus_outline_quad(
    mut bounds: gpui::Bounds<Pixels>,
    color: Rgba,
    radii: Corners<Pixels>,
    borders: gpui::Edges<Pixels>,
) -> gpui::PaintQuad {
    bounds.origin.x -= borders.left + px(3.);
    bounds.origin.y -= borders.top + px(3.);
    bounds.size.width += borders.left + borders.right + px(6.);
    bounds.size.height += borders.top + borders.bottom + px(6.);
    gpui::outline(bounds, color, Default::default())
        .corner_radii(radii.map(|r| if *r > px(0.) { *r + px(3.) } else { *r }))
        .border_widths(px(3.))
}

#[cfg(test)]
mod focus_outline_tests {
    use super::*;
    #[test]
    fn focus_outlines_follow_circle_and_segment_borders() {
        let bounds = gpui::Bounds::new(gpui::point(px(1.), px(1.)), gpui::size(px(10.), px(10.)));
        let quad = focus_outline_quad(
            bounds,
            black().into(),
            Corners::all(px(6.)),
            gpui::Edges::all(px(1.)),
        );
        assert_eq!(quad.bounds.origin, gpui::point(px(-3.), px(-3.)));
        assert_eq!(quad.bounds.size, gpui::size(px(18.), px(18.)));
        assert_eq!(quad.corner_radii, Corners::all(px(9.)));
        let quad = focus_outline_quad(
            bounds,
            black().into(),
            Corners::all(px(0.)),
            gpui::Edges {
                left: px(0.),
                ..gpui::Edges::all(px(1.))
            },
        );
        assert_eq!(quad.bounds.origin.x, px(-2.));
        assert_eq!(quad.bounds.size.width, px(17.));
        assert_eq!(quad.corner_radii, Corners::all(px(0.)));
    }
}
