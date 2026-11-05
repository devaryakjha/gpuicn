//! The app-owned shadcn Neutral theme for gpuicn.
//!
//! Source: shadcn/ui 4.19.0 at
//! `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`, Nova style.

use std::{sync::Arc, time::Duration};

use gpui_kit::{
    App, BoxShadow, ElementId, Global, Pixels, Rgba, SharedString, Styled, Subscription, Window,
    WindowAppearance, black, px,
};

#[path = "locale.rs"]
pub mod locale;
#[path = "primitives.rs"]
pub mod primitives;

pub use locale::{Direction, UiText};

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

/// The key context of a one-time-code field, which binds the paste shortcut.
#[doc(hidden)]
pub const OTP_CONTEXT: &str = "GpuicnOtp";

/// Installs component actions, keyboard traversal and the default theme once.
/// Call this from application startup for both copied source and crate usage.
pub fn init(cx: &mut App) {
    if cx.has_global::<Initialized>() {
        return;
    }
    seed_component_fonts(cx);
    gpui_kit::init(cx);
    cx.set_global(Initialized);
    if !cx.has_global::<UiTheme>() {
        UiTheme::set(cx, UiTheme::neutral_light());
    } else {
        let theme = UiTheme::read(cx).clone();
        sync_component_theme(cx, theme);
    }
    cx.bind_keys([
        gpui_kit::KeyBinding::new("tab", FocusNext, None),
        gpui_kit::KeyBinding::new("shift-tab", FocusPrevious, None),
    ]);
    locale::bind_keys(cx);
    {
        use gpui_kit::{KeyBinding, base::input::Paste};
        let context = Some(OTP_CONTEXT);
        #[cfg(any(target_os = "macos", target_family = "wasm"))]
        cx.bind_keys([KeyBinding::new("cmd-v", Paste, context)]);
        #[cfg(not(target_os = "macos"))]
        cx.bind_keys([KeyBinding::new("ctrl-v", Paste, context)]);
    }
    cx.on_action(|_: &FocusNext, cx| advance_focus(false, cx));
    cx.on_action(|_: &FocusPrevious, cx| advance_focus(true, cx));
    // Scoped component bindings take precedence over these window-wide defaults.
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

fn seed_component_fonts(cx: &mut App) {
    use gpui_kit::component::Theme;

    if cx.has_global::<Theme>() {
        return;
    }
    let fonts = if cx.has_global::<UiTheme>() {
        UiTheme::read(cx).fonts.clone()
    } else {
        UiTheme::neutral_light().fonts
    };
    let mut component = Theme::default();
    component.font_family = fonts.body;
    component.mono_font_family = fonts.mono;
    cx.set_global(component);
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

/// Which of an application's two palettes is shown.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Appearance {
    /// Always the light palette.
    Light,
    /// Always the dark palette.
    Dark,
    /// The palette matching the operating system's window appearance.
    #[default]
    System,
}

impl Appearance {
    /// The mode this appearance shows under the given system appearance.
    pub fn resolve(self, system: WindowAppearance) -> ThemeMode {
        match self {
            Appearance::Light => ThemeMode::Light,
            Appearance::Dark => ThemeMode::Dark,
            Appearance::System => match system {
                WindowAppearance::Dark | WindowAppearance::VibrantDark => ThemeMode::Dark,
                WindowAppearance::Light | WindowAppearance::VibrantLight => ThemeMode::Light,
            },
        }
    }
}

/// The application's light and dark themes and which one it shows.
#[derive(Clone)]
struct ThemePair {
    light: UiTheme,
    dark: UiTheme,
    appearance: Appearance,
}
impl Global for ThemePair {}

/// Swaps palettes when the operating system's appearance changes while the
/// application follows it. Call it for each window the application opens and
/// keep the subscription; `app::run` does this for the first window.
pub fn follow_system(window: &mut Window, cx: &mut App) -> Subscription {
    UiTheme::apply_appearance(cx);
    window.observe_window_appearance(|_, cx| UiTheme::apply_appearance(cx))
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
    /// Text on a solid `destructive` surface.
    pub destructive_foreground: Rgba,
    /// Positive status, such as a completed upload or an online presence dot.
    pub success: Rgba,
    /// Text on a `success` fill.
    pub success_foreground: Rgba,
    /// Cautionary status, such as a permalink highlight or a pending state.
    pub warning: Rgba,
    /// Text on a `warning` fill.
    pub warning_foreground: Rgba,
    /// Informational status.
    pub info: Rgba,
    /// Text on an `info` fill.
    pub info_foreground: Rgba,
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
    /// Markdown link text. `None` follows `primary`; read it with `link()`.
    pub link: Option<Rgba>,
    /// Fill behind a mention of the reader, and the tint of a mention row.
    /// `None` follows `primary`; read it with `mention_highlight()`.
    pub mention_highlight: Option<Rgba>,
    /// Text on `mention_highlight`. `None` follows `primary_foreground`.
    pub mention_highlight_foreground: Option<Rgba>,
    /// The rule and label marking where new messages start. `None` follows
    /// `destructive`; read it with `new_messages()`.
    pub new_messages: Option<Rgba>,
    /// Quiet sidebar text: read rows, icons and section labels. `None`
    /// follows `sidebar_foreground` at 70%.
    pub sidebar_muted_foreground: Option<Rgba>,
    /// Sidebar mention badges. `None` follows `sidebar_primary`.
    pub sidebar_badge: Option<Rgba>,
    /// Text on `sidebar_badge`. `None` follows `sidebar_primary_foreground`.
    pub sidebar_badge_foreground: Option<Rgba>,
}

impl UiColors {
    /// Markdown link text: `link`, or `primary`.
    pub fn link(&self) -> Rgba {
        self.link.unwrap_or(self.primary)
    }
    /// Mention fill: `mention_highlight`, or `primary`.
    pub fn mention_highlight(&self) -> Rgba {
        self.mention_highlight.unwrap_or(self.primary)
    }
    /// Text on the mention fill: `mention_highlight_foreground`, or `primary_foreground`.
    pub fn mention_highlight_foreground(&self) -> Rgba {
        self.mention_highlight_foreground
            .unwrap_or(self.primary_foreground)
    }
    /// The new-messages rule: `new_messages`, or `destructive`.
    pub fn new_messages(&self) -> Rgba {
        self.new_messages.unwrap_or(self.destructive)
    }
    /// Quiet sidebar text: `sidebar_muted_foreground`, or `sidebar_foreground` at 70%.
    pub fn sidebar_muted_foreground(&self) -> Rgba {
        self.sidebar_muted_foreground
            .unwrap_or(self.sidebar_foreground.opacity(0.7))
    }
    /// Sidebar mention badge fill: `sidebar_badge`, or `sidebar_primary`.
    pub fn sidebar_badge(&self) -> Rgba {
        self.sidebar_badge.unwrap_or(self.sidebar_primary)
    }
    /// Text on a sidebar mention badge: `sidebar_badge_foreground`, or
    /// `sidebar_primary_foreground`.
    pub fn sidebar_badge_foreground(&self) -> Rgba {
        self.sidebar_badge_foreground
            .unwrap_or(self.sidebar_primary_foreground)
    }
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
    /// Quick start and a long settle (`cubic-bezier(0.22, 1, 0.36, 1)`), for
    /// surfaces that slide in from an edge, such as drawers and sheets.
    Emphasized,
}
impl UiEasing {
    fn sample(self, progress: f32) -> f32 {
        match self {
            Self::Linear => progress,
            Self::EaseOut => gpui_kit::ease_out_quint()(progress),
            Self::EaseInOut => gpui_kit::ease_in_out(progress),
            Self::Emphasized => gpui_kit::base::motion::Easing::CubicBezier {
                x1: 0.22,
                y1: 1.,
                x2: 0.36,
                y2: 1.,
            }
            .sample(progress),
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
    let easing = UiTheme::read(cx).motion.easing;
    transition_with(id, target, duration, easing, window, cx)
}

/// [`transition_value`] with an explicit curve, for the few surfaces whose motion
/// differs from the theme default (drawers slide with [`UiEasing::Emphasized`]).
pub fn transition_with(
    id: impl Into<ElementId>,
    target: f32,
    duration: Duration,
    easing: UiEasing,
    window: &mut Window,
    cx: &mut App,
) -> f32 {
    assert!(target.is_finite(), "transition targets must be finite");
    gpui_kit::base::motion::transition(
        id.into(),
        target,
        motion_policy(duration, easing, cx),
        window,
        cx,
    )
}

/// Shares the theme's disclosure and notification timing with Kit's presence lifecycle.
/// Returns a sample whose `progress` runs 0→1 while entering and 1→0 while leaving, so
/// a surface fades or scales with it; `should_render()` keeps the element mounted
/// until the exit transition has finished.
#[doc(hidden)]
pub fn presence(
    id: impl Into<ElementId>,
    present: bool,
    window: &mut Window,
    cx: &mut App,
) -> gpui_kit::base::motion::PresenceSample {
    let motion = UiTheme::read(cx).motion;
    presence_with(id, present, motion.fast, motion.easing, window, cx)
}

/// [`presence`] with explicit timing and curve, for panels that take the theme's
/// `normal` duration or a different easing.
#[doc(hidden)]
pub fn presence_with(
    id: impl Into<ElementId>,
    present: bool,
    duration: Duration,
    easing: UiEasing,
    window: &mut Window,
    cx: &mut App,
) -> gpui_kit::base::motion::PresenceSample {
    gpui_kit::base::motion::Presence::new(id.into(), present)
        .transition(motion_policy(duration, easing, cx))
        .sample(window, cx)
}

/// The one place motion honors the theme and platform reduced-motion settings.
fn motion_policy(
    duration: Duration,
    easing: UiEasing,
    cx: &App,
) -> gpui_kit::base::motion::Transition {
    let duration = if UiTheme::read(cx).motion.reduced || cx.reduce_motion() {
        Duration::ZERO
    } else {
        duration
    };
    gpui_kit::base::motion::Transition::new(duration).ease(move |progress| easing.sample(progress))
}

/// Shared shadcn elevation tokens. Each level is shared, so cloning a theme
/// every render costs no allocation; `to_vec()` the level you apply.
#[derive(Clone, Debug, PartialEq)]
pub struct UiShadows {
    /// Tailwind `shadow-sm`.
    pub sm: Arc<[BoxShadow]>,
    /// Tailwind `shadow-md`.
    pub md: Arc<[BoxShadow]>,
    /// Tailwind `shadow-lg`.
    pub lg: Arc<[BoxShadow]>,
}

/// One step of the type scale: a text size and the line height that goes with it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiTextStyle {
    /// Font size before `text_scale`.
    pub size: Pixels,
    /// Line height before `text_scale`.
    pub line_height: Pixels,
}

/// The type scale, Tailwind's by default. Components read sizes from here through
/// `primitives::text`, so `text_scale` and a custom scale reach every label.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiTypography {
    /// Captions and key caps (12/16).
    pub xs: UiTextStyle,
    /// Controls and body copy in dense desktop layouts (14/20); the Nova default.
    pub sm: UiTextStyle,
    /// Prose and dialog titles (16/24).
    pub base: UiTextStyle,
    /// Section headings (18/28).
    pub lg: UiTextStyle,
    /// Page headings (20/28).
    pub xl: UiTextStyle,
}

impl Default for UiTypography {
    fn default() -> Self {
        let style = |size: f32, line_height: f32| UiTextStyle {
            size: px(size),
            line_height: px(line_height),
        };
        Self {
            xs: style(12., 16.),
            sm: style(14., 20.),
            base: style(16., 24.),
            lg: style(18., 28.),
            xl: style(20., 28.),
        }
    }
}

/// Window widths at which layouts change shape.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiBreakpoints {
    /// Below this width, sidebars become sheets and drawers fill the window (768px).
    pub mobile: Pixels,
    /// Below this width, side drawers take three quarters of the window (640px).
    pub compact: Pixels,
}

