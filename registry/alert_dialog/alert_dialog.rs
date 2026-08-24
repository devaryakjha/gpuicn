//! The shadcn Nova Alert Dialog visual port.
//!
//! Interaction and focus handling remain in the pinned Base GPUI Alert Dialog.

pub use base_gpui::alert_dialog::{
    AlertDialogBackdrop, AlertDialogClose, AlertDialogDescription, AlertDialogPopup,
    AlertDialogPortal, AlertDialogRoot, AlertDialogTitle, AlertDialogTrigger, AlertDialogViewport,
};
use gpui::{App, BoxShadow, Div, ElementId, FontWeight, SharedString, Styled, black, div, px};

use super::{
    button::{ButtonSize, ButtonVariant, style_button},
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
pub fn alert_dialog_backdrop() -> AlertDialogBackdrop<()> {
    AlertDialogBackdrop::new()
        .absolute()
        .inset_0()
        .bg(black().alpha(0.10))
}

/// Creates the centered, guttered dialog viewport.
pub fn alert_dialog_viewport() -> AlertDialogViewport<()> {
    AlertDialogViewport::new()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .p(px(16.0))
}

/// Creates Nova's stacked Alert Dialog header.
pub fn alert_dialog_header() -> Div {
    div().flex().flex_col().gap(px(6.0))
}

/// Creates Nova's inset Alert Dialog footer surface.
pub fn alert_dialog_footer(cx: &App) -> Div {
    let theme = UiTheme::read(cx);
    div()
        .mx(px(-16.0))
        .mb(px(-16.0))
        .flex()
        .justify_end()
        .gap(px(8.0))
        .rounded_b(theme.radius.base * 1.2)
        .border_t_1()
        .border_color(theme.colors.border)
        .bg(theme.colors.muted.alpha(0.50))
        .p(px(16.0))
}

/// Creates the Nova dialog surface. Callers choose the root's controlled/open state.
pub fn alert_dialog_popup(
    id: impl Into<ElementId>,
    aria_label: impl Into<SharedString>,
    cx: &App,
) -> AlertDialogPopup<()> {
    let theme = UiTheme::read(cx).clone();
    AlertDialogPopup::new()
        .id(id)
        .aria_label(aria_label)
        .style_with_state(move |_state, base| {
            base.w_full()
                .max_w(px(384.0))
                .flex()
                .flex_col()
                .gap(px(16.0))
                .rounded(px(f32::from(theme.radius.base) * 1.2))
                .p(px(16.0))
                .bg(theme.colors.popover)
                .text_color(theme.colors.popover_foreground)
                .font_family(theme.fonts.body.clone())
                .text_size(px(14.0))
                .shadow(vec![
                    BoxShadow::new(px(0.0), px(0.0), theme.colors.foreground.alpha(0.10).into())
                        .spread_radius(px(1.0)),
                ])
        })
}

/// Creates a medium-weight Alert Dialog title.
pub fn alert_dialog_title(id: impl Into<ElementId>, cx: &App) -> AlertDialogTitle<()> {
    let theme = UiTheme::read(cx).clone();
    AlertDialogTitle::new()
        .id(id)
        .font_family(theme.fonts.heading)
        .font_weight(FontWeight::MEDIUM)
        .text_size(px(16.0))
        .text_color(theme.colors.popover_foreground)
}

/// Creates a muted Alert Dialog description.
pub fn alert_dialog_description(id: impl Into<ElementId>, cx: &App) -> AlertDialogDescription<()> {
    let theme = UiTheme::read(cx).clone();
    AlertDialogDescription::new()
        .id(id)
        .font_family(theme.fonts.body)
        .text_size(px(14.0))
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
