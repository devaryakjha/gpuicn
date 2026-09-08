//! The shadcn Nova Popover visual port.
//!
//! Base GPUI owns collision handling, outside dismissal, focus and controlled state.

pub use base_gpui::popover::{
    PopoverAlign, PopoverArrow, PopoverBackdrop, PopoverClose, PopoverDescription, PopoverPopup,
    PopoverPortal, PopoverPositioner, PopoverRoot, PopoverSide, PopoverTitle, PopoverTrigger,
    PopoverViewport,
};
use gpui::{App, ElementId, FontWeight, SharedString, Styled, px};

use super::{
    button::{ButtonSize, ButtonVariant, style_button},
    theme::UiTheme,
};

/// Creates a Popover root with a caller-owned stable ID.
pub fn popover_root(id: impl Into<ElementId>) -> PopoverRoot<()> {
    PopoverRoot::new().id(id)
}

/// Creates an outline Popover trigger.
pub fn popover_trigger(id: impl Into<ElementId>, cx: &App) -> PopoverTrigger<()> {
    let theme = UiTheme::read(cx).clone();
    PopoverTrigger::new()
        .id(id)
        .style_with_state(move |state, base| {
            style_button(
                base,
                state.disabled,
                ButtonVariant::Outline,
                ButtonSize::Default,
                &theme,
            )
        })
}

/// Creates the Popover portal.
pub fn popover_portal() -> PopoverPortal<()> {
    PopoverPortal::new()
}

/// Creates an anchored Popover positioner with Nova's 4px side offset.
pub fn popover_positioner(cx: &App) -> PopoverPositioner<()> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    PopoverPositioner::new().side_offset(spacing * 1_f32)
}

/// Creates the Nova Popover surface.
pub fn popover_popup(
    id: impl Into<ElementId>,
    aria_label: impl Into<SharedString>,
    cx: &App,
) -> PopoverPopup<()> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    PopoverPopup::new()
        .id(id)
        .aria_label(aria_label)
        .style_with_state(move |_state, base| {
            base.w(spacing * 72_f32)
                .flex()
                .flex_col()
                .gap(spacing * 2.5_f32)
                .rounded(theme.radius.lg)
                .border_1()
                .border_color(theme.colors.foreground.opacity(0.10))
                .p(spacing * 2.5_f32)
                .bg(theme.colors.popover)
                .text_color(theme.colors.popover_foreground)
                .font_family(theme.fonts.body.clone())
                .text_size(px(14.0) * text_scale)
                .shadow(theme.shadows.md.clone())
        })
}

/// Creates the optional modal Popover backdrop.
pub fn popover_backdrop(cx: &App) -> PopoverBackdrop<()> {
    PopoverBackdrop::new()
        .absolute()
        .inset_0()
        .bg(UiTheme::read(cx).colors.overlay)
}

/// Creates the Popover arrow surface.
pub fn popover_arrow(cx: &App) -> PopoverArrow<()> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    PopoverArrow::new()
        .size(spacing * 2.5_f32)
        .bg(theme.colors.popover)
}

/// Creates a medium-weight Popover title.
pub fn popover_title(cx: &App) -> PopoverTitle<()> {
    let theme = UiTheme::read(cx).clone();
    let text_scale = theme.text_scale;
    PopoverTitle::new()
        .font_family(theme.fonts.body)
        .font_weight(FontWeight::MEDIUM)
        .text_size(px(14.0) * text_scale)
        .text_color(theme.colors.popover_foreground)
}

/// Creates a muted Popover description.
pub fn popover_description(cx: &App) -> PopoverDescription<()> {
    let theme = UiTheme::read(cx).clone();
    let text_scale = theme.text_scale;
    PopoverDescription::new()
        .font_family(theme.fonts.body)
        .text_size(px(14.0) * text_scale)
        .text_color(theme.colors.muted_foreground)
}

/// Creates an outline Popover close action.
pub fn popover_close(id: impl Into<ElementId>, cx: &App) -> PopoverClose<()> {
    let theme = UiTheme::read(cx).clone();
    PopoverClose::new()
        .id(id)
        .style_with_state(move |state, base| {
            style_button(
                base,
                state.disabled,
                ButtonVariant::Outline,
                ButtonSize::Sm,
                &theme,
            )
        })
}