impl Default for UiBreakpoints {
    fn default() -> Self {
        Self {
            mobile: px(768.),
            compact: px(640.),
        }
    }
}

/// Colors every form control shares, derived once from the palette and mode so
/// no component chooses its own dark-mode alpha. These follow shadcn Nova's
/// `dark:bg-input/30`, `dark:hover:bg-input/50` and `dark:border-input` rules.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlColors {
    /// Resting fill of inputs, checkboxes, radios and outline buttons.
    pub background: Rgba,
    /// Resting border of the same controls.
    pub border: Rgba,
    /// Hover fill of outline, ghost and toggle controls.
    pub hover: Rgba,
    /// Hover fill of ghost buttons and plain icon buttons.
    pub ghost_hover: Rgba,
    /// Fill of a pressed toggle or selected tab.
    pub pressed: Rgba,
    /// Unchecked switch track.
    pub track: Rgba,
    /// Switch thumb on the unchecked track.
    pub thumb: Rgba,
    /// Switch thumb on the checked (`primary`) track.
    pub thumb_checked: Rgba,
    /// Tinted destructive fill, for destructive buttons, badges and tags.
    pub destructive_background: Rgba,
    /// Hover of the tinted destructive fill.
    pub destructive_hover: Rgba,
    /// Hairline around tinted destructive chips.
    pub destructive_border: Rgba,
    /// Focus ring color at Nova's 50% opacity.
    pub ring: Rgba,
    /// Focus ring of an invalid control.
    pub invalid_ring: Rgba,
    /// Hairline around popovers and dialogs (`ring-foreground/10`).
    pub surface_border: Rgba,
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
    /// Text sizes and line heights, multiplied by `text_scale` when applied.
    pub typography: UiTypography,
    /// Window widths at which layouts change shape.
    pub breakpoints: UiBreakpoints,
    /// Syntax colors and surface of Markdown code blocks.
    pub code_theme: CodeTheme,
    /// Layout direction. [`locale::set_direction`] sets it and keeps it
    /// across theme swaps; [`UiTheme::set`] overwrites it with that setting.
    pub direction: Direction,
}

