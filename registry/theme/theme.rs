//! The app-owned shadcn Neutral theme for gpuicn.
//!
//! Source: shadcn/ui 4.19.0 at
//! `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`, Nova style.

use std::{sync::Arc, time::Duration};
use web_time::Instant;

use gpui::{
    App, BoxShadow, Corners, Div, ElementId, Global, ParentElement as _, Pixels, Rgba,
    SharedString, Styled, Window, black, px,
};

gpui::actions!(
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
        gpui::KeyBinding::new("tab", FocusNext, None),
        gpui::KeyBinding::new("shift-tab", FocusPrevious, None),
    ]);
    cx.on_action(|_: &FocusNext, cx| advance_focus(false, cx));
    cx.on_action(|_: &FocusPrevious, cx| advance_focus(true, cx));
    // Later scoped bindings take precedence over window-wide defaults.
    base_gpui::init(cx);
    #[cfg(target_family = "wasm")]
    {
        // WASM has no macOS target_os, so Base GPUI only registers Control
        // shortcuts. Accept Command as well for previews on macOS browsers.
        use base_gpui::primitives::input::{
            INPUT_KEY_CONTEXT, InputCopy, InputCut, InputEnd, InputHome, InputPaste, InputSelectAll,
        };
        use gpui::KeyBinding;
        cx.bind_keys([
            KeyBinding::new("cmd-a", InputSelectAll, Some(INPUT_KEY_CONTEXT)),
            KeyBinding::new("cmd-c", InputCopy, Some(INPUT_KEY_CONTEXT)),
            KeyBinding::new("cmd-v", InputPaste, Some(INPUT_KEY_CONTEXT)),
            KeyBinding::new("cmd-x", InputCut, Some(INPUT_KEY_CONTEXT)),
            KeyBinding::new("cmd-left", InputHome, Some(INPUT_KEY_CONTEXT)),
            KeyBinding::new("cmd-right", InputEnd, Some(INPUT_KEY_CONTEXT)),
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
    /// Sidebar focus ring.
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
            Self::EaseOut => gpui::ease_out_quint()(progress),
            Self::EaseInOut => gpui::ease_in_out(progress),
        }
    }
}

struct Transition {
    from: f32,
    target: f32,
    started: Instant,
}
impl Transition {
    fn value(&self, now: Instant, duration: Duration, easing: UiEasing) -> f32 {
        if duration.is_zero() {
            return self.target;
        }
        let progress =
            (now.duration_since(self.started).as_secs_f32() / duration.as_secs_f32()).min(1.);
        self.from + (self.target - self.from) * easing.sample(progress)
    }
    fn retarget(&mut self, target: f32, now: Instant, duration: Duration, easing: UiEasing) {
        if target != self.target {
            self.from = self.value(now, duration, easing);
            self.target = target;
            self.started = now;
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
    let now = Instant::now();
    let key = ElementId::NamedChild(Arc::new(id.into()), "transition".into());
    let state = window.use_keyed_state(key, cx, |_, _| Transition {
        from: target,
        target,
        started: now,
    });
    let (value, active) = state.update(cx, |state, _| {
        state.retarget(target, now, duration, motion.easing);
        if duration.is_zero() {
            state.from = target;
        }
        (
            state.value(now, duration, motion.easing),
            state.from != target && now.duration_since(state.started) < duration,
        )
    });
    if active {
        window.request_animation_frame();
    }
    value
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
    fn editor_semantics_survive_upstream_id_assignment() {
        use gpui::{Element as _, InteractiveElement as _, StatefulInteractiveElement as _};
        let input = InputSemantics(gpui::div())
            .role(gpui::Role::SpinButton)
            .aria_label("Quantity")
            .aria_value("4")
            .aria_numeric_value(4.)
            .0
            .id("upstream-editor");
        assert_eq!(input.a11y_role(), Some(gpui::Role::SpinButton));
        let mut node = gpui::accesskit::Node::new(gpui::Role::SpinButton);
        input.write_a11y_info(&mut node);
        assert_eq!(node.label(), Some("Quantity"));
        assert_eq!(node.value(), Some("4"));
        assert_eq!(node.numeric_value(), Some(4.));
    }

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

/// Adds semantics to an upstream editor's existing Div before it receives its ID.
/// Keeping the same Div preserves the editor's focus handle and event handlers.
pub(crate) struct InputSemantics(pub Div);
impl gpui::InteractiveElement for InputSemantics {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.0.interactivity()
    }
}
impl gpui::StatefulInteractiveElement for InputSemantics {}
impl gpui::IntoElement for InputSemantics {
    type Element = Div;
    fn into_element(self) -> Div {
        self.0
    }
}

/// Centers the Base GPUI single-line editor independently of inherited typography.
/// The editor sizes its text and caret from the line height, so padding alone
/// cannot keep both centered across bordered and borderless controls.
pub(crate) fn input_text_layout(base: Div, text_scale: f32) -> Div {
    base.flex().items_center().line_height(px(20.) * text_scale)
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

// Refine the actual control after default styling; no extra layout or focus node.
pub(crate) fn apply_style<T: Styled>(mut element: T, style: &gpui::StyleRefinement) -> T {
    use gpui::Refineable as _;
    element.style().refine(style);
    element
}

#[derive(gpui::IntoElement)]
pub(crate) struct DisclosureIcon {
    icon: gpui::Svg,
    open: bool,
}
pub(crate) fn disclosure_icon(icon: gpui::Svg, open: bool) -> DisclosureIcon {
    DisclosureIcon { icon, open }
}
impl gpui::RenderOnce for DisclosureIcon {
    fn render(self, window: &mut Window, cx: &mut App) -> impl gpui::IntoElement {
        let duration = UiTheme::read(cx).motion.fast;
        let value = transition_value(
            "disclosure-rotation",
            if self.open { 1. } else { 0. },
            duration,
            window,
            cx,
        );
        self.icon
            .with_transformation(gpui::Transformation::rotate(gpui::radians(
                value * std::f32::consts::PI,
            )))
    }
}

#[cfg(test)]
mod motion_tests {
    use super::*;
    #[test]
    fn transitions_reverse_continuously_and_zero_duration_is_immediate() {
        let start = Instant::now();
        let duration = Duration::from_millis(200);
        let mut transition = Transition {
            from: 0.,
            target: 1.,
            started: start,
        };
        let halfway = start + Duration::from_millis(100);
        let current = transition.value(halfway, duration, UiEasing::EaseInOut);
        assert_eq!(current, 0.5);
        transition.retarget(0., halfway, duration, UiEasing::EaseInOut);
        assert_eq!(
            transition.value(halfway, duration, UiEasing::EaseInOut),
            current
        );
        assert_eq!(
            transition.value(halfway + duration, duration, UiEasing::EaseInOut),
            0.
        );
        transition.retarget(1., halfway + duration, Duration::ZERO, UiEasing::Linear);
        assert_eq!(
            transition.value(halfway + duration, Duration::ZERO, UiEasing::Linear),
            1.
        );
    }
}
