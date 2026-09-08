//! The shadcn Nova Menubar visual port.
//!
//! Visual source: shadcn/ui 4.19.0 `menubar.tsx` and `style-nova.css` at
//! `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`. Interaction comes from the
//! pinned Base GPUI menubar and menu primitives.

pub use base_gpui::menu::{
    MenuCheckboxItem as MenubarCheckboxItem,
    MenuCheckboxItemIndicator as MenubarCheckboxItemIndicator, MenuGroup as MenubarGroup,
    MenuGroupLabel as MenubarLabel, MenuItem as MenubarItem, MenuPopup as MenubarContent,
    MenuPortal as MenubarPortal, MenuRadioGroup as MenubarRadioGroup,
    MenuRadioItem as MenubarRadioItem, MenuRadioItemIndicator as MenubarRadioItemIndicator,
    MenuRoot as MenubarMenu, MenuSeparator as MenubarSeparator, MenuSubmenuRoot as MenubarSub,
    MenuSubmenuTrigger as MenubarSubTrigger, MenuTrigger as MenubarTrigger,
};
pub use base_gpui::menubar::{Menubar, MenubarOrientation};
use gpui::{
    App, ElementId, FontWeight, ParentElement as _, Styled, prelude::FluentBuilder as _, px,
};
use gpui_icons::{LucideIcon, lucide};

use super::{menu, theme::UiTheme};

/// Creates the styled menubar container with a caller-owned stable ID.
pub fn menubar(id: impl Into<ElementId>, cx: &App) -> Menubar {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    Menubar::new().id(id).style_with_state(move |_state, base| {
        base.flex()
            .items_center()
            .h(spacing * 8_f32)
            .gap(spacing * 0.5_f32)
            .rounded(theme.radius.lg)
            .border_1()
            .border_color(theme.colors.border)
            .p(spacing * 0.75_f32)
            .bg(theme.colors.background)
            .font_family(theme.fonts.body.clone())
    })
}

/// Creates a menubar-owned menu root with a caller-owned stable ID.
pub fn menubar_menu<P: Clone + 'static>(id: impl Into<ElementId>) -> MenubarMenu<P> {
    MenubarMenu::new().id(id)
}

/// Creates the styled trigger for one menubar menu.
pub fn menubar_trigger<P: Clone + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> MenubarTrigger<P> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    MenubarTrigger::new()
        .id(id)
        .style_with_state(move |state, base| {
            base.flex()
                .items_center()
                .rounded(theme.radius.sm)
                .px(spacing * 1.5_f32)
                .py(spacing * 0.5_f32)
                .font_family(theme.fonts.body.clone())
                .font_weight(FontWeight::MEDIUM)
                .text_size(px(14.) * text_scale)
                .text_color(theme.colors.foreground)
                .when(state.open || state.focused, |base| {
                    base.bg(theme.colors.muted)
                })
                .when(!state.disabled, |base| base.cursor_pointer())
                .when(state.disabled, |base| {
                    base.opacity(0.5).cursor_not_allowed()
                })
        })
}

/// Creates the in-canvas portal for one menubar menu.
pub fn menubar_portal<P: Clone + 'static>() -> MenubarPortal<P> {
    MenubarPortal::new()
}

/// Creates the styled popup content for a menubar menu.
pub fn menubar_content<P: Clone + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> MenubarContent<P> {
    let theme = UiTheme::read(cx).clone();
    MenubarContent::new()
        .id(id)
        .style_with_state(move |_state, base| menu::popup_style(base, &theme, theme.space(36.)))
}

/// Creates a styled menubar item.
pub fn menubar_item<P: Clone + 'static>(id: impl Into<ElementId>, cx: &App) -> MenubarItem<P> {
    let theme = UiTheme::read(cx).clone();
    MenubarItem::new()
        .id(id)
        .style_with_state(move |state, base| {
            menu::item_style(base, state.highlighted, state.disabled, &theme, false)
        })
}