/// The syntax theme of Markdown code blocks, as in Mattermost's `codeTheme`.
///
/// `Default` follows the light or dark mode on the theme's muted code
/// surface. The named themes bring their own background and text color, so
/// a dark code theme stays readable in a light window.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum CodeTheme {
    /// GPUI Kit's light or dark syntax colors, by mode.
    #[default]
    Default,
    /// GitHub's light syntax colors.
    GitHub,
    /// Monokai: bright syntax on a dark olive surface.
    Monokai,
    /// Solarized, dark variant.
    SolarizedDark,
    /// Solarized, light variant.
    SolarizedLight,
}

impl CodeTheme {
    /// Every code theme, for a settings picker.
    pub const ALL: [CodeTheme; 5] = [
        CodeTheme::Default,
        CodeTheme::GitHub,
        CodeTheme::Monokai,
        CodeTheme::SolarizedDark,
        CodeTheme::SolarizedLight,
    ];

    /// Reads Mattermost's names: `github`, `monokai`, `solarized-dark`,
    /// `solarized-light`, and `default`.
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name.trim().to_ascii_lowercase().as_str() {
            "default" | "" => Self::Default,
            "github" => Self::GitHub,
            "monokai" => Self::Monokai,
            "solarized-dark" | "solarized_dark" => Self::SolarizedDark,
            "solarized-light" | "solarized_light" => Self::SolarizedLight,
            _ => return None,
        })
    }

    /// The name `from_name` reads.
    pub fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::GitHub => "github",
            Self::Monokai => "monokai",
            Self::SolarizedDark => "solarized-dark",
            Self::SolarizedLight => "solarized-light",
        }
    }

    /// The code block's own background and text color, or `None` for the
    /// theme's muted surface.
    pub fn surface(self) -> Option<(Rgba, Rgba)> {
        let hex = |value: u32| gpui_kit::rgb(value);
        match self {
            Self::Default => None,
            Self::GitHub => Some((hex(0xf6f8fa), hex(0x24292e))),
            Self::Monokai => Some((hex(0x272822), hex(0xf8f8f2))),
            Self::SolarizedDark => Some((hex(0x002b36), hex(0x839496))),
            Self::SolarizedLight => Some((hex(0xfdf6e3), hex(0x657b83))),
        }
    }

    /// The Kit highlight theme for this code theme in `mode`, built once.
    #[doc(hidden)]
    pub fn highlight_theme(
        self,
        mode: ThemeMode,
    ) -> Arc<gpui_kit::component::highlighter::HighlightTheme> {
        use gpui_kit::component::highlighter::HighlightTheme;
        use std::sync::OnceLock;
        static BUILT: [OnceLock<Arc<HighlightTheme>>; 4] = [const { OnceLock::new() }; 4];
        // (keyword, string, comment, function, number, type, tag, attribute,
        //  property, operator, variable)
        let (index, palette, dark) = match self {
            Self::Default => {
                return match mode {
                    ThemeMode::Light => HighlightTheme::default_light(),
                    ThemeMode::Dark => HighlightTheme::default_dark(),
                };
            }
            Self::GitHub => (
                0,
                [
                    "#d73a49", "#032f62", "#6a737d", "#6f42c1", "#005cc5", "#e36209", "#22863a",
                    "#6f42c1", "#005cc5", "#d73a49", "#24292e",
                ],
                false,
            ),
            Self::Monokai => (
                1,
                [
                    "#f92672", "#e6db74", "#75715e", "#a6e22e", "#ae81ff", "#66d9ef", "#f92672",
                    "#a6e22e", "#fd971f", "#f92672", "#f8f8f2",
                ],
                true,
            ),
            Self::SolarizedDark => (
                2,
                [
                    "#859900", "#2aa198", "#586e75", "#268bd2", "#d33682", "#b58900", "#268bd2",
                    "#93a1a1", "#268bd2", "#859900", "#839496",
                ],
                true,
            ),
            Self::SolarizedLight => (
                3,
                [
                    "#859900", "#2aa198", "#93a1a1", "#268bd2", "#d33682", "#b58900", "#268bd2",
                    "#586e75", "#268bd2", "#859900", "#657b83",
                ],
                false,
            ),
        };
        BUILT[index]
            .get_or_init(|| {
                let [
                    keyword,
                    string,
                    comment,
                    function,
                    number,
                    kind,
                    tag,
                    attribute,
                    property,
                    operator,
                    variable,
                ] = palette;
                let color = |value: &str| gpui_kit::private::serde_json::json!({ "color": value });
                let json = gpui_kit::private::serde_json::json!({
                    "name": self.name(),
                    "appearance": if dark { "dark" } else { "light" },
                    "style": {
                        "syntax": {
                            "keyword": color(keyword),
                            "string": color(string),
                            "string.escape": color(number),
                            "comment": { "color": comment, "font_style": "italic" },
                            "comment.doc": { "color": comment, "font_style": "italic" },
                            "function": color(function),
                            "constructor": color(function),
                            "number": color(number),
                            "boolean": color(number),
                            "constant": color(number),
                            "type": color(kind),
                            "enum": color(kind),
                            "tag": color(tag),
                            "attribute": color(attribute),
                            "property": color(property),
                            "operator": color(operator),
                            "variable": color(variable),
                            "punctuation": color(variable),
                            "title": { "color": function, "font_weight": 700 },
                            "link_uri": color(string),
                            "link_text": color(function),
                        }
                    }
                });
                Arc::new(
                    gpui_kit::private::serde_json::from_value::<HighlightTheme>(json)
                        .expect("the code theme palettes are valid"),
                )
            })
            .clone()
    }
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

    /// Installs light and dark themes together and shows the one the current
    /// [`Appearance`] selects (System until [`UiTheme::set_appearance`] says
    /// otherwise). [`follow_system`] keeps the pair in step with the OS.
    pub fn set_pair(cx: &mut App, light: Self, dark: Self) {
        let appearance = cx
            .try_global::<ThemePair>()
            .map_or(Appearance::System, |pair| pair.appearance);
        cx.set_global(ThemePair {
            light,
            dark,
            appearance,
        });
        Self::apply_appearance(cx);
    }

    /// Chooses which of the installed pair to show. Without a pair, the
    /// pinned Neutral palettes stand in.
    pub fn set_appearance(cx: &mut App, appearance: Appearance) {
        match cx.try_global::<ThemePair>() {
            Some(_) => cx.global_mut::<ThemePair>().appearance = appearance,
            None => cx.set_global(ThemePair {
                light: Self::neutral_light(),
                dark: Self::neutral_dark(),
                appearance,
            }),
        }
        Self::apply_appearance(cx);
    }

    /// The appearance the application asked for, System when none was set.
    pub fn appearance(cx: &App) -> Appearance {
        cx.try_global::<ThemePair>()
            .map_or(Appearance::System, |pair| pair.appearance)
    }

    /// Installs the pair member the appearance selects, if a pair is installed.
    fn apply_appearance(cx: &mut App) {
        let Some(pair) = cx.try_global::<ThemePair>().cloned() else {
            return;
        };
        let theme = match pair.appearance.resolve(cx.window_appearance()) {
            ThemeMode::Light => pair.light,
            ThemeMode::Dark => pair.dark,
        };
        let current = cx.try_global::<Self>();
        if current
            .is_some_and(|current| current.mode == theme.mode && current.colors == theme.colors)
        {
            return;
        }
        Self::set(cx, theme);
    }

    /// Installs a theme into the application.
    pub fn set(cx: &mut App, mut theme: Self) {
        theme.direction = locale::direction(cx);
        assert!(f32::from(theme.spacing.unit).is_finite() && theme.spacing.unit > px(0.));
        assert!(theme.text_scale.is_finite() && theme.text_scale > 0.);
        sync_component_theme(cx, theme.clone());
        cx.set_global(theme);
        cx.refresh_windows();
    }

    /// Reads the installed theme.
    ///
    /// Panics when the application has not installed a theme.
    #[track_caller]
    pub fn read(cx: &App) -> &Self {
        cx.try_global::<Self>().expect(
            "gpuicn theme is not installed: call `ui::theme::init(cx)` once at application startup",
        )
    }

    /// Builds a theme from shadcn theme CSS, such as the output of the shadcn
    /// theme picker or tweakcn.
    ///
    /// Light reads `:root`; dark reads `:root` and then `.dark`, like the
    /// browser cascade. Recognized `--background`, `--primary`, `--sidebar-*`,
    /// `--chart-*` and `--radius` declarations replace the Neutral defaults;
    /// other declarations are ignored. Colors may use `oklch()`, `hsl()`,
    /// bare HSL triplets, `rgb()` or hex. Build both modes and call
    /// [`UiTheme::set`] to switch; [`UiTheme::switch`] restores Neutral colors.
    pub fn from_shadcn_css(css: &str, mode: ThemeMode) -> Result<Self, ThemeCssError> {
        let blocks = css_blocks(css);
        let mut theme = match mode {
            ThemeMode::Light => Self::neutral_light(),
            ThemeMode::Dark => Self::neutral_dark(),
        };
        let mut selectors = vec![":root"];
        if mode == ThemeMode::Dark {
            selectors.push(".dark");
        }
        for selector in selectors {
            let declarations = blocks
                .iter()
                .filter(|(name, _)| name.split(',').any(|part| part.trim() == selector))
                .flat_map(|(_, body)| body.split(';'))
                .collect::<Vec<_>>();
            if declarations.is_empty() {
                return Err(ThemeCssError(format!("no `{selector}` block found")));
            }
            for declaration in declarations {
                let Some((name, value)) = declaration.split_once(':') else {
                    continue;
                };
                let (name, value) = (name.trim(), value.trim());
                let Some(name) = name.strip_prefix("--") else {
                    continue;
                };
                if name == "radius" {
                    theme.radius = UiRadius::new(css_length(value).ok_or_else(|| {
                        ThemeCssError(format!("unsupported length `--radius: {value}`"))
                    })?);
                } else if let Some(slot) = theme.colors.slot(name) {
                    *slot = css_color(value).ok_or_else(|| {
                        ThemeCssError(format!("unsupported color `--{name}: {value}`"))
                    })?;
                } else if let Some(slot) = theme.colors.optional_slot(name) {
                    *slot = Some(css_color(value).ok_or_else(|| {
                        ThemeCssError(format!("unsupported color `--{name}: {value}`"))
                    })?);
                }
            }
        }
        Ok(theme)
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
            typography: UiTypography::default(),
            breakpoints: UiBreakpoints::default(),
            code_theme: CodeTheme::Default,
            direction: Direction::Ltr,
            spacing: UiSpacing { unit: px(4.) },
            shadows: UiShadows {
                sm: Arc::from([
                    BoxShadow::new(px(0.), px(1.), black().alpha(0.10)).blur_radius(px(3.)),
                    BoxShadow::new(px(0.), px(1.), black().alpha(0.10))
                        .blur_radius(px(2.))
                        .spread_radius(px(-1.)),
                ]),
                md: Arc::from([
                    BoxShadow::new(px(0.), px(4.), black().alpha(0.10))
                        .blur_radius(px(6.))
                        .spread_radius(px(-1.)),
                    BoxShadow::new(px(0.), px(2.), black().alpha(0.10))
                        .blur_radius(px(4.))
                        .spread_radius(px(-2.)),
                ]),
                lg: Arc::from([
                    BoxShadow::new(px(0.), px(10.), black().alpha(0.10))
                        .blur_radius(px(15.))
                        .spread_radius(px(-3.)),
                    BoxShadow::new(px(0.), px(4.), black().alpha(0.10))
                        .blur_radius(px(6.))
                        .spread_radius(px(-4.)),
                ]),
            },
        }
    }

    /// The shared form-control colors for the active mode. See [`ControlColors`].
    pub fn control(&self) -> ControlColors {
        let colors = self.colors;
        let dark = self.mode == ThemeMode::Dark;
        ControlColors {
            background: if dark {
                colors.input.opacity(0.30)
            } else {
                colors.background.opacity(0.)
            },
            border: colors.input,
            hover: if dark {
                colors.input.opacity(0.50)
            } else {
                colors.muted
            },
            ghost_hover: if dark {
                colors.muted.opacity(0.50)
            } else {
                colors.muted
            },
            pressed: colors.muted,
            track: if dark {
                colors.input.opacity(0.80)
            } else {
                colors.input
            },
            thumb: if dark {
                colors.foreground
            } else {
                colors.background
            },
            thumb_checked: if dark {
                colors.primary_foreground
            } else {
                colors.background
            },
            destructive_background: colors.destructive.opacity(if dark { 0.20 } else { 0.10 }),
            destructive_hover: colors.destructive.opacity(if dark { 0.30 } else { 0.20 }),
            destructive_border: colors.destructive.opacity(0.20),
            ring: colors.ring.opacity(0.50),
            invalid_ring: colors.destructive.opacity(0.20),
            surface_border: colors.foreground.opacity(0.10),
        }
    }

    /// `color` composited over the `background` token: the same color with
    /// no transparency. A translucent fill lets a focus ring's shadow, which
    /// GPUI paints under the whole box, show through it; this one does not.
    pub fn opaque(&self, color: Rgba) -> Rgba {
        let under = self.colors.background;
        let alpha = color.a.clamp(0., 1.);
        let mix = |top: f32, bottom: f32| top * alpha + bottom * (1. - alpha);
        Rgba {
            r: mix(color.r, under.r),
            g: mix(color.g, under.g),
            b: mix(color.b, under.b),
            a: 1.,
        }
    }

    /// A step of the type scale multiplied by `text_scale`.
    pub fn text_style(&self, style: UiTextStyle) -> UiTextStyle {
        UiTextStyle {
            size: style.size * self.text_scale,
            line_height: style.line_height * self.text_scale,
        }
    }
}

