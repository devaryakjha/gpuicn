//! The app-owned shadcn Neutral theme for gpuicn.
//!
//! Source: shadcn/ui 4.19.0 at
//! `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`, Nova style.

use std::time::Duration;

use gpui_kit::{
    App, BoxShadow, ElementId, Global, Pixels, Rgba, SharedString, Styled, Window, black, px,
};

gpui_kit::actions!(
    gpuicn,
    [
        /// Move to the next keyboard tab stop.
        FocusNext,
        /// Move to the previous keyboard tab stop.
        FocusPrevious
    ]
);

struct Initialized;
impl Global for Initialized {}

/// Installs component actions, keyboard traversal and the default theme once.
/// Call this from application startup for both copied source and crate usage.
pub fn init(cx: &mut App) {
    if cx.has_global::<Initialized>() {
        return;
    }
    cx.set_global(Initialized);
    if !cx.has_global::<UiTheme>() {
        UiTheme::set(cx, UiTheme::neutral_light());
    }
    cx.bind_keys([
        gpui_kit::KeyBinding::new("tab", FocusNext, None),
        gpui_kit::KeyBinding::new("shift-tab", FocusPrevious, None),
    ]);
    cx.on_action(|_: &FocusNext, cx| advance_focus(false, cx));
    cx.on_action(|_: &FocusPrevious, cx| advance_focus(true, cx));
    // Later scoped bindings take precedence over window-wide defaults.
    gpui_kit::base::init(cx);
    #[cfg(target_family = "wasm")]
    {
        // Browser previews also accept macOS editing shortcuts.
        use gpui_kit::KeyBinding;
        use gpui_kit::base::input::{
            Copy, Cut, MoveEnd, MoveHome, MoveToNextWord, MoveToPreviousWord, Paste, Redo,
            SelectAll, SelectToNextWordEnd, SelectToPreviousWordStart, Undo,
        };
        cx.bind_keys([
            KeyBinding::new("cmd-a", SelectAll, Some("Input")),
            KeyBinding::new("cmd-c", Copy, Some("Input")),
            KeyBinding::new("cmd-v", Paste, Some("Input")),
            KeyBinding::new("cmd-x", Cut, Some("Input")),
            KeyBinding::new("cmd-left", MoveHome, Some("Input")),
            KeyBinding::new("cmd-right", MoveEnd, Some("Input")),
            KeyBinding::new("cmd-z", Undo, Some("Input")),
            KeyBinding::new("cmd-shift-z", Redo, Some("Input")),
            KeyBinding::new("alt-left", MoveToPreviousWord, Some("Input")),
            KeyBinding::new("alt-right", MoveToNextWord, Some("Input")),
            KeyBinding::new("alt-shift-left", SelectToPreviousWordStart, Some("Input")),
            KeyBinding::new("alt-shift-right", SelectToNextWordEnd, Some("Input")),
        ]);
    }
}

fn advance_focus(reverse: bool, cx: &mut App) {
    let Some(handle) = cx.active_window() else {
        return;
    };
    // Action dispatch still holds the window borrow until this callback returns.
    cx.defer(move |cx| {
        let _ = handle.update(cx, |_, window, cx| {
            if reverse {
                window.focus_prev(cx);
            } else {
                window.focus_next(cx);
            }
        });
    });
}

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
    /// Sidebar focus border color.
    pub sidebar_ring: Rgba,
    /// Scrim behind modal surfaces.
    pub overlay: Rgba,
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

impl UiRadius {
    /// Derives the complete Nova radius scale from one nonnegative base radius.
    pub fn new(base: Pixels) -> Self {
        assert!(f32::from(base).is_finite() && base >= px(0.));
        Self {
            base,
            sm: base * 0.6,
            md: base * 0.8,
            lg: base,
            xl: base * 1.4,
            two_xl: base * 1.8,
            three_xl: base * 2.2,
            four_xl: base * 2.6,
        }
    }
}

/// Shared UI transition durations. Honor these alongside `App::reduce_motion()`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiMotion {
    /// Thumb and small-state transitions (150ms by default).
    pub fast: Duration,
    /// Panel/layout transitions (200ms by default).
    pub normal: Duration,
    /// Application preference; true removes movement without removing state feedback.
    pub reduced: bool,
    /// Shared easing for reversible state transitions.
    pub easing: UiEasing,
}
impl Default for UiMotion {
    fn default() -> Self {
        Self {
            fast: Duration::from_millis(150),
            normal: Duration::from_millis(200),
            reduced: false,
            easing: UiEasing::EaseInOut,
        }
    }
}

