//! The shadcn Nova Alert Dialog visual port.
//!
//! Interaction and focus handling remain in the pinned Base GPUI Alert Dialog.

pub use base_gpui::alert_dialog::{
    AlertDialogBackdrop, AlertDialogClose, AlertDialogDescription, AlertDialogPopup,
    AlertDialogPortal, AlertDialogRoot, AlertDialogTitle, AlertDialogTrigger, AlertDialogViewport,
};
use gpui::{App, Div, ElementId, FontWeight, SharedString, Styled, div, px};

use super::{
    button::{ButtonSize, ButtonVariant, style_button},
    dialog::modal_focus::ModalFocus,
    theme::UiTheme,
};

/// Creates an Alert Dialog root with a caller-owned stable ID.
pub fn alert_dialog_root(id: impl Into<ElementId>) -> AlertDialogRoot<()> {
    AlertDialogRoot::new().id(id)
}

/// Creates an outline Alert Dialog trigger.
pub fn alert_dialog_trigger(id: impl Into<ElementId>, cx: &App) -> AlertDialogTrigger<()> {
    let theme = UiTheme::read(cx).clone();
    AlertDialogTrigger::new()
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

/// Creates the portal for the dialog layers.
pub fn alert_dialog_portal() -> AlertDialogPortal<()> {
    AlertDialogPortal::new()
}

/// Creates the dismissible Nova backdrop.
pub fn alert_dialog_backdrop(cx: &App) -> AlertDialogBackdrop<()> {
    AlertDialogBackdrop::new()
        .absolute()
        .inset_0()
        .bg(UiTheme::read(cx).colors.overlay)
}

/// Creates the centered, guttered dialog viewport.
pub fn alert_dialog_viewport(cx: &App) -> AlertDialogViewport<()> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    AlertDialogViewport::new()
        .absolute()
        .inset_0()
        .flex()
        .flex_col()
        .items_stretch()
        .justify_center()
        .p(spacing * 4_f32)
}

/// Creates Nova's stacked Alert Dialog header.
pub fn alert_dialog_header(cx: &App) -> Div {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    div().flex().flex_col().gap(spacing * 1.5_f32)
}

/// Creates Nova's inset Alert Dialog footer surface.
pub fn alert_dialog_footer(cx: &App) -> Div {
    let theme = UiTheme::read(cx);
    let spacing = theme.spacing.unit;
    div()
        .mx(spacing * -4_f32)
        .mb(spacing * -4_f32)
        .flex()
        .justify_end()
        .gap(spacing * 2_f32)
        .rounded_b(theme.radius.xl)
        .border_t_1()
        .border_color(theme.colors.border)
        .bg(theme.colors.muted.opacity(0.50))
        .p(spacing * 4_f32)
}

/// Creates the Nova dialog surface. Callers choose the root's controlled/open state.
pub fn alert_dialog_popup(
    id: impl Into<ElementId>,
    aria_label: impl Into<SharedString>,
    cx: &App,
) -> AlertDialogPopup<()> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    let id = id.into();
    let focus = ModalFocus::new(id.clone());
    AlertDialogPopup::new()
        .id(id)
        .aria_label(aria_label)
        .child_any(focus.boundary(false))
        .child_any(focus.boundary(true))
        .style_with_state(move |state, base| {
            let base = focus.trap(
                base,
                state.modal_mode.traps_focus() && !state.nested_dialog_open,
            );
            base.w_full()
                .min_w(px(0.))
                .mx_auto()
                .max_w(spacing * 96_f32)
                .max_h(spacing * 100_f32)
                .overflow_hidden()
                .flex()
                .flex_col()
                .gap(spacing * 4_f32)
                .rounded(theme.radius.xl)
                .border_1()
                .border_color(theme.colors.foreground.opacity(0.10))
                .p(spacing * 4_f32)
                .bg(theme.colors.popover)
                .text_color(theme.colors.popover_foreground)
                .font_family(theme.fonts.body.clone())
                .text_size(px(14.0) * text_scale)
        })
}

/// Creates a medium-weight Alert Dialog title.
pub fn alert_dialog_title(id: impl Into<ElementId>, cx: &App) -> AlertDialogTitle<()> {
    let theme = UiTheme::read(cx).clone();
    let text_scale = theme.text_scale;
    AlertDialogTitle::new()
        .id(id)
        .font_family(theme.fonts.heading)
        .font_weight(FontWeight::MEDIUM)
        .text_size(px(16.0) * text_scale)
        .text_color(theme.colors.popover_foreground)
}

/// Creates a muted Alert Dialog description.
pub fn alert_dialog_description(id: impl Into<ElementId>, cx: &App) -> AlertDialogDescription<()> {
    let theme = UiTheme::read(cx).clone();
    let text_scale = theme.text_scale;
    AlertDialogDescription::new()
        .id(id)
        .font_family(theme.fonts.body)
        .text_size(px(14.0) * text_scale)
        .text_color(theme.colors.muted_foreground)
}

/// Creates a primary close action. The Base GPUI Close keeps dismissal and focus return.
pub fn alert_dialog_action(id: impl Into<ElementId>, cx: &App) -> AlertDialogClose<()> {
    let theme = UiTheme::read(cx).clone();
    AlertDialogClose::new()
        .id(id)
        .style_with_state(move |state, base| {
            style_button(
                base,
                state.disabled,
                ButtonVariant::Default,
                ButtonSize::Default,
                &theme,
            )
        })
}

/// Creates an outline cancel action.
pub fn alert_dialog_cancel(id: impl Into<ElementId>, cx: &App) -> AlertDialogClose<()> {
    let theme = UiTheme::read(cx).clone();
    AlertDialogClose::new()
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
