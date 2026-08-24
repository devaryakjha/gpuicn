//! The shadcn Nova Toast visual port.
//!
//! Base GPUI owns queueing, auto-dismiss, pause/resume, stacking and swipe dismissal.

pub use base_gpui::toast::{
    ToastAction, ToastClose, ToastContent, ToastDescription, ToastOptions, ToastPortal,
    ToastProvider, ToastRoot, ToastTitle, ToastViewport, create_toast_manager,
};
use gpui::{App, BoxShadow, ElementId, FontWeight, Styled, px};
use gpui_icons::{LucideIcon, lucide};

use super::{
    button::{ButtonSize, ButtonVariant, style_button},
    theme::UiTheme,
};

/// Creates a Toast provider with a caller-owned stable ID.
pub fn toast_provider(id: impl Into<ElementId>) -> ToastProvider<()> {
    ToastProvider::new().id(id)
}

/// Creates the Toast portal.
pub fn toast_portal() -> ToastPortal<()> {
    ToastPortal::new()
}

/// Creates Nova's bottom-right Toast viewport.
pub fn toast_viewport(id: impl Into<ElementId>) -> ToastViewport<()> {
    ToastViewport::new()
        .id(id)
        .absolute()
        .right(px(16.0))
        .bottom(px(16.0))
        .w(px(384.0))
        .max_w(px(384.0))
        .flex()
        .flex_col()
        .gap(px(12.0))
}

/// Creates the stacked Nova Toast surface.
pub fn toast_root(cx: &App) -> ToastRoot<()> {
    let theme = UiTheme::read(cx).clone();
    ToastRoot::new().style_with_state(move |_state, base| {
        base.w_full()
            .rounded(px(16.0))
            .border_1()
            .border_color(theme.colors.border)
            .bg(theme.colors.popover)
            .text_color(theme.colors.popover_foreground)
            .font_family(theme.fonts.body.clone())
            .shadow(vec![
                BoxShadow::new(px(0.0), px(8.0), theme.colors.foreground.alpha(0.16).into())
                    .blur_radius(px(16.0)),
            ])
    })
}

/// Creates the padded Toast content row.
pub fn toast_content(cx: &App) -> ToastContent<()> {
    let theme = UiTheme::read(cx).clone();
    ToastContent::new().style_with_state(move |_state, base| {
        base.flex()
            .items_center()
            .gap(px(12.0))
            .overflow_hidden()
            .p(px(16.0))
            .font_family(theme.fonts.body.clone())
    })
}

/// Creates a medium-weight Toast title.
pub fn toast_title(cx: &App) -> ToastTitle<()> {
    let theme = UiTheme::read(cx).clone();
    ToastTitle::new().style_with_state(move |_state, base| {
        base.font_family(theme.fonts.body.clone())
            .font_weight(FontWeight::MEDIUM)
            .text_size(px(14.0))
            .text_color(theme.colors.popover_foreground)
    })
}

/// Creates a muted Toast description.
pub fn toast_description(cx: &App) -> ToastDescription<()> {
    let theme = UiTheme::read(cx).clone();
    ToastDescription::new().style_with_state(move |_state, base| {
        base.font_family(theme.fonts.body.clone())
            .text_size(px(14.0))
            .text_color(theme.colors.muted_foreground)
    })
}

/// Creates the compact outline Toast action.
pub fn toast_action(cx: &App) -> ToastAction<()> {
    let theme = UiTheme::read(cx).clone();
    ToastAction::new().style_with_state(move |_state, base| {
        style_button(base, false, ButtonVariant::Outline, ButtonSize::Sm, &theme)
    })
}

/// Creates the compact icon-only Toast close button.
pub fn toast_close(cx: &App) -> ToastClose<()> {
    let theme = UiTheme::read(cx).clone();
    let icon_color = theme.colors.muted_foreground;
    ToastClose::new()
        .aria_label("Close toast")
        .style_with_state(move |_state, base| {
            style_button(
                base,
                false,
                ButtonVariant::Ghost,
                ButtonSize::IconSm,
                &theme,
            )
        })
        .child_any(lucide(LucideIcon::X).size(px(16.0)).text_color(icon_color))
}