/// Curves supported by the shared native transition helper.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum UiEasing {
    /// Constant-speed motion, as used by shadcn sidebar layout transitions.
    Linear,
    /// Fast response followed by a smooth stop.
    EaseOut,
    /// Smooth acceleration and deceleration for changes within a layout.
    #[default]
    EaseInOut,
}
impl UiEasing {
    fn sample(self, progress: f32) -> f32 {
        match self {
            Self::Linear => progress,
            Self::EaseOut => gpui_kit::ease_out_quint()(progress),
            Self::EaseInOut => gpui_kit::ease_in_out(progress),
        }
    }
}

/// Returns a smoothly retargetable value for one stable, caller-named property.
/// Starts at the target on mount; rapid reversals continue from the current value.
/// Honors theme and GPUI reduced-motion settings and schedules no idle frames.
/// Use for state changes, never for pointer-drag coordinates or keyboard navigation.
/// Native layout transitions perform layout work; retain expensive child entities.
pub fn transition_value(
    id: impl Into<ElementId>,
    target: f32,
    duration: Duration,
    window: &mut Window,
    cx: &mut App,
) -> f32 {
    assert!(target.is_finite(), "transition targets must be finite");
    let motion = UiTheme::read(cx).motion;
    let duration = if motion.reduced || cx.reduce_motion() {
        Duration::ZERO
    } else {
        duration
    };
    gpui_kit::base::motion::transition(
        id.into(),
        target,
        gpui_kit::base::motion::Transition::new(duration)
            .ease(move |progress| motion.easing.sample(progress)),
        window,
        cx,
    )
}

/// Shares the theme's disclosure and notification timing with Kit's presence lifecycle.
pub(crate) fn presence(
    id: impl Into<ElementId>,
    present: bool,
    window: &mut Window,
    cx: &mut App,
) -> gpui_kit::base::motion::PresenceSample {
    let motion = UiTheme::read(cx).motion;
    let duration = if motion.reduced {
        Duration::ZERO
    } else {
        motion.fast
    };
    gpui_kit::base::motion::Presence::new(id.into(), present)
        .transition(
            gpui_kit::base::motion::Transition::new(duration)
                .ease(move |progress| motion.easing.sample(progress)),
        )
        .sample(window, cx)
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
    /// Shared elevation tokens.
    pub shadows: UiShadows,
    /// Shared motion durations and reduced-motion preference.
    pub motion: UiMotion,
    /// Typography scale, independent of control density (1 by default).
    pub text_scale: f32,
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
        assert!(f32::from(theme.spacing.unit).is_finite() && theme.spacing.unit > px(0.));
        assert!(theme.text_scale.is_finite() && theme.text_scale > 0.);
        cx.set_global(theme);
        cx.refresh_windows();
    }

    /// Reads the installed theme.
    ///
    /// Panics when the application has not installed a theme.
    pub fn read(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    /// Selects the pinned palette, preserving non-color tokens or initializing defaults.
    pub fn switch(cx: &mut App, mode: ThemeMode) {
        let mut theme = cx
            .try_global::<Self>()
            .cloned()
            .unwrap_or_else(Self::neutral_light);
        theme.mode = mode;
        theme.colors = match mode {
            ThemeMode::Light => neutral_light_colors(),
            ThemeMode::Dark => neutral_dark_colors(),
        };
        Self::set(cx, theme);
    }

    /// Returns a multiple of the configured spacing unit (2 means 8px by default).
    pub fn space(&self, units: f32) -> Pixels {
        self.spacing.unit * units
    }

    /// Scales typography independently of spacing; sizes use Nova's default pixels.
    pub fn text(&self, size: f32) -> Pixels {
        px(size * self.text_scale)
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
            radius: UiRadius::new(px(10.)),
            motion: UiMotion::default(),
            text_scale: 1.,
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
        overlay: black().alpha(0.10).into(),
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
        overlay: black().alpha(0.10).into(),
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
    }
}

// Refine the actual control after default styling; no extra layout or focus node.
pub(crate) fn apply_style<T: Styled>(mut element: T, style: &gpui_kit::StyleRefinement) -> T {
    use gpui_kit::Refineable as _;
    element.style().refine(style);
    element
}

#[derive(gpui_kit::IntoElement)]
pub(crate) struct DisclosureIcon {
    icon: gpui_kit::Svg,
    open: bool,
}
pub(crate) fn disclosure_icon(icon: gpui_kit::Svg, open: bool) -> DisclosureIcon {
    DisclosureIcon { icon, open }
}
impl gpui_kit::RenderOnce for DisclosureIcon {
    fn render(self, window: &mut Window, cx: &mut App) -> impl gpui_kit::IntoElement {
        let duration = UiTheme::read(cx).motion.fast;
        let value = transition_value(
            "disclosure-rotation",
            if self.open { 1. } else { 0. },
            duration,
            window,
            cx,
        );
        self.icon
            .with_transformation(gpui_kit::Transformation::rotate(gpui_kit::radians(
                value * std::f32::consts::PI,
            )))
    }
}

