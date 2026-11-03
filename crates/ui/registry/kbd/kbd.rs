//! Nova-styled keyboard shortcut hints.

use gpui_icons::{LucideIcon, lucide};
use gpui_kit::{
    Action, AnyElement, App, AsKeystroke as _, Div, FontWeight, IntoElement, Keystroke,
    OperatingSystem, ParentElement, RenderOnce, SharedString, Styled, Window, div, relative,
};

use super::theme::UiTheme;

/// A noninteractive key cap that shows one key or shortcut.
#[derive(IntoElement)]
pub struct Kbd {
    style: gpui_kit::StyleRefinement,
    keys: Keys,
}

enum Keys {
    Text(SharedString),
    Shortcut(SharedString),
    Keystroke(Keystroke),
}

impl Kbd {
    /// Shows `keys` as written, for example `"Esc"` or `"⌘K"`.
    pub fn new(keys: impl Into<SharedString>) -> Self {
        Self::with(Keys::Text(keys.into()))
    }

    /// Shows a GPUI keystroke with the platform's modifier names, such as `⌘K` on
    /// macOS and `Ctrl+K` elsewhere.
    pub fn keystroke(keystroke: &Keystroke) -> Self {
        Self::with(Keys::Keystroke(keystroke.clone()))
    }

    /// Shows a keystroke written like a key binding, such as `"secondary-k"`, for
    /// the platform the app runs on. Text that does not parse is shown as written.
    pub fn shortcut(source: impl Into<SharedString>) -> Self {
        Self::with(Keys::Shortcut(source.into()))
    }

    /// Shows the first keystroke bound to `action` in the window, if any.
    pub fn for_action(action: &dyn Action, window: &Window) -> Option<Self> {
        let binding = window.highest_precedence_binding_for_action(action)?;
        let keystroke = binding.keystrokes().first()?;
        Some(Self::keystroke(keystroke.as_keystroke()))
    }

    fn with(keys: Keys) -> Self {
        Self {
            style: gpui_kit::StyleRefinement::default(),
            keys,
        }
    }
}

/// Formats a key-binding string such as `"secondary-shift-p"` for `os`, or
/// returns it unchanged when it does not parse.
pub fn format_shortcut(source: &str, os: OperatingSystem) -> SharedString {
    match Keystroke::parse_with_os(source, os) {
        Ok(keystroke) => format_keystroke(&keystroke, os),
        Err(_) => SharedString::from(source.to_string()),
    }
}

/// Formats a keystroke the way `os` shows shortcuts in menus: modifier glyphs
/// such as `⌃⌥⇧⌘K` on macOS, and `Ctrl+Alt+Shift+K` elsewhere.
pub fn format_keystroke(keystroke: &Keystroke, os: OperatingSystem) -> SharedString {
    let mac = matches!(os, OperatingSystem::Mac | OperatingSystem::Ios);
    let modifiers = keystroke.modifiers;
    let mut parts: Vec<&str> = Vec::new();
    for (on, glyph, name) in [
        (modifiers.control, "⌃", "Ctrl"),
        (modifiers.alt, "⌥", "Alt"),
        (modifiers.shift, "⇧", "Shift"),
        (
            modifiers.platform,
            "⌘",
            if os == OperatingSystem::Windows {
                "Win"
            } else {
                "Super"
            },
        ),
    ] {
        if on {
            parts.push(if mac { glyph } else { name });
        }
    }
    let key = keystroke.key.as_str();
    let named = match (key, mac) {
        ("enter", true) => Some("↩"),
        ("escape", true) => Some("⎋"),
        ("backspace" | "delete", true) => Some("⌫"),
        ("tab", true) => Some("⇥"),
        ("up", true) => Some("↑"),
        ("down", true) => Some("↓"),
        ("left", true) => Some("←"),
        ("right", true) => Some("→"),
        ("escape", false) => Some("Esc"),
        ("pageup", _) => Some("Page Up"),
        ("pagedown", _) => Some("Page Down"),
        _ => None,
    };
    let key = match named {
        Some(named) => named.to_string(),
        None => {
            let mut chars = key.chars();
            chars
                .next()
                .map(|first| first.to_uppercase().chain(chars).collect())
                .unwrap_or_default()
        }
    };
    parts.push(&key);
    parts.join(if mac { "" } else { "+" }).into()
}

