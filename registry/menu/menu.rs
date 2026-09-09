//! The shadcn Nova Dropdown Menu visual port.
//!
//! Visual source: shadcn/ui 4.19.0 `dropdown-menu.tsx` and `style-nova.css`
//! at `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`. Interaction comes from the
//! pinned Base GPUI menu primitives.

pub use base_gpui::menu::{
    MenuAlign, MenuArrow, MenuBackdrop, MenuCheckboxItem, MenuCheckboxItemIndicator, MenuContext,
    MenuGroup, MenuGroupLabel, MenuItem, MenuLinkItem, MenuOrientation, MenuPopup, MenuPortal,
    MenuPositioner, MenuRadioGroup, MenuRadioItem, MenuRadioItemIndicator, MenuRoot, MenuSeparator,
    MenuSide, MenuSubmenuRoot, MenuSubmenuTrigger, MenuTrigger,
};
use gpui::{
    App, Div, ElementId, FontWeight, ParentElement as _, Styled, prelude::FluentBuilder as _, px,
};
use gpui_icons::{LucideIcon, lucide};

use super::{
    button::{ButtonSize, ButtonVariant, style_button},
    theme::UiTheme,
};

/// Creates a dropdown menu root with a caller-owned stable ID.
pub fn menu_root<P: Clone + 'static>(id: impl Into<ElementId>) -> MenuRoot<P> {
    MenuRoot::new().id(id)
}

/// Creates a styled dropdown trigger.
pub fn menu_trigger<P: Clone + 'static>(id: impl Into<ElementId>, cx: &App) -> MenuTrigger<P> {
    let theme = UiTheme::read(cx).clone();
    MenuTrigger::new()
        .id(id)
        .style_with_state(move |state, base| {
            style_button(
                base,
                state.disabled,
                ButtonVariant::Outline,
                ButtonSize::Default,
                &theme,
            )
            .when(state.open, |base| {
                base.bg(theme.colors.muted)
                    .text_color(theme.colors.foreground)
            })
        })
}

/// Creates the in-canvas dropdown portal.
pub fn menu_portal<P: Clone + 'static>() -> MenuPortal<P> {
    MenuPortal::new()
}

/// Creates a dropdown positioner with the pinned 4px content offset.
pub fn menu_positioner<P: Clone + 'static>(cx: &App) -> MenuPositioner<P> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    MenuPositioner::new().side_offset(spacing * 1_f32)
}

/// Creates the styled dropdown popup.
pub fn menu_popup<P: Clone + 'static>(id: impl Into<ElementId>, cx: &App) -> MenuPopup<P> {
    let theme = UiTheme::read(cx).clone();
    MenuPopup::new()
        .id(id)
        .style_with_state(move |_state, base| popup_style(base, &theme, theme.space(32.)))
}

/// Creates a styled dropdown item.
pub fn menu_item<P: Clone + 'static>(id: impl Into<ElementId>, cx: &App) -> MenuItem<P> {
    let theme = UiTheme::read(cx).clone();
    MenuItem::new().id(id).style_with_state(move |state, base| {
        item_style(base, state.highlighted, state.disabled, &theme, false)
    })
}

/// Creates a styled checkable dropdown item.
pub fn menu_checkbox_item<P: Clone + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> MenuCheckboxItem<P> {
    let theme = UiTheme::read(cx).clone();
    MenuCheckboxItem::new()
        .id(id)
        .style_with_state(move |state, base| {
            item_style(base, state.highlighted, state.disabled, &theme, true)
        })
        .child(menu_checkbox_item_indicator(cx))
}

/// Creates a check indicator for [`menu_checkbox_item`].
pub fn menu_checkbox_item_indicator<P: Clone + 'static>(cx: &App) -> MenuCheckboxItemIndicator<P> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let foreground = theme.colors.foreground;
    MenuCheckboxItemIndicator::new()
        .keep_mounted(true)
        .style_with_state(move |state, base| {
            indicator_style(base, &theme).opacity(if state.checked { 1.0 } else { 0.0 })
        })
        .child(
            lucide(LucideIcon::Check)
                .size(spacing * 4_f32)
                .text_color(foreground),
        )
}

/// Creates a styled dropdown radio group.
pub fn menu_radio_group<P: Clone + 'static, V: Clone + Eq + 'static>() -> MenuRadioGroup<P, V> {
    MenuRadioGroup::new()
}

