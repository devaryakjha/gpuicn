//! The shadcn Nova Toast visual port.
//!
//! Base GPUI owns queueing, auto-dismiss, pause/resume, stacking and swipe dismissal.

pub use base_gpui::toast::{
    ToastAction, ToastClose, ToastContent, ToastDescription, ToastOptions, ToastPortal,
    ToastProvider, ToastRoot, ToastTitle, ToastViewport, create_toast_manager,
};
use gpui::{App, ElementId, FontWeight, Styled, px};
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

/// Creates a Toast viewport with the standard title, description, and close button.
/// Mount it once inside a provider. Override `content_builder` for custom content.
pub fn toast_viewport(id: impl Into<ElementId>, cx: &App) -> ToastViewport<()> {
    let theme = UiTheme::read(cx).clone();
    ToastViewport::new()
        .id(id)
        .absolute()
        .left(px(16.0))
        .right(px(16.0))
        .bottom(px(16.0))
        .max_w(px(384.0))
        .flex()
        .flex_col()
        .gap(px(12.0))
        .content_builder(move |_| {
            root_from_theme(&theme).child(
                content_from_theme(&theme)
                    .child(title_from_theme(&theme))
                    .child(description_from_theme(&theme))
                    .child(close_from_theme(&theme)),
            )
        })
}

/// Creates the stacked Nova Toast surface.
pub fn toast_root(cx: &App) -> ToastRoot<()> {
    root_from_theme(UiTheme::read(cx))
}

fn root_from_theme(theme: &UiTheme) -> ToastRoot<()> {
    let theme = theme.clone();
    ToastRoot::new().style_with_state(move |_state, base| {
        base.w_full()
            .rounded(theme.radius.two_xl)
            .border_1()
            .border_color(theme.colors.border)
            .bg(theme.colors.popover)
            .text_color(theme.colors.popover_foreground)
            .font_family(theme.fonts.body.clone())
    })
}

/// Creates the padded Toast content row.
pub fn toast_content(cx: &App) -> ToastContent<()> {
    content_from_theme(UiTheme::read(cx))
}

fn content_from_theme(theme: &UiTheme) -> ToastContent<()> {
    let theme = theme.clone();
    ToastContent::new().style_with_state(move |_state, base| {
        base.flex()
            .relative()
            .flex_col()
            .items_start()
            .gap(px(4.0))
            .overflow_hidden()
            .p(px(16.0))
            .pr(px(48.0))
            .font_family(theme.fonts.body.clone())
    })
}

/// Creates a medium-weight Toast title.
pub fn toast_title(cx: &App) -> ToastTitle<()> {
    title_from_theme(UiTheme::read(cx))
}

fn title_from_theme(theme: &UiTheme) -> ToastTitle<()> {
    let theme = theme.clone();
    ToastTitle::new().style_with_state(move |_state, base| {
        base.font_family(theme.fonts.body.clone())
            .font_weight(FontWeight::MEDIUM)
            .text_size(px(14.0))
            .text_color(theme.colors.popover_foreground)
    })
}

/// Creates a muted Toast description.
pub fn toast_description(cx: &App) -> ToastDescription<()> {
    description_from_theme(UiTheme::read(cx))
}

fn description_from_theme(theme: &UiTheme) -> ToastDescription<()> {
    let theme = theme.clone();
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
    close_from_theme(UiTheme::read(cx))
}

fn close_from_theme(theme: &UiTheme) -> ToastClose<()> {
    let theme = theme.clone();
    let icon_color = theme.colors.muted_foreground;
    ToastClose::new()
        .aria_label("Close toast")
        .absolute()
        .top(px(8.0))
        .right(px(8.0))
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
