//! The narrow shadcn Nova Dialog visual port.
//!
//! Visual source: shadcn/ui 4.19.0 `dialog.tsx` and `style-nova.css` at
//! `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`. Interaction, dismissal, and the
//! Popup/Close focus cycle come from the pinned Base GPUI Dialog primitives.

pub use base_gpui::dialog::{
    DialogBackdrop, DialogClose, DialogDescription, DialogPopup, DialogPortal, DialogRoot,
    DialogTitle, DialogTrigger, DialogViewport,
};
use gpui::{
    App, BoxShadow, Div, ElementId, FontWeight, ParentElement as _, SharedString, Styled, black,
    div, px,
};
use gpui_icons::{LucideIcon, lucide};

use super::{
    button::{ButtonSize, ButtonVariant, style_button},
    theme::UiTheme,
};

/// Creates the Dialog state root with a caller-owned stable ID.
pub fn dialog_root(id: impl Into<ElementId>) -> DialogRoot<()> {
    DialogRoot::new().id(id)
}

/// Creates a styled Dialog trigger with a caller-owned stable ID.
pub fn dialog_trigger(id: impl Into<ElementId>, cx: &App) -> DialogTrigger<()> {
    let theme = UiTheme::read(cx).clone();
    DialogTrigger::new()
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

/// Creates the in-canvas Dialog portal.
pub fn dialog_portal() -> DialogPortal<()> {
    DialogPortal::new()
}

/// Creates the dismissible, full-window Dialog backdrop.
pub fn dialog_backdrop() -> DialogBackdrop<()> {
    DialogBackdrop::new()
        .absolute()
        .inset_0()
        .bg(black().alpha(0.10))
}

/// Creates the centered Dialog viewport with the pinned 16px page gutter.
pub fn dialog_viewport() -> DialogViewport<()> {
    DialogViewport::new()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .p(px(16.0))
}

/// Creates Nova's stacked Dialog header.
pub fn dialog_header() -> Div {
    div().flex().flex_col().gap(px(8.0))
}

/// Creates Nova's inset Dialog footer surface.
pub fn dialog_footer(cx: &App) -> Div {
    let theme = UiTheme::read(cx);
    div()
        .mx(px(-16.0))
        .mb(px(-16.0))
        .flex()
        .justify_end()
        .gap(px(8.0))
        .rounded_b(theme.radius.base * 1.4)
        .border_t_1()
        .border_color(theme.colors.border)
        .bg(theme.colors.muted.alpha(0.50))
        .p(px(16.0))
}

/// Creates a styled Dialog popup with a caller-owned stable ID and name.
pub fn dialog_popup(
    id: impl Into<ElementId>,
    aria_label: impl Into<SharedString>,
    cx: &App,
) -> DialogPopup<()> {
    let theme = UiTheme::read(cx).clone();
    DialogPopup::new()
        .id(id)
        .aria_label(aria_label)
        .style_with_state(move |_state, base| {
            base.w_full()
                .max_w(px(384.0))
                .flex()
                .flex_col()
                .gap(px(16.0))
                .rounded(theme.radius.base * 1.4)
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

/// Creates a styled Dialog title with a caller-owned stable ID.
pub fn dialog_title(id: impl Into<ElementId>, cx: &App) -> DialogTitle<()> {
    let theme = UiTheme::read(cx).clone();
    DialogTitle::new()
        .id(id)
        .font_family(theme.fonts.heading)
        .font_weight(FontWeight::MEDIUM)
        .text_size(px(16.0))
        .line_height(px(16.0))
        .text_color(theme.colors.popover_foreground)
}

/// Creates a styled Dialog description with a caller-owned stable ID.
pub fn dialog_description(id: impl Into<ElementId>, cx: &App) -> DialogDescription<()> {
    let theme = UiTheme::read(cx).clone();
    DialogDescription::new()
        .id(id)
        .font_family(theme.fonts.body)
        .text_size(px(14.0))
        .text_color(theme.colors.muted_foreground)
}

/// Creates the styled, icon-only Dialog close control.
pub fn dialog_close(id: impl Into<ElementId>, cx: &App) -> DialogClose<()> {
    let theme = UiTheme::read(cx).clone();
    let icon_color = theme.colors.popover_foreground;
    DialogClose::new()
        .id(id)
        .aria_label("Close")
        .absolute()
        .top(px(8.0))
        .right(px(8.0))
        .style_with_state(move |state, base| {
            style_button(
                base,
                state.disabled,
                ButtonVariant::Ghost,
                ButtonSize::IconSm,
                &theme,
            )
        })
        .child(lucide(LucideIcon::X).size(px(16.0)).text_color(icon_color))
}