/// Creates a styled menubar checkbox item.
pub fn menubar_checkbox_item<P: Clone + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> MenubarCheckboxItem<P> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    MenubarCheckboxItem::new()
        .id(id)
        .style_with_state(move |state, base| {
            menu::item_style(base, state.highlighted, state.disabled, &theme, true)
                .pr(spacing * 1.5_f32)
                .pl(spacing * 7_f32)
        })
        .child(menubar_checkbox_item_indicator(cx))
}

/// Creates the check indicator for a menubar checkbox item.
pub fn menubar_checkbox_item_indicator<P: Clone + 'static>(
    cx: &App,
) -> MenubarCheckboxItemIndicator<P> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    MenubarCheckboxItemIndicator::new()
        .keep_mounted(true)
        .style_with_state(move |state, base| {
            base.absolute()
                .left(spacing * 1.5_f32)
                .flex()
                .size(spacing * 4_f32)
                .items_center()
                .justify_center()
                .opacity(if state.checked { 1.0 } else { 0.0 })
        })
        .child(
            lucide(LucideIcon::Check)
                .size(spacing * 4_f32)
                .text_color(theme.colors.foreground),
        )
}

/// Creates a menubar radio group.
pub fn menubar_radio_group<P: Clone + 'static, V: Clone + Eq + 'static>() -> MenubarRadioGroup<P, V>
{
    MenubarRadioGroup::new()
}

/// Creates a styled menubar radio item.
pub fn menubar_radio_item<P: Clone + 'static, V: Clone + Eq + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> MenubarRadioItem<P, V> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    MenubarRadioItem::new()
        .id(id)
        .style_with_state(move |state, base| {
            menu::item_style(base, state.highlighted, state.disabled, &theme, true)
                .pr(spacing * 1.5_f32)
                .pl(spacing * 7_f32)
        })
        .child(menubar_radio_item_indicator(cx))
}

/// Creates the check indicator for a menubar radio item.
pub fn menubar_radio_item_indicator<P: Clone + 'static, V: Clone + Eq + 'static>(
    cx: &App,
) -> MenubarRadioItemIndicator<P, V> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    MenubarRadioItemIndicator::new()
        .keep_mounted(true)
        .style_with_state(move |state, base| {
            base.absolute()
                .left(spacing * 1.5_f32)
                .flex()
                .size(spacing * 4_f32)
                .items_center()
                .justify_center()
                .opacity(if state.checked { 1.0 } else { 0.0 })
        })
        .child(
            lucide(LucideIcon::Check)
                .size(spacing * 4_f32)
                .text_color(theme.colors.foreground),
        )
}

/// Creates a menubar group.
pub fn menubar_group<P: Clone + 'static>() -> MenubarGroup<P> {
    MenubarGroup::new()
}

/// Creates a styled menubar group label.
pub fn menubar_label<P: Clone + 'static>(cx: &App) -> MenubarLabel<P> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    MenubarLabel::new().style_with_state(move |_state, base| {
        base.px(spacing * 1.5_f32)
            .py(spacing * 1_f32)
            .font_family(theme.fonts.body.clone())
            .font_weight(FontWeight::MEDIUM)
            .text_size(px(14.) * text_scale)
            .text_color(theme.colors.popover_foreground)
    })
}

/// Creates a styled menubar separator.
pub fn menubar_separator(cx: &App) -> MenubarSeparator {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    MenubarSeparator::new().style_with_state(move |_state, base| {
        base.h(px(1.))
            .mx(spacing * -1_f32)
            .my(spacing * 1_f32)
            .bg(theme.colors.border)
    })
}

/// Creates a menubar submenu root with a caller-owned stable ID.
pub fn menubar_sub<P: Clone + 'static>(id: impl Into<ElementId>) -> MenubarSub<P> {
    MenubarSub::new().id(id)
}

/// Creates a styled menubar submenu trigger.
pub fn menubar_sub_trigger<P: Clone + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> MenubarSubTrigger<P> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let icon_color = theme.colors.muted_foreground;
    MenubarSubTrigger::new()
        .id(id)
        .style_with_state(move |state, base| {
            menu::item_style(
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
