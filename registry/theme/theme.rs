//! The app-owned shadcn Neutral theme for imajha/ui.
//!
//! Source: shadcn/ui 4.19.0 at
//! `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`, Nova style.

use gpui::{App, Global, Pixels, Rgba, SharedString, px};

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

/// Base corner radius from the pinned shadcn theme.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiRadius {
    /// Ten pixels (`0.625rem` at the shadcn 16px root size).
    pub base: Pixels,
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

    fn new(mode: ThemeMode, colors: UiColors) -> Self {
        Self {
            mode,
            colors,
            fonts: UiFonts {
                body: "Geist".into(),
                heading: "Geist".into(),
                mono: "Geist Mono".into(),
            },
            radius: UiRadius { base: px(10.) },
            spacing: UiSpacing { unit: px(4.) },
        }
    }
}

fn neutral_light_colors() -> UiColors {
    UiColors {
        background: oklch(1.0, 0.0, 0.0),
        foreground: oklch(0.145, 0.0, 0.0),
        card: oklch(1.0, 0.0, 0.0),
        card_foreground: oklch(0.145, 0.0, 0.0),
        popover: oklch(1.0, 0.0, 0.0),
        popover_foreground: oklch(0.145, 0.0, 0.0),
        primary: oklch(0.205, 0.0, 0.0),
        primary_foreground: oklch(0.985, 0.0, 0.0),
        secondary: oklch(0.97, 0.0, 0.0),
        secondary_foreground: oklch(0.205, 0.0, 0.0),
        muted: oklch(0.97, 0.0, 0.0),
        muted_foreground: oklch(0.556, 0.0, 0.0),
        accent: oklch(0.97, 0.0, 0.0),
        accent_foreground: oklch(0.205, 0.0, 0.0),
        destructive: oklch(0.577, 0.245, 27.325),
        border: oklch(0.922, 0.0, 0.0),
        input: oklch(0.922, 0.0, 0.0),
        ring: oklch(0.708, 0.0, 0.0),
        chart_1: oklch(0.87, 0.0, 0.0),
        chart_2: oklch(0.556, 0.0, 0.0),
        chart_3: oklch(0.439, 0.0, 0.0),
        chart_4: oklch(0.371, 0.0, 0.0),
        chart_5: oklch(0.269, 0.0, 0.0),
        sidebar: oklch(0.985, 0.0, 0.0),
        sidebar_foreground: oklch(0.145, 0.0, 0.0),
        sidebar_primary: oklch(0.205, 0.0, 0.0),
        sidebar_primary_foreground: oklch(0.985, 0.0, 0.0),
        sidebar_accent: oklch(0.97, 0.0, 0.0),
        sidebar_accent_foreground: oklch(0.205, 0.0, 0.0),
        sidebar_border: oklch(0.922, 0.0, 0.0),
        sidebar_ring: oklch(0.708, 0.0, 0.0),
    }
}

fn neutral_dark_colors() -> UiColors {
    UiColors {
        background: oklch(0.145, 0.0, 0.0),
        foreground: oklch(0.985, 0.0, 0.0),
        card: oklch(0.205, 0.0, 0.0),
        card_foreground: oklch(0.985, 0.0, 0.0),
        popover: oklch(0.205, 0.0, 0.0),
        popover_foreground: oklch(0.985, 0.0, 0.0),
        primary: oklch(0.922, 0.0, 0.0),
        primary_foreground: oklch(0.205, 0.0, 0.0),
        secondary: oklch(0.269, 0.0, 0.0),
        secondary_foreground: oklch(0.985, 0.0, 0.0),
        muted: oklch(0.269, 0.0, 0.0),
        muted_foreground: oklch(0.708, 0.0, 0.0),
        accent: oklch(0.269, 0.0, 0.0),
        accent_foreground: oklch(0.985, 0.0, 0.0),
        destructive: oklch(0.704, 0.191, 22.216),
        border: oklch_alpha(1.0, 0.0, 0.0, 0.10),
        input: oklch_alpha(1.0, 0.0, 0.0, 0.15),
        ring: oklch(0.556, 0.0, 0.0),
        chart_1: oklch(0.87, 0.0, 0.0),
        chart_2: oklch(0.556, 0.0, 0.0),
        chart_3: oklch(0.439, 0.0, 0.0),
        chart_4: oklch(0.371, 0.0, 0.0),
        chart_5: oklch(0.269, 0.0, 0.0),
        sidebar: oklch(0.205, 0.0, 0.0),
        sidebar_foreground: oklch(0.985, 0.0, 0.0),
        sidebar_primary: oklch(0.488, 0.243, 264.376),
        sidebar_primary_foreground: oklch(0.985, 0.0, 0.0),
        sidebar_accent: oklch(0.269, 0.0, 0.0),
        sidebar_accent_foreground: oklch(0.985, 0.0, 0.0),
        sidebar_border: oklch_alpha(1.0, 0.0, 0.0, 0.10),
        sidebar_ring: oklch(0.556, 0.0, 0.0),
    }
}

fn oklch(l: f64, c: f64, h: f64) -> Rgba {
    oklch_alpha(l, c, h, 1.0)
}

fn oklch_alpha(l: f64, c: f64, h: f64, alpha: f64) -> Rgba {
    let hue = h.to_radians();
    let a = c * hue.cos();
    let b = c * hue.sin();

    let l_root = l + 0.396_337_777_4 * a + 0.215_803_757_3 * b;
    let m_root = l - 0.105_561_345_8 * a - 0.063_854_172_8 * b;
    let s_root = l - 0.089_484_177_5 * a - 1.291_485_548 * b;
    let l = l_root * l_root * l_root;
    let m = m_root * m_root * m_root;
    let s = s_root * s_root * s_root;

    let linear_r = 4.076_741_662_1 * l - 3.307_711_591_3 * m + 0.230_969_929_2 * s;
    let linear_g = -1.268_438_004_6 * l + 2.609_757_401_1 * m - 0.341_319_396_5 * s;
    let linear_b = -0.004_196_086_3 * l - 0.703_418_614_7 * m + 1.707_614_701 * s;

    Rgba {
        r: srgb(linear_r) as f32,
        g: srgb(linear_g) as f32,
        b: srgb(linear_b) as f32,
        a: alpha.clamp(0.0, 1.0) as f32,
    }
}

fn srgb(channel: f64) -> f64 {
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
            oklch(1.0, 0.0, 0.0),
            Rgba {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }
        );
        assert_eq!(
            oklch(0.0, 0.0, 0.0),
            Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }
        );

        let destructive = oklch(0.577, 0.245, 27.325);
        assert!((destructive.r - 0.906_46).abs() < 0.000_01);
        assert_eq!(destructive.g, 0.0);
        assert!((destructive.b - 0.042_21).abs() < 0.000_01);
    }

    #[test]
    fn preserves_pinned_alpha_tokens() {
        let dark = neutral_dark_colors();
        assert_eq!(dark.border.a, 0.10);
        assert_eq!(dark.input.a, 0.15);
        assert_eq!(dark.sidebar_border.a, 0.10);
    }
}