#[cfg(test)]
mod motion_tests {
    use super::*;
    use gpui_kit::{
        AppContext as _, Context, IntoElement, Render, TestAppContext, VisualTestContext, div,
    };

    struct View {
        target: f32,
        sampled: f32,
    }
    impl Render for View {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            self.sampled = transition_value(
                "test-motion",
                self.target,
                Duration::from_millis(200),
                window,
                cx,
            );
            div()
        }
    }

    #[gpui_kit::test]
    fn theme_reduced_motion_snaps_an_active_upstream_transition(cx: &mut TestAppContext) {
        cx.update(init);
        let window = cx.add_window(|_, _| View {
            target: 0.,
            sampled: 0.,
        });
        let mut visual = VisualTestContext::from_window(window.into(), cx);
        visual.update(|window, cx| window.draw(cx).clear(cx));
        window
            .update(cx, |view, _, cx| {
                view.target = 1.;
                cx.notify();
            })
            .unwrap();
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(
            cx.read_window(&window, |view, cx| view.read(cx).sampled)
                .unwrap(),
            0.
        );
        cx.update(|cx| {
            let mut theme = UiTheme::read(cx).clone();
            theme.motion.reduced = true;
            UiTheme::set(cx, theme);
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(
            cx.read_window(&window, |view, cx| view.read(cx).sampled)
                .unwrap(),
            1.
        );
    }
}

/// Shared focus traversal for the tab, radio and toggle collections.
pub(crate) struct RovingFocus {
    pub handles: Vec<Option<gpui_kit::FocusHandle>>,
    active: gpui_kit::Entity<Option<gpui_kit::FocusHandle>>,
}
impl RovingFocus {
    pub fn new(
        id: gpui_kit::ElementId,
        items: &[(gpui_kit::ElementId, bool)],
        selected: Option<usize>,
        window: &mut gpui_kit::Window,
        cx: &mut App,
    ) -> Self {
        let active =
            window.use_keyed_state((id, "active"), cx, |_, _| None::<gpui_kit::FocusHandle>);
        let mut handles: Vec<_> = items
            .iter()
            .map(|(id, disabled)| {
                let handle = window
                    .use_keyed_state((id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
                    .read(cx)
                    .clone();
                handle.clone().tab_stop(false);
                (!disabled).then_some(handle)
            })
            .collect();
        let index = handles
            .iter()
            .position(|h| h.as_ref().is_some_and(|h| h.is_focused(window)))
            .or(selected.filter(|&i| handles.get(i).is_some_and(Option::is_some)))
            .or_else(|| {
                handles
                    .iter()
                    .position(|h| h.is_some() && h.as_ref() == active.read(cx).as_ref())
            })
            .or_else(|| handles.iter().position(Option::is_some));
        if let Some(index) = index {
            let handle = handles[index].as_ref().unwrap().clone().tab_stop(true);
            handles[index] = Some(handle.clone());
            active.update(cx, |value, _| *value = Some(handle));
        }
        Self { handles, active }
    }
    pub fn key(
        &self,
        event: &gpui_kit::KeyDownEvent,
        axis: Option<gpui_kit::Axis>,
        window: &mut gpui_kit::Window,
        cx: &mut App,
    ) -> Option<usize> {
        if event.keystroke.modifiers.modified() {
            return None;
        }
        let enabled: Vec<_> = self
            .handles
            .iter()
            .enumerate()
            .filter_map(|(i, h)| h.as_ref().map(|h| (i, h)))
            .collect();
        let current = enabled.iter().position(|(_, h)| h.is_focused(window))?;
        let next = match event.keystroke.key.as_str() {
            "home" => 0,
            "end" => enabled.len() - 1,
            "left" if axis != Some(gpui_kit::Axis::Vertical) => {
                (current + enabled.len() - 1) % enabled.len()
            }
            "right" if axis != Some(gpui_kit::Axis::Vertical) => (current + 1) % enabled.len(),
            "up" if axis != Some(gpui_kit::Axis::Horizontal) => {
                (current + enabled.len() - 1) % enabled.len()
            }
            "down" if axis != Some(gpui_kit::Axis::Horizontal) => (current + 1) % enabled.len(),
            _ => return None,
        };
        enabled[current].1.clone().tab_stop(false);
        let handle = enabled[next].1.clone().tab_stop(true);
        handle.focus(window, cx);
        self.active.update(cx, |value, _| *value = Some(handle));
        window.refresh();
        cx.stop_propagation();
        Some(enabled[next].0)
    }
}
