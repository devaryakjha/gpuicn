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
    App, BoxShadow, Div, ElementId, FontWeight, InteractiveElement as _, ParentElement as _,
    Styled, prelude::FluentBuilder as _, px,
};
use gpui_icons::{LucideIcon, lucide};

use super::theme::UiTheme;

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
            base.flex()
                .items_center()
                .rounded(theme.radius.base)
                .px(px(10.))
                .py(px(6.))
                .font_family(theme.fonts.body.clone())
                .font_weight(FontWeight::MEDIUM)
                .text_size(px(14.))
                .text_color(theme.colors.foreground)
                .when(state.open || state.focused, |base| {
                    base.bg(theme.colors.muted)
                })
                .when(!state.disabled, |base| {
                    base.hover(move |style| style.bg(theme.colors.muted))
                })
                .when(state.disabled, |base| base.opacity(0.5))
        })
}

/// Creates the in-canvas dropdown portal.
pub fn menu_portal<P: Clone + 'static>() -> MenuPortal<P> {
    MenuPortal::new()
}

/// Creates a dropdown positioner with the pinned 4px content offset.
pub fn menu_positioner<P: Clone + 'static>() -> MenuPositioner<P> {
    MenuPositioner::new().side_offset(px(4.))
}

/// Creates the styled dropdown popup.
pub fn menu_popup<P: Clone + 'static>(id: impl Into<ElementId>, cx: &App) -> MenuPopup<P> {
    let theme = UiTheme::read(cx).clone();
    MenuPopup::new()
        .id(id)
        .style_with_state(move |_state, base| popup_style(base, &theme, px(128.)))
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
}

/// Creates a check indicator for [`menu_checkbox_item`].
pub fn menu_checkbox_item_indicator<P: Clone + 'static>(cx: &App) -> MenuCheckboxItemIndicator<P> {
    let theme = UiTheme::read(cx).clone();
    let foreground = theme.colors.foreground;
    MenuCheckboxItemIndicator::new()
        .style_with_state(move |_state, base| indicator_style(base, &theme))
        .child(
            lucide(LucideIcon::Check)
                .size(px(16.))
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
}

/// Creates a check indicator for [`menu_radio_item`].
pub fn menu_radio_item_indicator<P: Clone + 'static, V: Clone + Eq + 'static>(
    cx: &App,
) -> MenuRadioItemIndicator<P, V> {
    let theme = UiTheme::read(cx).clone();
    let foreground = theme.colors.foreground;
    MenuRadioItemIndicator::new()
        .style_with_state(move |_state, base| indicator_style(base, &theme))
        .child(
            lucide(LucideIcon::Check)
                .size(px(16.))
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
    MenuGroupLabel::new().style_with_state(move |_state, base| {
        base.px(px(6.))
            .py(px(4.))
            .font_family(theme.fonts.body.clone())
            .font_weight(FontWeight::MEDIUM)
            .text_size(px(12.))
            .text_color(theme.colors.muted_foreground)
    })
}

/// Creates a styled dropdown separator.
pub fn menu_separator(cx: &App) -> MenuSeparator {
    let theme = UiTheme::read(cx).clone();
    MenuSeparator::new().style_with_state(move |_state, base| {
        base.h(px(1.))
            .mx(px(-4.))
            .my(px(4.))
            .bg(theme.colors.border)
    })
}

/// Creates a submenu root with a caller-owned stable ID.
pub fn menu_submenu_root<P: Clone + 'static>(id: impl Into<ElementId>) -> MenuSubmenuRoot<P> {
    MenuSubmenuRoot::new().id(id)
}

/// Creates a styled submenu trigger. Add a trailing ChevronRight icon when
/// callers need the visual affordance in custom content.
pub fn menu_submenu_trigger<P: Clone + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> MenuSubmenuTrigger<P> {
    let theme = UiTheme::read(cx).clone();
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
}

pub(crate) fn popup_style(base: Div, theme: &UiTheme, min_width: gpui::Pixels) -> Div {
    base.min_w(min_width)
        .max_h(px(288.))
        .overflow_hidden()
        .rounded(theme.radius.base)
        .p(px(4.))
        .bg(theme.colors.popover)
        .text_color(theme.colors.popover_foreground)
        .font_family(theme.fonts.body.clone())
        .text_size(px(14.))
        .border_1()
        .border_color(theme.colors.foreground.alpha(0.10))
        .shadow(vec![
            BoxShadow::new(px(0.), px(4.), theme.colors.foreground.alpha(0.12).into())
                .blur_radius(px(12.)),
        ])
}

pub(crate) fn item_style(
    base: Div,
    highlighted: bool,
    disabled: bool,
    theme: &UiTheme,
    has_indicator: bool,
) -> Div {
    base.relative()
        .flex()
        .items_center()
        .gap(px(6.))
        .rounded(px(6.))
        .py(px(4.))
        .pr(px(if has_indicator { 32. } else { 6. }))
        .pl(px(6.))
        .font_family(theme.fonts.body.clone())
        .text_size(px(14.))
        .text_color(theme.colors.popover_foreground)
        .when(highlighted, |base| {
            base.bg(theme.colors.accent)
                .text_color(theme.colors.accent_foreground)
        })
        .when(disabled, |base| base.opacity(0.5))
}

pub(crate) fn indicator_style(base: Div, theme: &UiTheme) -> Div {
    base.absolute()
        .right(px(8.))
        .flex()
        .size(px(16.))
        .items_center()
        .justify_center()
        .text_color(theme.colors.popover_foreground)
}
