//! The app-owned shadcn Neutral theme for gpuicn.
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
    }
}