fn sync_component_theme(cx: &mut App, theme: UiTheme) {
    use gpui_kit::component::{Theme, ThemeTokens};

    if !cx.has_global::<Theme>() {
        return;
    }

    let colors = theme.colors;
    let component = Theme::global_mut(cx);
    component.mode = match theme.mode {
        ThemeMode::Light => gpui_kit::component::ThemeMode::Light,
        ThemeMode::Dark => gpui_kit::component::ThemeMode::Dark,
    };
    // Start from Kit's palette for this mode, so colors gpuicn doesn't set
    // below, such as scrollbar tracks, never keep the other mode's values.
    component.colors = *match theme.mode {
        ThemeMode::Light => gpui_kit::component::ThemeColor::light(),
        ThemeMode::Dark => gpui_kit::component::ThemeColor::dark(),
    };
    component.font_family = theme.fonts.body;
    component.mono_font_family = theme.fonts.mono;
    component.radius = theme.radius.md;
    component.radius_lg = theme.radius.lg;
    component.background = colors.background.into();
    component.foreground = colors.foreground.into();
    component.popover = colors.popover.into();
    component.popover_foreground = colors.popover_foreground.into();
    component.primary = colors.primary.into();
    component.primary_foreground = colors.primary_foreground.into();
    component.secondary = colors.secondary.into();
    component.secondary_foreground = colors.secondary_foreground.into();
    component.muted = colors.muted.into();
    component.muted_foreground = colors.muted_foreground.into();
    component.accent = colors.accent.into();
    component.accent_foreground = colors.accent_foreground.into();
    component.primary_hover = colors.primary.opacity(0.9).into();
    component.primary_active = colors.primary.opacity(0.8).into();
    component.secondary_hover = colors.secondary.opacity(0.8).into();
    component.secondary_active = colors.secondary.opacity(0.7).into();
    component.border = colors.border.into();
    component.input = colors.input.into();
    component.ring = colors.ring.into();
    component.overlay = colors.overlay.into();
    component.caret = colors.foreground.into();
    component.window_border = colors.border.into();
    component.danger = colors.destructive.into();
    component.danger_foreground = colors.destructive_foreground.into();
    component.danger_hover = colors.destructive.opacity(0.9).into();
    component.danger_active = colors.destructive.opacity(0.8).into();
    component.success = colors.success.into();
    component.success_foreground = colors.success_foreground.into();
    component.success_hover = colors.success.opacity(0.9).into();
    component.success_active = colors.success.opacity(0.8).into();
    component.warning = colors.warning.into();
    component.warning_foreground = colors.warning_foreground.into();
    component.warning_hover = colors.warning.opacity(0.9).into();
    component.warning_active = colors.warning.opacity(0.8).into();
    component.info = colors.info.into();
    component.info_foreground = colors.info_foreground.into();
    component.info_hover = colors.info.opacity(0.9).into();
    component.info_active = colors.info.opacity(0.8).into();
    component.chart_1 = colors.chart_1.into();
    component.chart_2 = colors.chart_2.into();
    component.chart_3 = colors.chart_3.into();
    component.chart_4 = colors.chart_4.into();
    component.chart_5 = colors.chart_5.into();
    component.chart_bullish = colors.success.into();
    component.chart_bearish = colors.destructive.into();
    component.chart_grid = colors.border.opacity(0.6).into();
    // Buttons: Kit's plain button is shadcn `outline`; the variants follow their token.
    component.button = colors.background.into();
    component.button_foreground = colors.foreground.into();
    component.button_hover = colors.accent.into();
    component.button_active = colors.muted.into();
    component.button_primary = component.primary;
    component.button_primary_foreground = component.primary_foreground;
    component.button_primary_hover = component.primary_hover;
    component.button_primary_active = component.primary_active;
    component.button_secondary = component.secondary;
    component.button_secondary_foreground = component.secondary_foreground;
    component.button_secondary_hover = component.secondary_hover;
    component.button_secondary_active = component.secondary_active;
    component.button_danger = component.danger;
    component.button_danger_foreground = component.danger_foreground;
    component.button_danger_hover = component.danger_hover;
    component.button_danger_active = component.danger_active;
    component.button_success = component.success;
    component.button_success_foreground = component.success_foreground;
    component.button_success_hover = component.success_hover;
    component.button_success_active = component.success_active;
    component.button_warning = component.warning;
    component.button_warning_foreground = component.warning_foreground;
    component.button_warning_hover = component.warning_hover;
    component.button_warning_active = component.warning_active;
    component.button_info = component.info;
    component.button_info_foreground = component.info_foreground;
    component.button_info_hover = component.info_hover;
    component.button_info_active = component.info_active;
    // Nova scrollbars float over content with no track.
    component.scrollbar = gpui_kit::transparent_black();
    component.scrollbar_thumb = colors.border.into();
    component.scrollbar_thumb_hover = colors.muted_foreground.opacity(0.72).into();
    component.sidebar = colors.sidebar.into();
    component.sidebar_foreground = colors.sidebar_foreground.into();
    component.sidebar_border = colors.sidebar_border.into();
    component.sidebar_accent = colors.sidebar_accent.into();
    component.sidebar_accent_foreground = colors.sidebar_accent_foreground.into();
    component.sidebar_primary = colors.sidebar_primary.into();
    component.sidebar_primary_foreground = colors.sidebar_primary_foreground.into();
    component.title_bar = colors.sidebar.into();
    component.title_bar_border = colors.border.into();
    component.status_bar = colors.sidebar.into();
    component.status_bar_border = colors.border.into();
    // Rich text: links, selection, table headings and code highlighting.
    component.link = colors.link().into();
    component.link_hover = colors.link().opacity(0.9).into();
    component.link_active = colors.link().opacity(0.8).into();
    component.selection = colors.ring.opacity(0.3).into();
    component.description_list_label = colors.muted.into();
    component.description_list_label_foreground = colors.muted_foreground.into();
    component.group_box = colors.card.into();
    component.group_box_foreground = colors.card_foreground.into();
    component.accordion = colors.card.into();
    // Lists and tables share the `muted` zebra and the `accent` hover of menus.
    component.colors.list = colors.background.into();
    component.list_even = colors.muted.opacity(0.5).into();
    component.list_head = colors.muted.into();
    component.list_hover = colors.accent.into();
    component.list_active = colors.accent.into();
    component.list_active_border = colors.ring.into();
    component.table = colors.background.into();
    component.table_even = colors.muted.opacity(0.5).into();
    component.table_hover = colors.muted.opacity(0.5).into();
    component.table_active = colors.accent.into();
    component.table_active_border = colors.ring.into();
    component.table_row_border = colors.border.into();
    component.table_head = colors.muted.into();
    component.table_head_foreground = colors.foreground.into();
    component.table_foot = colors.muted.opacity(0.5).into();
    component.table_foot_foreground = colors.foreground.into();
    // Controls: shadcn tracks are `input`, thumbs and active tabs are `background`.
    component.switch = colors.input.into();
    component.switch_thumb = colors.background.into();
    component.slider_bar = colors.primary.into();
    component.slider_thumb = colors.background.into();
    component.progress_bar = colors.primary.into();
    component.skeleton = colors.accent.into();
    component.tab_bar = colors.muted.into();
    component.tab_bar_segmented = colors.muted.into();
    component.tab_foreground = colors.muted_foreground.into();
    component.tab_active = colors.background.into();
    component.tab_active_foreground = colors.foreground.into();
    component.drag_border = colors.ring.into();
    component.drop_target = colors.ring.opacity(0.15).into();
    component.highlight_theme = theme.code_theme.highlight_theme(theme.mode);
    component.tokens = ThemeTokens::from(&component.colors);

    Theme::sync_base(cx);
}

