//! The shadcn Nova Toolbar visual port.
//!
//! Base UI does not ship a shadcn Toolbar component. This ports Base GPUI's
//! Toolbar behavior into the same Neutral controls used by shadcn Nova.

pub use base_gpui::toolbar::{
    ToolbarButton, ToolbarGroup, ToolbarInput, ToolbarLink, ToolbarRoot, ToolbarSeparator,
};
use gpui::{
    App, BoxShadow, FontWeight, InteractiveElement as _, Styled, prelude::FluentBuilder as _, px,
};

use super::theme::UiTheme;

/// Creates a compact toolbar root. Give it an accessible label for icon-only controls.
pub fn toolbar(cx: &App) -> ToolbarRoot {
    let theme = UiTheme::read(cx).clone();
    ToolbarRoot::new()
        .flex()
        .items_center()
        .gap(px(4.0))
        .rounded(theme.radius.base)
        .border_1()
        .border_color(theme.colors.border)
        .p(px(4.0))
        .bg(theme.colors.background)
        .font_family(theme.fonts.body)
}

/// Creates a grouped toolbar section.
pub fn toolbar_group(cx: &App) -> ToolbarGroup {
    let theme = UiTheme::read(cx).clone();
    ToolbarGroup::new()
        .flex()
        .items_center()
        .gap(px(2.0))
        .font_family(theme.fonts.body)
}

/// Creates a Nova icon or text toolbar button. Add its content as children.
pub fn toolbar_button(cx: &App) -> ToolbarButton {
    let theme = UiTheme::read(cx).clone();
    ToolbarButton::new().style_with_state(move |state, base| {
        let colors = theme.colors;
        base.flex()
            .items_center()
            .justify_center()
            .h(px(28.0))
            .rounded(px(6.0))
            .border_1()
            .border_color(colors.background.alpha(0.0))
            .px(px(8.0))
            .font_family(theme.fonts.body.clone())
            .font_weight(FontWeight::MEDIUM)
            .text_size(px(14.0))
            .text_color(colors.foreground)
            .when(!state.disabled, |base| {
                base.cursor_pointer()
                    .hover(move |style| style.bg(colors.muted))
            })
            .when(state.disabled, |base| {
                base.opacity(0.50).cursor_not_allowed()
            })
            .focus_visible(move |style| {
                style
                    .bg(colors.background)
                    .border_color(colors.ring)
                    .shadow(vec![
                        BoxShadow::new(px(0.0), px(0.0), colors.ring.alpha(0.50).into())
                            .spread_radius(px(3.0)),
                    ])
            })
    })
}

/// Creates a Nova toolbar link.
pub fn toolbar_link(cx: &App) -> ToolbarLink {
    let theme = UiTheme::read(cx).clone();
    ToolbarLink::new().style_with_state(move |_state, base| {
        let colors = theme.colors;
        base.flex()
            .items_center()
            .justify_center()
            .h(px(28.0))
            .rounded(px(6.0))
            .border_1()
            .border_color(colors.background.alpha(0.0))
            .px(px(8.0))
            .font_family(theme.fonts.body.clone())
            .font_weight(FontWeight::MEDIUM)
            .text_size(px(14.0))
            .text_color(colors.foreground)
            .cursor_pointer()
            .hover(move |style| style.bg(colors.muted))
            .focus_visible(move |style| {
                style
                    .bg(colors.background)
                    .border_color(colors.ring)
                    .shadow(vec![
                        BoxShadow::new(px(0.0), px(0.0), colors.ring.alpha(0.50).into())
                            .spread_radius(px(3.0)),
                    ])
            })
    })
}

/// Creates a toolbar text input backed by Base GPUI's Input component.
pub fn toolbar_input(cx: &App) -> ToolbarInput {
    let theme = UiTheme::read(cx).clone();
    ToolbarInput::new().style_with_state(move |_state, base| {
        base.h(px(28.0))
            .rounded(px(6.0))
            .border_1()
            .border_color(theme.colors.input)
            .bg(theme.colors.background)
            .font_family(theme.fonts.body.clone())
            .text_size(px(14.0))
            .text_color(theme.colors.foreground)
    })
}

/// Creates the neutral separator between toolbar groups.
pub fn toolbar_separator(cx: &App) -> ToolbarSeparator {
    let theme = UiTheme::read(cx).clone();
    ToolbarSeparator::new().style_with_state(move |_state, base| base.bg(theme.colors.border))
}
