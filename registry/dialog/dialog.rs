//! The narrow shadcn Nova Dialog visual port.
//!
//! Visual source: shadcn/ui 4.19.0 `dialog.tsx` and `style-nova.css` at
//! `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`. Interaction, dismissal, and the
//! Popup/Close focus cycle come from the pinned Base GPUI Dialog primitives.

#[path = "modal_focus.rs"]
pub(crate) mod modal_focus;

pub use base_gpui::dialog::{
    DialogBackdrop, DialogClose, DialogDescription, DialogPopup, DialogPortal, DialogRoot,
    DialogTitle, DialogTrigger, DialogViewport,
};
use gpui::{App, Div, ElementId, FontWeight, ParentElement as _, SharedString, Styled, div, px};
use gpui_icons::{LucideIcon, lucide};

use super::{
    button::{ButtonSize, ButtonVariant, style_button},
    theme::UiTheme,
};

/// Creates the Dialog state root with a caller-owned stable ID.
/// An internal handle reconciles controlled focus. If replacing `.handle(...)`,
/// drive open/close transitions through the supplied handle.
pub fn dialog_root(id: impl Into<ElementId>) -> DialogRoot<()> {
    let id = id.into();
    let handle = base_gpui::dialog::DialogHandle::new();
    DialogRoot::new()
        .id(id.clone())
        .handle(handle.clone())
        .child_any(modal_focus::RootFocus::new(id, handle))
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
pub fn dialog_backdrop(cx: &App) -> DialogBackdrop<()> {
    DialogBackdrop::new()
        .absolute()
        .inset_0()
        .bg(UiTheme::read(cx).colors.overlay)
}

/// Creates the centered Dialog viewport with the pinned 16px page gutter.
pub fn dialog_viewport(cx: &App) -> DialogViewport<()> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    DialogViewport::new()
        .absolute()
        .inset_0()
        .flex()
        .flex_col()
        .items_stretch()
        .justify_center()
        .p(spacing * 4_f32)
}

/// Creates Nova's stacked Dialog header.
pub fn dialog_header(cx: &App) -> Div {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    div().flex().flex_col().gap(spacing * 2_f32)
}

/// Creates Nova's inset Dialog footer surface.
pub fn dialog_footer(cx: &App) -> Div {
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

/// Creates a styled Dialog popup with a caller-owned stable ID and name.
pub fn dialog_popup(
    id: impl Into<ElementId>,
    aria_label: impl Into<SharedString>,
    cx: &App,
) -> DialogPopup<()> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    let id = id.into();
    let focus = modal_focus::ModalFocus::new(id.clone());
    DialogPopup::new()
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

/// Creates a styled Dialog title with a caller-owned stable ID.
pub fn dialog_title(id: impl Into<ElementId>, cx: &App) -> DialogTitle<()> {
    let theme = UiTheme::read(cx).clone();
    let text_scale = theme.text_scale;
    DialogTitle::new()
        .id(id)
        .font_family(theme.fonts.heading)
        .font_weight(FontWeight::MEDIUM)
        .text_size(px(16.0) * text_scale)
        .line_height(px(16.0) * text_scale)
        .text_color(theme.colors.popover_foreground)
}

/// Creates a styled Dialog description with a caller-owned stable ID.
pub fn dialog_description(id: impl Into<ElementId>, cx: &App) -> DialogDescription<()> {
    let theme = UiTheme::read(cx).clone();
    let text_scale = theme.text_scale;
    DialogDescription::new()
        .id(id)
        .font_family(theme.fonts.body)
        .text_size(px(14.0) * text_scale)
        .text_color(theme.colors.muted_foreground)
}

/// Creates the styled, icon-only Dialog close control.
pub fn dialog_close(id: impl Into<ElementId>, cx: &App) -> DialogClose<()> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let icon_color = theme.colors.popover_foreground;
    DialogClose::new()
        .id(id)
        .aria_label("Close")
        .absolute()
        .top(spacing * 2_f32)
        .right(spacing * 2_f32)
        .style_with_state(move |state, base| {
            style_button(
                base,
                state.disabled,
                ButtonVariant::Ghost,
                ButtonSize::IconSm,
                &theme,
            )
        })
        .child(
            lucide(LucideIcon::X)
                .size(spacing * 4_f32)
                .text_color(icon_color),
        )
}

/// Creates a primary Dialog action that dismisses the Dialog after activation.
pub fn dialog_action(id: impl Into<ElementId>, cx: &App) -> DialogClose<()> {
    let theme = UiTheme::read(cx).clone();
    DialogClose::new()
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