/// Why [`UiTheme::from_shadcn_css`] could not read a theme.
#[derive(Clone, Debug, PartialEq)]
pub struct ThemeCssError(pub String);

impl std::fmt::Display for ThemeCssError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "shadcn theme CSS: {}", self.0)
    }
}

impl std::error::Error for ThemeCssError {}

impl UiColors {
    #[doc(hidden)]
    pub fn slot(&mut self, variable: &str) -> Option<&mut Rgba> {
        Some(match variable {
            "background" => &mut self.background,
            "foreground" => &mut self.foreground,
            "card" => &mut self.card,
            "card-foreground" => &mut self.card_foreground,
            "popover" => &mut self.popover,
            "popover-foreground" => &mut self.popover_foreground,
            "primary" => &mut self.primary,
            "primary-foreground" => &mut self.primary_foreground,
            "secondary" => &mut self.secondary,
            "secondary-foreground" => &mut self.secondary_foreground,
            "muted" => &mut self.muted,
            "muted-foreground" => &mut self.muted_foreground,
            "accent" => &mut self.accent,
            "accent-foreground" => &mut self.accent_foreground,
            "destructive" => &mut self.destructive,
            "destructive-foreground" => &mut self.destructive_foreground,
            "success" => &mut self.success,
            "success-foreground" => &mut self.success_foreground,
            "warning" => &mut self.warning,
            "warning-foreground" => &mut self.warning_foreground,
            "info" => &mut self.info,
            "info-foreground" => &mut self.info_foreground,
            "border" => &mut self.border,
            "input" => &mut self.input,
            "ring" => &mut self.ring,
            "chart-1" => &mut self.chart_1,
            "chart-2" => &mut self.chart_2,
            "chart-3" => &mut self.chart_3,
            "chart-4" => &mut self.chart_4,
            "chart-5" => &mut self.chart_5,
            "sidebar" | "sidebar-background" => &mut self.sidebar,
            "sidebar-foreground" => &mut self.sidebar_foreground,
            "sidebar-primary" => &mut self.sidebar_primary,
            "sidebar-primary-foreground" => &mut self.sidebar_primary_foreground,
            "sidebar-accent" => &mut self.sidebar_accent,
            "sidebar-accent-foreground" => &mut self.sidebar_accent_foreground,
            "sidebar-border" => &mut self.sidebar_border,
            "sidebar-ring" => &mut self.sidebar_ring,
            _ => return None,
        })
    }