/// Creates a styled radio dropdown item.
pub fn menu_radio_item<P: Clone + 'static, V: Clone + Eq + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> MenuRadioItem<P, V> {
    let theme = UiTheme::read(cx).clone();
    MenuRadioItem::new()
        .id(id)
        .style_with_state(move |state, base| {
            item_style(base, state.highlighted, state.disabled, &theme, true)
        })
        .child(menu_radio_item_indicator(cx))
}

/// Creates a check indicator for [`menu_radio_item`].
pub fn menu_radio_item_indicator<P: Clone + 'static, V: Clone + Eq + 'static>(
    cx: &App,
) -> MenuRadioItemIndicator<P, V> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let foreground = theme.colors.foreground;
    MenuRadioItemIndicator::new()
        .keep_mounted(true)
        .style_with_state(move |state, base| {
            indicator_style(base, &theme).opacity(if state.checked { 1.0 } else { 0.0 })
        })
        .child(
            lucide(LucideIcon::Check)
                .size(spacing * 4_f32)
                .text_color(foreground),
        )
}

/// Creates a styled dropdown group.
pub fn menu_group<P: Clone + 'static>() -> MenuGroup<P> {
    MenuGroup::new()
}

/// Creates a styled dropdown group label.
pub fn menu_group_label<P: Clone + 'static>(cx: &App) -> MenuGroupLabel<P> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    MenuGroupLabel::new().style_with_state(move |_state, base| {
        base.px(spacing * 1.5_f32)
            .py(spacing * 1_f32)
            .font_family(theme.fonts.body.clone())
            .font_weight(FontWeight::MEDIUM)
            .text_size(px(12.) * text_scale)
            .text_color(theme.colors.muted_foreground)
    })
}

/// Creates a styled dropdown separator.
pub fn menu_separator(cx: &App) -> MenuSeparator {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    MenuSeparator::new().style_with_state(move |_state, base| {
        base.h(px(1.))
            .mx(spacing * -1_f32)
            .my(spacing * 1_f32)
            .bg(theme.colors.border)
    })
}

/// Creates a submenu root with a caller-owned stable ID.
pub fn menu_submenu_root<P: Clone + 'static>(id: impl Into<ElementId>) -> MenuSubmenuRoot<P> {
    MenuSubmenuRoot::new().id(id)
}

/// Creates a styled submenu trigger with Nova's trailing chevron.
pub fn menu_submenu_trigger<P: Clone + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> MenuSubmenuTrigger<P> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let icon_color = theme.colors.muted_foreground;
    MenuSubmenuTrigger::new()
        .id(id)
        .style_with_state(move |state, base| {
            item_style(
                base,
                state.highlighted || state.open,
                state.disabled,
                &theme,
                false,
            )
        })
        .child(
            lucide(LucideIcon::ChevronRight)
                .size(spacing * 4_f32)
                .text_color(icon_color),
        )
}

pub(crate) fn popup_style(base: Div, theme: &UiTheme, min_width: gpui::Pixels) -> Div {
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    base.min_w(min_width)
        .max_h(spacing * 72_f32)
        .overflow_hidden()
        .rounded(theme.radius.lg)
        .p(spacing * 1_f32)
        .bg(theme.colors.popover)
        .text_color(theme.colors.popover_foreground)
        .font_family(theme.fonts.body.clone())
        .text_size(px(14.) * text_scale)
        .border_1()
        .border_color(theme.colors.foreground.opacity(0.10))
        .shadow(theme.shadows.md.clone())
}

pub(crate) fn item_style(
    base: Div,
    highlighted: bool,
    disabled: bool,
    theme: &UiTheme,
    has_indicator: bool,
) -> Div {
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    base.relative()
        .flex()
        .items_center()
        .gap(spacing * 1.5_f32)
        .rounded(theme.radius.sm)
        .py(spacing * 1_f32)
        .pr(spacing * if has_indicator { 8. } else { 1.5 })
        .pl(spacing * 1.5_f32)
        .font_family(theme.fonts.body.clone())
        .text_size(px(14.) * text_scale)
        .text_color(theme.colors.popover_foreground)
        .when(!disabled, |base| base.cursor_pointer())
        .when(highlighted && !disabled, |base| {
            base.bg(theme.colors.accent)
                .text_color(theme.colors.accent_foreground)
        })
        .when(disabled, |base| base.opacity(0.5).cursor_not_allowed())
}

pub(crate) fn indicator_style(base: Div, theme: &UiTheme) -> Div {
    let spacing = theme.spacing.unit;
    base.absolute()
        .right(spacing * 2_f32)
        .flex()
        .size(spacing * 4_f32)
        .items_center()
        .justify_center()
        .text_color(theme.colors.popover_foreground)
}
