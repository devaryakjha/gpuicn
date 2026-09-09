//! The shadcn Nova Drawer visual port.
//!
//! Drag dismissal, snap points, nesting, and modal focus behavior stay in Base GPUI.

pub use base_gpui::drawer::{
    DrawerBackdrop, DrawerClose, DrawerContent, DrawerDescription, DrawerPopup, DrawerPortal,
    DrawerRoot, DrawerSwipeDirection, DrawerTitle, DrawerTrigger, DrawerViewport,
};
use gpui::{App, Div, ElementId, FontWeight, SharedString, Styled, div, px};

use super::{
    button::{ButtonSize, ButtonVariant, style_button},
    dialog::modal_focus::ModalFocus,
    theme::UiTheme,
};

/// Creates a Drawer root with a caller-owned stable ID.
pub fn drawer_root(id: impl Into<ElementId>) -> DrawerRoot<()> {
    DrawerRoot::new().id(id)
}

/// Creates an outline Drawer trigger.
pub fn drawer_trigger(id: impl Into<ElementId>, cx: &App) -> DrawerTrigger<()> {
    let theme = UiTheme::read(cx).clone();
    DrawerTrigger::new()
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

/// Creates the portal for Drawer layers.
pub fn drawer_portal() -> DrawerPortal<()> {
    DrawerPortal::new()
}

/// Creates the Nova modal backdrop.
pub fn drawer_backdrop(cx: &App) -> DrawerBackdrop<()> {
    DrawerBackdrop::new()
        .absolute()
        .inset_0()
        .bg(UiTheme::read(cx).colors.overlay)
}

/// Creates the full-window Drawer viewport.
pub fn drawer_viewport() -> DrawerViewport<()> {
    DrawerViewport::new()
        .absolute()
        .inset_0()
        .style_with_state(|state, base| match state.swipe_direction {
            DrawerSwipeDirection::Down => base.flex().flex_col().justify_end(),
            DrawerSwipeDirection::Up => base.flex().flex_col().justify_start(),
            DrawerSwipeDirection::Left => base.flex().justify_start(),
            DrawerSwipeDirection::Right => base.flex().justify_end(),
        })
}

/// Creates the side-aware Drawer surface. The root's swipe direction controls its edge.
pub fn drawer_popup(
    id: impl Into<ElementId>,
    aria_label: impl Into<SharedString>,
    cx: &App,
) -> DrawerPopup<()> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    let id = id.into();
    let focus = ModalFocus::new(id.clone());
    DrawerPopup::new()
        .id(id)
        .aria_label(aria_label)
        .child_any(focus.boundary(false))
        .child_any(focus.boundary(true))
        .style_with_state(move |state, base| {
            let base = focus.trap(
                base,
                state.modal_mode.traps_focus() && !state.nested_drawer_open,
            );
            let surface = base
                .overflow_hidden()
                .bg(theme.colors.popover)
                .text_color(theme.colors.popover_foreground)
                .font_family(theme.fonts.body.clone())
                .text_size(px(14.0) * text_scale);
            match state.swipe_direction {
                DrawerSwipeDirection::Down => surface
                    .w_full()
                    .max_h(spacing * 100_f32)
                    .rounded_t(theme.radius.xl)
                    .border_t_1(),
                DrawerSwipeDirection::Up => surface
                    .w_full()
                    .max_h(spacing * 100_f32)
                    .rounded_b(theme.radius.xl)
                    .border_b_1(),
                DrawerSwipeDirection::Left => surface
                    .h_full()
                    .w(spacing * 80_f32)
                    .rounded_r(theme.radius.xl)
                    .border_r_1(),
                DrawerSwipeDirection::Right => surface
                    .h_full()
                    .w(spacing * 80_f32)
                    .rounded_l(theme.radius.xl)
                    .border_l_1(),
            }
            .border_color(theme.colors.border)
        })
}

/// Creates the padded Drawer content container.
pub fn drawer_content(cx: &App) -> DrawerContent<()> {
    let theme = UiTheme::read(cx).clone();
    DrawerContent::new().style_with_state(move |_state, base| {
        base.flex()
            .flex_col()
            .overflow_hidden()
            .bg(theme.colors.popover)
            .text_color(theme.colors.popover_foreground)
    })
}

/// Creates Nova's non-interactive vertical swipe handle.
pub fn drawer_swipe_handle(cx: &App) -> Div {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    div()
        .mx_auto()
        .mt(spacing * 4_f32)
        .h(spacing * 1_f32)
        .w(spacing * 24_f32)
        .rounded_full()
        .bg(theme.colors.muted)
}

/// Creates a Drawer title.
pub fn drawer_title(id: impl Into<ElementId>, cx: &App) -> DrawerTitle<()> {
    let theme = UiTheme::read(cx).clone();
    let text_scale = theme.text_scale;
    DrawerTitle::new()
        .id(id)
        .font_family(theme.fonts.heading)
        .font_weight(FontWeight::MEDIUM)
        .text_size(px(16.0) * text_scale)
        .text_color(theme.colors.foreground)
}

/// Creates a Drawer description.
pub fn drawer_description(id: impl Into<ElementId>, cx: &App) -> DrawerDescription<()> {
    let theme = UiTheme::read(cx).clone();
    let text_scale = theme.text_scale;
    DrawerDescription::new()
        .id(id)
        .font_family(theme.fonts.body)
        .text_size(px(14.0) * text_scale)
        .text_color(theme.colors.muted_foreground)
}

/// Creates an outline Drawer close action.
pub fn drawer_close(id: impl Into<ElementId>, cx: &App) -> DrawerClose<()> {
    let theme = UiTheme::read(cx).clone();
    DrawerClose::new()
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