    /// The optional tokens that follow another one until set.
    #[doc(hidden)]
    pub fn optional_slot(&mut self, variable: &str) -> Option<&mut Option<Rgba>> {
        Some(match variable {
            "link" => &mut self.link,
            "mention-highlight" => &mut self.mention_highlight,
            "mention-highlight-foreground" => &mut self.mention_highlight_foreground,
            "new-messages" => &mut self.new_messages,
            "sidebar-muted-foreground" => &mut self.sidebar_muted_foreground,
            "sidebar-badge" => &mut self.sidebar_badge,
            "sidebar-badge-foreground" => &mut self.sidebar_badge_foreground,
            _ => return None,
        })
    }
}

/// Innermost `selector { body }` pairs with comments removed.
fn css_blocks(css: &str) -> Vec<(String, String)> {
    let mut text = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        text.push_str(&rest[..start]);
        rest = rest[start + 2..]
            .split_once("*/")
            .map_or("", |(_, after)| after);
    }
    text.push_str(rest);
    let mut blocks = Vec::new();
    let mut selector_start = 0;
    let mut open = None;
    for (index, character) in text.char_indices() {
        match character {
            '{' => {
                open = Some((text[selector_start..index].trim().to_owned(), index + 1));
                selector_start = index + 1;
            }
            '}' => {
                if let Some((selector, body)) = open.take() {
                    blocks.push((selector, text[body..index].to_owned()));
                }
                selector_start = index + 1;
            }
            ';' if open.is_none() => selector_start = index + 1,
            _ => {}
        }
    }
    blocks
}

fn css_length(value: &str) -> Option<Pixels> {
    let value = value.trim();
    let pixels = if let Some(rem) = value.strip_suffix("rem") {
        rem.trim().parse::<f32>().ok()? * 16.
    } else if let Some(pixels) = value.strip_suffix("px") {
        pixels.trim().parse::<f32>().ok()?
    } else {
        value.parse::<f32>().ok().filter(|value| *value == 0.)?
    };
    (pixels.is_finite() && pixels >= 0.).then(|| px(pixels))
}

