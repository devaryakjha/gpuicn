//! Nova-styled keyboard shortcut hints.

use gpui_kit::{
    Action, App, AsKeystroke as _, Div, FontWeight, IntoElement, Keystroke, ParentElement,
    RenderOnce, SharedString, Styled, Window, div, relative,
};

use super::theme::UiTheme;

/// A noninteractive key cap that shows one key or shortcut.
#[derive(IntoElement)]
pub struct Kbd {
    style: gpui_kit::StyleRefinement,
    keys: SharedString,
}

impl Kbd {
    /// Shows `keys` as written, for example `"Esc"` or `"⌘K"`.
    pub fn new(keys: impl Into<SharedString>) -> Self {
        Self {
            style: gpui_kit::StyleRefinement::default(),
            keys: keys.into(),
        }
    }

    /// Shows a GPUI keystroke with the platform's modifier names, such as `⌘K` on
    /// macOS and `Ctrl+K` elsewhere.
    pub fn keystroke(keystroke: &Keystroke) -> Self {
        Self::new(format_keystroke(keystroke))
    }

    /// Parses a GPUI keystroke such as `"secondary-k"` and shows it for the current
    /// platform. Text that does not parse is shown as written.
    pub fn shortcut(source: &'static str) -> Self {
        match Keystroke::parse(source) {
            Ok(keystroke) => Self::keystroke(&keystroke),
            Err(_) => Self::new(source),
        }
    }

    /// Shows the first keystroke bound to `action` in the window, if any.
    pub fn for_action(action: &dyn Action, window: &Window) -> Option<Self> {
        let binding = window.highest_precedence_binding_for_action(action)?;
        let keystroke = binding.keystrokes().first()?;
        Some(Self::keystroke(keystroke.as_keystroke()))
    }
}

/// Formats a keystroke the way the platform's menus show it.
pub fn format_keystroke(keystroke: &Keystroke) -> SharedString {
    gpui_kit::component::kbd::Kbd::format(keystroke).into()
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
            .child(self.keys);
        super::theme::apply_style(root, &self.style).into_any_element()
    }
}

impl Styled for Kbd {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}
