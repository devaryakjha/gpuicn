#![allow(missing_docs)]
//! The shadcn Nova Toolbar visual port.
//!
//! Base UI does not ship a shadcn Toolbar component. This ports Base GPUI's
//! Toolbar behavior into the same Neutral controls used by shadcn Nova.

#[path = "toolbar_children.rs"]
mod toolbar_children;
#[path = "toolbar_group.rs"]
mod toolbar_group;
#[path = "toolbar_input.rs"]
mod toolbar_input;
#[path = "toolbar_root.rs"]
mod toolbar_root;
#[path = "toolbar_wiring.rs"]
mod toolbar_wiring;

use base_gpui::toolbar::{
    TOOLBAR_KEY_CONTEXT, ToolbarContext, ToolbarFocusDown, ToolbarFocusLeft, ToolbarFocusRight,
    ToolbarFocusUp, ToolbarGroupStyleState, ToolbarInputStyleState, ToolbarItemMetadata,
    ToolbarMove, ToolbarOrientation, ToolbarProps, ToolbarRootStyleState,
};
pub use base_gpui::toolbar::{ToolbarButton, ToolbarLink, ToolbarSeparator};
use gpui::{
    App, FontWeight, InteractiveElement as _, SharedString, Styled, prelude::FluentBuilder as _, px,
};
pub use toolbar_children::{ToolbarChild, ToolbarGroupChild};
pub use toolbar_group::ToolbarGroup;
pub use toolbar_input::ToolbarInput;
pub use toolbar_root::ToolbarRoot;

use super::theme::{UiTheme, input_text_layout};

/// Creates a compact toolbar root. Give it an accessible label for icon-only controls.
pub fn toolbar(cx: &App) -> ToolbarRoot {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    ToolbarRoot::new()
        .flex()
        .items_center()
        .gap(spacing * 1_f32)
        .rounded(theme.radius.lg)
        .border_1()
        .border_color(theme.colors.border)
        .p(spacing * 1_f32)
        .bg(theme.colors.background)
        .font_family(theme.fonts.body)
}

/// Creates a grouped toolbar section.
pub fn toolbar_group(cx: &App) -> ToolbarGroup {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    ToolbarGroup::new()
        .flex()
        .items_center()
        .gap(spacing * 0.5_f32)
        .font_family(theme.fonts.body)
}

/// Creates a Nova icon or text toolbar button. Add its content as children.
pub fn toolbar_button(cx: &App) -> ToolbarButton {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    let focus_ring = theme.focus_ring();
    ToolbarButton::new().style_with_state(move |state, base| {
        let colors = theme.colors;
        let focus_ring = focus_ring.clone();
        base.flex()
            .items_center()
            .justify_center()
            .h(spacing * 7_f32)
            .rounded(theme.radius.sm)
            .border_1()
            .border_color(colors.background.opacity(0.0))
            .px(spacing * 2_f32)
            .font_family(theme.fonts.body.clone())
            .font_weight(FontWeight::MEDIUM)
            .text_size(px(14.0) * text_scale)
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
                    .shadow(focus_ring.clone())
            })
    })
}

/// Creates a Nova toolbar link.
pub fn toolbar_link(cx: &App) -> ToolbarLink {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    let focus_ring = theme.focus_ring();
    ToolbarLink::new().style_with_state(move |_state, base| {
        let colors = theme.colors;
        let focus_ring = focus_ring.clone();
        base.flex()
            .items_center()
            .justify_center()
            .h(spacing * 7_f32)
            .rounded(theme.radius.sm)
            .border_1()
            .border_color(colors.background.opacity(0.0))
            .px(spacing * 2_f32)
            .font_family(theme.fonts.body.clone())
            .font_weight(FontWeight::MEDIUM)
            .text_size(px(14.0) * text_scale)
            .text_color(colors.foreground)
            .cursor_pointer()
            .hover(move |style| style.bg(colors.muted))
            .focus_visible(move |style| {
                style
                    .bg(colors.background)
                    .border_color(colors.ring)
                    .shadow(focus_ring.clone())
            })
    })
}

/// Creates a toolbar text input using the shared native editor.
pub fn toolbar_input(cx: &App) -> ToolbarInput {
    toolbar_input_with_label("Toolbar input", cx)
}

/// Creates a toolbar input with an accessible name for its purpose.
pub fn toolbar_input_with_label(label: impl Into<SharedString>, cx: &App) -> ToolbarInput {
    let label = label.into();
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    ToolbarInput::new()
        .aria_label(label.clone())
        .style_with_state(move |state, base| {
            let ring = if state.input.invalid {
                theme.destructive_focus_ring()
            } else {
                theme.focus_ring()
            };
            input_text_layout(base, text_scale)
                .h(spacing * 7_f32)
                .px(spacing * 2_f32)
                .rounded(theme.radius.sm)
                .border_1()
                .border_color(if state.input.invalid {
                    theme.colors.destructive
                } else if state.input.focused {
                    theme.colors.ring
                } else {
                    theme.colors.input
                })
                .bg(theme.colors.background)
                .font_family(theme.fonts.body.clone())
                .text_size(px(14.0) * text_scale)
                .text_color(theme.colors.foreground)
                .when(state.input.focused, |base| base.shadow(ring))
                .when(state.disabled, |base| {
                    base.opacity(0.50).cursor_not_allowed()
                })
        })
}

/// Creates the neutral separator between toolbar groups.
pub fn toolbar_separator(cx: &App) -> ToolbarSeparator {
    let theme = UiTheme::read(cx).clone();
    ToolbarSeparator::new().style_with_state(move |_state, base| base.bg(theme.colors.border))
}

#[cfg(test)]
#[path = "toolbar_tests.rs"]
mod tests;