/// A CSS number or percentage; `percent` is the value of `100%`.
fn css_number(value: &str, percent: f64) -> Option<f64> {
    let value = value.trim();
    let number = match value.strip_suffix('%') {
        Some(value) => value.trim().parse::<f64>().ok()? / 100. * percent,
        None => value
            .strip_suffix("deg")
            .unwrap_or(value)
            .parse::<f64>()
            .ok()?,
    };
    number.is_finite().then_some(number)
}

fn css_color(value: &str) -> Option<Rgba> {
    let value = value.trim().trim_end_matches("!important").trim();
    if let Some(hex) = value.strip_prefix('#') {
        return css_hex(hex);
    }
    let (function, arguments) = match value.split_once('(') {
        Some((function, arguments)) => (
            function.trim().to_ascii_lowercase(),
            arguments.strip_suffix(')')?,
        ),
        // shadcn v3 themes store bare HSL triplets such as `0 0% 100%`.
        None => ("hsl".to_owned(), value),
    };
    let (channels, alpha) = match arguments.split_once('/') {
        Some((channels, alpha)) => (channels, css_number(alpha, 1.)?),
        None => (arguments, 1.),
    };
    let channels: Vec<_> = channels
        .split(|character: char| character == ',' || character.is_whitespace())
        .filter(|part| !part.is_empty())
        .collect();
    let (channels, alpha) = match channels.len() {
        3 => (channels, alpha),
        4 => (channels[..3].to_vec(), css_number(channels[3], 1.)?),
        _ => return None,
    };
    let color = match function.as_str() {
        "oklch" => oklch(
            css_number(channels[0], 1.)?,
            css_number(channels[1], 0.4)?,
            css_number(channels[2], 1.)?,
        ),
        "hsl" | "hsla" => hsl(
            css_number(channels[0], 1.)?,
            css_number(channels[1], 1.)?,
            css_number(channels[2], 1.)?,
        ),
        "rgb" | "rgba" => [
            css_number(channels[0], 255.)? / 255.,
            css_number(channels[1], 255.)? / 255.,
            css_number(channels[2], 255.)? / 255.,
        ],
        _ => return None,
    };
    Some(Rgba {
        r: color[0].clamp(0., 1.) as f32,
        g: color[1].clamp(0., 1.) as f32,
        b: color[2].clamp(0., 1.) as f32,
        a: alpha.clamp(0., 1.) as f32,
    })
}

fn css_hex(hex: &str) -> Option<Rgba> {
    if !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let digits: Vec<u8> = match hex.len() {
        3 | 4 => hex
            .chars()
            .map(|digit| u8::from_str_radix(&format!("{digit}{digit}"), 16).ok())
            .collect::<Option<_>>()?,
        6 | 8 => (0..hex.len())
            .step_by(2)
            .map(|index| u8::from_str_radix(&hex[index..index + 2], 16).ok())
            .collect::<Option<_>>()?,
        _ => return None,
    };
    Some(Rgba {
        r: digits[0] as f32 / 255.,
        g: digits[1] as f32 / 255.,
        b: digits[2] as f32 / 255.,
        a: digits.get(3).map_or(1., |alpha| *alpha as f32 / 255.),
    })
}

/// OKLCH to gamma-encoded sRGB, matching the pinned Neutral conversion when chroma is 0.
fn oklch(lightness: f64, chroma: f64, hue: f64) -> [f64; 3] {
    let (a, b) = (
        chroma * hue.to_radians().cos(),
        chroma * hue.to_radians().sin(),
    );
    let l = (lightness + 0.396_337_777_4 * a + 0.215_803_757_3 * b).powi(3);
    let m = (lightness - 0.105_561_345_8 * a - 0.063_854_172_8 * b).powi(3);
    let s = (lightness - 0.089_484_177_5 * a - 1.291_485_548 * b).powi(3);
    [
        4.076_741_662_1 * l - 3.307_711_591_3 * m + 0.230_969_929_2 * s,
        -1.268_438_004_6 * l + 2.609_757_401_1 * m - 0.341_319_396_5 * s,
        -0.004_196_086_3 * l - 0.703_418_614_7 * m + 1.707_614_701 * s,
    ]
    .map(srgb_channel)
}

fn hsl(hue: f64, saturation: f64, lightness: f64) -> [f64; 3] {
    let hue = hue.rem_euclid(360.) / 60.;
    let chroma = (1. - (2. * lightness - 1.).abs()) * saturation;
    let x = chroma * (1. - (hue % 2. - 1.).abs());
    let (r, g, b) = match hue as u32 {
        0 => (chroma, x, 0.),
        1 => (x, chroma, 0.),
        2 => (0., chroma, x),
        3 => (0., x, chroma),
        4 => (x, 0., chroma),
        _ => (chroma, 0., x),
    };
    let base = lightness - chroma / 2.;
    [r + base, g + base, b + base]
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
        destructive_foreground: neutral(0.985),
        success: srgb(0x16, 0xa3, 0x4a),
        success_foreground: neutral(0.985),
        warning: srgb(0xd9, 0x77, 0x06),
        warning_foreground: neutral(0.985),
        info: srgb(0x25, 0x63, 0xeb),
        info_foreground: neutral(0.985),
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
        link: None,
        mention_highlight: None,
        mention_highlight_foreground: None,
        new_messages: None,
        sidebar_muted_foreground: None,
        sidebar_badge: None,
        sidebar_badge_foreground: None,
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
        destructive_foreground: neutral(0.985),
        success: srgb(0x4a, 0xde, 0x80),
        success_foreground: neutral(0.145),
        warning: srgb(0xfb, 0xbf, 0x24),
        warning_foreground: neutral(0.145),
        info: srgb(0x60, 0xa5, 0xfa),
        info_foreground: neutral(0.145),
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
        // shadcn pairs black/10 with a backdrop blur GPUI lacks; without it a
        // 10% scrim disappears on a near-black page.
        overlay: black().alpha(0.50).into(),
        link: None,
        mention_highlight: None,
        mention_highlight_foreground: None,
        new_messages: None,
        sidebar_muted_foreground: None,
        sidebar_badge: None,
        sidebar_badge_foreground: None,
    }
}

