//! Nova tooltip presentation at GPUI's native tooltip attachment point.
use super::theme::UiTheme;
use gpui_kit::{App, Styled, px};

/// Creates a tooltip view for GPUI's native `.tooltip(...)` attachment point.
/// Use this on an existing interactive control to avoid nesting another button
/// and tab stop. The host control must retain its own full accessible name.
pub fn text_tooltip(label: gpui_kit::SharedString, cx: &mut App) -> gpui_kit::AnyView {
    use gpui_kit::AppContext as _;
    cx.new(|_| TextTooltip(label)).into()
}
struct TextTooltip(gpui_kit::SharedString);
impl gpui_kit::Render for TextTooltip {
    fn render(
        &mut self,
        _: &mut gpui_kit::Window,
        cx: &mut gpui_kit::Context<Self>,
    ) -> impl gpui_kit::IntoElement {
        use gpui_kit::ParentElement as _;
        let theme = UiTheme::read(cx);
        let spacing = theme.spacing.unit;
        let text_scale = theme.text_scale;
        gpui_kit::div()
            .rounded(theme.radius.sm)
            .px(spacing * 3_f32)
            .py(spacing * 1.5_f32)
            .bg(theme.colors.foreground)
            .text_color(theme.colors.background)
            .font_family(theme.fonts.body.clone())
            .text_size(px(12.) * text_scale)
            .child(self.0.clone())
    }
}
