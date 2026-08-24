//! The shadcn Nova Popover visual port.
//!
//! Base GPUI owns collision handling, outside dismissal, focus and controlled state.

pub use base_gpui::popover::{
    PopoverAlign, PopoverArrow, PopoverBackdrop, PopoverClose, PopoverDescription, PopoverPopup,
    PopoverPortal, PopoverPositioner, PopoverRoot, PopoverSide, PopoverTitle, PopoverTrigger,
    PopoverViewport,
};
use gpui::{App, BoxShadow, ElementId, FontWeight, SharedString, Styled, black, px};

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
pub fn popover_positioner() -> PopoverPositioner<()> {
    PopoverPositioner::new().side_offset(px(4.0))
}

/// Creates the Nova Popover surface.
pub fn popover_popup(
    id: impl Into<ElementId>,
    aria_label: impl Into<SharedString>,
    cx: &App,
) -> PopoverPopup<()> {
    let theme = UiTheme::read(cx).clone();
    PopoverPopup::new()
        .id(id)
        .aria_label(aria_label)
        .style_with_state(move |_state, base| {
            base.w(px(288.0))
                .flex()
                .flex_col()
                .gap(px(10.0))
                .rounded(px(8.0))
                .border_1()
                .border_color(theme.colors.foreground.alpha(0.10))
                .p(px(10.0))
                .bg(theme.colors.popover)
                .text_color(theme.colors.popover_foreground)
                .font_family(theme.fonts.body.clone())
                .text_size(px(14.0))
                .shadow(vec![
                    BoxShadow::new(px(0.0), px(4.0), theme.colors.foreground.alpha(0.12).into())
                        .blur_radius(px(8.0)),
                ])
        })
}

/// Creates the optional modal Popover backdrop.
pub fn popover_backdrop() -> PopoverBackdrop<()> {
    PopoverBackdrop::new()
        .absolute()
        .inset_0()
        .bg(black().alpha(0.10))
}

/// Creates the Popover arrow surface.
pub fn popover_arrow(cx: &App) -> PopoverArrow<()> {
    let theme = UiTheme::read(cx).clone();
    PopoverArrow::new().size(px(10.0)).bg(theme.colors.popover)
}

/// Creates a medium-weight Popover title.
pub fn popover_title(cx: &App) -> PopoverTitle<()> {
    let theme = UiTheme::read(cx).clone();
    PopoverTitle::new()
        .font_family(theme.fonts.body)
        .font_weight(FontWeight::MEDIUM)
        .text_size(px(14.0))
        .text_color(theme.colors.popover_foreground)
}

/// Creates a muted Popover description.
pub fn popover_description(cx: &App) -> PopoverDescription<()> {
    let theme = UiTheme::read(cx).clone();
    PopoverDescription::new()
        .font_family(theme.fonts.body)
        .text_size(px(14.0))
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
