//! Nova field groups and headings. Pass disabled state to their controls explicitly.
use super::theme::UiTheme;
use gpui_kit::{
    App, Div, ElementId, FontWeight, InteractiveElement as _, Role, Stateful,
    StatefulInteractiveElement as _, Styled, div,
};

/// The two visual treatments for a group legend.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FieldsetLegendVariant {
    /// A section heading.
    #[default]
    Legend,
    /// A compact field label.
    Label,
}

/// Creates a semantic group; give it an accessible label matching its legend.
pub fn fieldset_root(id: impl Into<ElementId>, cx: &App) -> Stateful<Div> {
    let t = UiTheme::read(cx);
    div()
        .id(id)
        .role(Role::Group)
        .flex()
        .flex_col()
        .w_full()
        .gap(t.space(6.))
        .font_family(t.fonts.body.clone())
        .text_color(t.colors.foreground)
}
/// Creates the group heading.
pub fn fieldset_legend(variant: FieldsetLegendVariant, cx: &App) -> Div {
    let t = UiTheme::read(cx);
    div()
        .font_family(t.fonts.body.clone())
        .font_weight(FontWeight::MEDIUM)
        .text_size(t.text(if variant == FieldsetLegendVariant::Legend {
            16.
        } else {
            14.
        }))
        .line_height(t.text(20.))
        .text_color(t.colors.foreground)
}