/// Lays out several key caps in a row, for sequences such as `Ctrl` `B`.
pub fn kbd_group(keys: impl IntoIterator<Item = Kbd>, cx: &App) -> Div {
    let theme = UiTheme::read(cx);
    div()
        .flex()
        .flex_shrink_0()
        .items_center()
        .gap(theme.space(1.))
        .children(keys)
}

impl RenderOnce for Kbd {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx);
        let root = div()
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .h(theme.space(5.))
            .min_w(theme.space(5.))
            .px(theme.space(1.))
            .rounded(theme.radius.sm)
            .bg(theme.colors.muted)
            .text_color(theme.colors.muted_foreground)
            .font_family(theme.fonts.body.clone())
            .font_weight(FontWeight::MEDIUM)
            .text_size(theme.text(12.))
            .line_height(relative(1.))
            .whitespace_nowrap()
            .gap(theme.space(0.5))
            .children(key_parts(
                &match self.keys {
                    Keys::Text(text) => text,
                    Keys::Shortcut(source) => format_shortcut(&source, cx.platform_info().os),
                    Keys::Keystroke(keystroke) => {
                        format_keystroke(&keystroke, cx.platform_info().os)
                    }
                },
                theme.text(12.),
                // Icons don't inherit text color, so follow a caller's override.
                self.style
                    .text
                    .color
                    .unwrap_or(theme.colors.muted_foreground.into()),
            ));
        super::theme::apply_style(root, &self.style).into_any_element()
    }
}

/// The icon drawn for a key symbol. UI fonts such as Geist have no glyphs for
/// most of these, and the browser build has no system font to fall back on.
fn key_icon(symbol: char) -> Option<LucideIcon> {
    Some(match symbol {
        '⌘' => LucideIcon::Command,
        '⌥' => LucideIcon::Option,
        '⇧' => LucideIcon::ArrowBigUp,
        '⌃' => LucideIcon::ChevronUp,
        '↩' => LucideIcon::CornerDownLeft,
        '⌫' => LucideIcon::Delete,
        '⇥' => LucideIcon::ArrowRightToLine,
        '↑' => LucideIcon::ArrowUp,
        '↓' => LucideIcon::ArrowDown,
        '←' => LucideIcon::ArrowLeft,
        '→' => LucideIcon::ArrowRight,
        _ => return None,
    })
}

/// Draws formatted keys such as `"⇧⌘P"` inline, with key symbols as icons, for
/// shortcut hints outside a key cap.
pub fn key_label(keys: &str, size: gpui_kit::Pixels, color: gpui_kit::Hsla) -> Div {
    div()
        .flex()
        .items_center()
        .gap(size / 6.)
        .text_color(color)
        .children(key_parts(keys, size, color))
}

/// Splits formatted keys into text runs and symbol icons.
fn key_parts(keys: &str, size: gpui_kit::Pixels, color: gpui_kit::Hsla) -> Vec<AnyElement> {
    let mut parts = Vec::new();
    let mut text = String::new();
    for symbol in keys.chars() {
        if let Some(icon) = key_icon(symbol) {
            if !text.is_empty() {
                parts.push(std::mem::take(&mut text).into_any_element());
            }
            parts.push(lucide(icon).size(size).text_color(color).into_any_element());
        } else if symbol == '⎋' {
            text.push_str("Esc");
        } else {
            text.push(symbol);
        }
    }
    if !text.is_empty() {
        parts.push(text.into_any_element());
    }
    parts
}

impl Styled for Kbd {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}

#[cfg(test)]
mod tests {
    use super::{OperatingSystem, format_shortcut};

    #[test]
    fn shortcuts_follow_platform_conventions() {
        let mac = OperatingSystem::Mac;
        let linux = OperatingSystem::Linux;
        assert_eq!(format_shortcut("secondary-k", mac).as_ref(), "⌘K");
        assert_eq!(format_shortcut("secondary-k", linux).as_ref(), "Ctrl+K");
        assert_eq!(format_shortcut("secondary-shift-p", mac).as_ref(), "⇧⌘P");
        assert_eq!(
            format_shortcut("ctrl-alt-shift-a", linux).as_ref(),
            "Ctrl+Alt+Shift+A"
        );
        assert_eq!(format_shortcut("escape", linux).as_ref(), "Esc");
        assert_eq!(format_shortcut("alt-up", mac).as_ref(), "⌥↑");
        assert_eq!(format_shortcut("secondary-,", linux).as_ref(), "Ctrl+,");
    }
}