#[doc(hidden)]
pub fn neutral(l: f64) -> Rgba {
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
    fn code_themes_build_with_their_own_surface() {
        for theme in CodeTheme::ALL {
            assert_eq!(CodeTheme::from_name(theme.name()), Some(theme));
            let built = theme.highlight_theme(ThemeMode::Light);
            // Built once: the Kit shares one highlighter per theme instance.
            assert!(Arc::ptr_eq(
                &built,
                &theme.highlight_theme(ThemeMode::Light)
            ));
            if theme.surface().is_some() {
                assert_eq!(built.name, theme.name());
                assert!(built.keyword.is_some());
            }
        }
        assert_eq!(
            CodeTheme::from_name("Solarized-Dark"),
            Some(CodeTheme::SolarizedDark)
        );
        assert_eq!(CodeTheme::from_name("dracula"), None);
    }

    #[test]
    fn optional_tokens_follow_their_base_until_set() {
        let mut theme = UiTheme::neutral_light();
        theme.colors.primary = gpui_kit::rgb(0x1c58d9);
        assert_eq!(theme.colors.link(), theme.colors.primary);
        assert_eq!(theme.colors.mention_highlight(), theme.colors.primary);
        assert_eq!(theme.colors.new_messages(), theme.colors.destructive);
        assert_eq!(theme.colors.sidebar_badge(), theme.colors.sidebar_primary);
        theme.colors.link = Some(gpui_kit::rgb(0x2389d7));
        assert_eq!(theme.colors.link(), gpui_kit::rgb(0x2389d7));
        let css = ":root { --primary: #111111; --link: #2389d7; --sidebar-badge: #ff0000; }";
        let parsed = UiTheme::from_shadcn_css(css, ThemeMode::Light).unwrap();
        assert_eq!(parsed.colors.link(), gpui_kit::rgb(0x2389d7));
        assert_eq!(parsed.colors.sidebar_badge(), gpui_kit::rgb(0xff0000));
        assert_eq!(parsed.colors.mention_highlight(), gpui_kit::rgb(0x111111));
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

    fn close(left: Rgba, right: Rgba) -> bool {
        [
            (left.r, right.r),
            (left.g, right.g),
            (left.b, right.b),
            (left.a, right.a),
        ]
        .iter()
        .all(|(left, right)| (left - right).abs() < 1.5 / 255.)
    }

    #[test]
    fn imports_shadcn_theme_css() {
        let css = r#"
            @layer base {
              :root {
                --radius: 0.5rem; /* 8px */
                --background: oklch(1 0 0);
                --foreground: oklch(0.145 0 0);
                --primary: #2563eb;
                --ring: rgb(37 99 235 / 50%);
                --sidebar-background: 0 0% 98%;
                --font-sans: Inter, sans-serif;
              }
              .dark {
                --background: oklch(0.145 0 0);
                --border: oklch(1 0 0 / 10%);
                --sidebar-primary: oklch(0.488 0.243 264.376);
                --chart-1: hsl(220 70% 50%);
              }
            }
        "#;
        let light = UiTheme::from_shadcn_css(css, ThemeMode::Light).unwrap();
        assert_eq!(light.mode, ThemeMode::Light);
        assert_eq!(light.radius.lg, px(8.));
        assert!(close(light.colors.background, neutral(1.0)));
        assert!(close(light.colors.foreground, neutral(0.145)));
        assert_eq!(light.colors.primary, srgb(0x25, 0x63, 0xeb));
        assert!(close(
            light.colors.ring,
            Rgba {
                a: 0.5,
                ..srgb(37, 99, 235)
            }
        ));
        assert!(close(light.colors.sidebar, neutral(0.985)));
        // Variables the CSS leaves out keep the Neutral defaults.
        assert_eq!(light.colors.muted, neutral_light_colors().muted);

        let dark = UiTheme::from_shadcn_css(css, ThemeMode::Dark).unwrap();
        assert_eq!(dark.mode, ThemeMode::Dark);
        assert!(close(dark.colors.background, neutral(0.145)));
        // `.dark` inherits `:root` values it does not override.
        assert_eq!(dark.colors.primary, srgb(0x25, 0x63, 0xeb));
        assert!(close(dark.colors.border, neutral_alpha(1.0, 0.10)));
        assert!(close(dark.colors.sidebar_primary, srgb(0x14, 0x47, 0xe6)));
        assert!(close(dark.colors.chart_1, srgb(0x26, 0x63, 0xd9)));

        assert!(UiTheme::from_shadcn_css(":root { --primary: blue; }", ThemeMode::Light).is_err());
        assert!(UiTheme::from_shadcn_css(":root { --radius: 1em; }", ThemeMode::Light).is_err());
        assert!(UiTheme::from_shadcn_css(":root { }", ThemeMode::Dark).is_err());
        assert_eq!(
            css_hex("fff8"),
            Some(Rgba {
                r: 1.,
                g: 1.,
                b: 1.,
                a: 0x88 as f32 / 255.
            })
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

    #[gpui_kit::test]
    fn keeps_component_state_tokens_in_sync_with_ui_theme(cx: &mut gpui_kit::TestAppContext) {
        cx.update(|cx| {
            init(cx);
            let component = gpui_kit::component::Theme::global(cx);
            assert_eq!(component.mode, gpui_kit::component::ThemeMode::Light);
            assert_eq!(
                component.button_hover,
                UiTheme::read(cx).colors.accent.into()
            );

            UiTheme::switch(cx, ThemeMode::Dark);
            let component = gpui_kit::component::Theme::global(cx);
            assert_eq!(component.mode, gpui_kit::component::ThemeMode::Dark);
            assert_eq!(component.ring, UiTheme::read(cx).colors.ring.into());

            let mut custom = UiTheme::read(cx).clone();
            custom.colors.accent = srgb(0x22, 0x66, 0xaa);
            custom.fonts.body = "Custom Sans".into();
            custom.radius = UiRadius::new(px(12.));
            let expected_accent = custom.colors.accent;
            UiTheme::set(cx, custom);

            let component = gpui_kit::component::Theme::global(cx);
            assert_eq!(component.button_hover, expected_accent.into());
            assert_eq!(component.font_family, "Custom Sans");
            assert_eq!(component.radius, px(9.6));
            assert_eq!(
                component.tokens.button_hover.background,
                expected_accent.into()
            );

            init(cx);
            assert_eq!(
                gpui_kit::component::Theme::global(cx).button_hover,
                expected_accent.into()
            );
        });
    }
}

// Refine the actual control after default styling; no extra layout or focus node.
#[doc(hidden)]
pub fn apply_style<T: Styled>(mut element: T, style: &gpui_kit::StyleRefinement) -> T {
    use gpui_kit::Refineable as _;
    element.style().refine(style);
    element
}

#[derive(gpui_kit::IntoElement)]
#[doc(hidden)]
pub struct DisclosureIcon {
    icon: gpui_kit::Svg,
    open: bool,
}
#[doc(hidden)]
pub fn disclosure_icon(icon: gpui_kit::Svg, open: bool) -> DisclosureIcon {
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
#[doc(hidden)]
pub struct RovingFocus {
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
        let direction = UiTheme::read(cx).direction;
        let next = match direction.logical_key(event.keystroke.key.as_str()) {
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
