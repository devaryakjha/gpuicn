//! The shadcn Nova Context Menu visual port.
//!
//! Visual source: shadcn/ui 4.19.0 `context-menu.tsx` and `style-nova.css`
//! at `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`. Interaction comes from the
//! pinned Base GPUI context-menu primitives.

pub use base_gpui::context_menu::{
    ContextMenuArrow, ContextMenuBackdrop, ContextMenuCheckboxItem,
    ContextMenuCheckboxItemIndicator, ContextMenuGroup, ContextMenuGroupLabel, ContextMenuItem,
    ContextMenuLinkItem, ContextMenuPortal, ContextMenuPositioner, ContextMenuRadioGroup,
    ContextMenuRadioItem, ContextMenuRadioItemIndicator, ContextMenuRoot, ContextMenuSeparator,
    ContextMenuSubmenuRoot, ContextMenuSubmenuTrigger, ContextMenuTrigger,
};
use base_gpui::menu::{MenuPopup as ContextMenuPopup, MenuSide};
use gpui::{App, ElementId, FontWeight, ParentElement as _, Styled, px};
use gpui_icons::{LucideIcon, lucide};

use super::{menu, theme::UiTheme};

/// Creates a context-menu root with a caller-owned stable ID.
pub fn context_menu_root<P: Clone + 'static>(id: impl Into<ElementId>) -> ContextMenuRoot<P> {
    ContextMenuRoot::new().id(id)
}

/// Creates the right-click context area with a caller-owned stable ID.
pub fn context_menu_trigger<P: Clone + 'static>(id: impl Into<ElementId>) -> ContextMenuTrigger<P> {
    ContextMenuTrigger::new().id(id)
}

/// Creates the in-canvas context-menu portal.
pub fn context_menu_portal<P: Clone + 'static>() -> ContextMenuPortal<P> {
    ContextMenuPortal::new()
}

/// Creates the cursor-anchored context-menu positioner.
pub fn context_menu_positioner<P: Clone + 'static>() -> ContextMenuPositioner<P> {
    ContextMenuPositioner::new()
        .side(MenuSide::Right)
        .align_offset(px(4.))
}

/// Creates the styled context-menu popup.
pub fn context_menu_popup<P: Clone + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> ContextMenuPopup<P> {
    let theme = UiTheme::read(cx).clone();
    ContextMenuPopup::new()
        .id(id)
        .style_with_state(move |_state, base| menu::popup_style(base, &theme, px(144.)))
}

/// Creates a styled context-menu item.
pub fn context_menu_item<P: Clone + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> ContextMenuItem<P> {
    let theme = UiTheme::read(cx).clone();
    ContextMenuItem::new()
        .id(id)
        .style_with_state(move |state, base| {
            menu::item_style(base, state.highlighted, state.disabled, &theme, false)
        })
}

/// Creates a styled checkable context-menu item.
pub fn context_menu_checkbox_item<P: Clone + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> ContextMenuCheckboxItem<P> {
    let theme = UiTheme::read(cx).clone();
    ContextMenuCheckboxItem::new()
        .id(id)
        .style_with_state(move |state, base| {
            menu::item_style(base, state.highlighted, state.disabled, &theme, true)
        })
}

/// Creates the check indicator for a context-menu checkbox item.
pub fn context_menu_checkbox_item_indicator<P: Clone + 'static>(
    cx: &App,
) -> ContextMenuCheckboxItemIndicator<P> {
    let theme = UiTheme::read(cx).clone();
    let foreground = theme.colors.foreground;
    ContextMenuCheckboxItemIndicator::new()
        .style_with_state(move |_state, base| menu::indicator_style(base, &theme))
        .child(
            lucide(LucideIcon::Check)
                .size(px(16.))
                .text_color(foreground),
        )
}

/// Creates a context-menu radio group.
pub fn context_menu_radio_group<P: Clone + 'static, V: Clone + Eq + 'static>()
-> ContextMenuRadioGroup<P, V> {
    ContextMenuRadioGroup::new()
}

/// Creates a styled context-menu radio item.
pub fn context_menu_radio_item<P: Clone + 'static, V: Clone + Eq + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> ContextMenuRadioItem<P, V> {
    let theme = UiTheme::read(cx).clone();
    ContextMenuRadioItem::new()
        .id(id)
        .style_with_state(move |state, base| {
            menu::item_style(base, state.highlighted, state.disabled, &theme, true)
        })
}

/// Creates the check indicator for a context-menu radio item.
pub fn context_menu_radio_item_indicator<P: Clone + 'static, V: Clone + Eq + 'static>(
    cx: &App,
) -> ContextMenuRadioItemIndicator<P, V> {
    let theme = UiTheme::read(cx).clone();
    let foreground = theme.colors.foreground;
    ContextMenuRadioItemIndicator::new()
        .style_with_state(move |_state, base| menu::indicator_style(base, &theme))
        .child(
            lucide(LucideIcon::Check)
                .size(px(16.))
                .text_color(foreground),
        )
}

/// Creates a context-menu group.
pub fn context_menu_group<P: Clone + 'static>() -> ContextMenuGroup<P> {
    ContextMenuGroup::new()
}

/// Creates a styled context-menu group label.
pub fn context_menu_group_label<P: Clone + 'static>(cx: &App) -> ContextMenuGroupLabel<P> {
    let theme = UiTheme::read(cx).clone();
    ContextMenuGroupLabel::new().style_with_state(move |_state, base| {
        base.px(px(6.))
            .py(px(4.))
            .font_family(theme.fonts.body.clone())
            .font_weight(FontWeight::MEDIUM)
            .text_size(px(12.))
            .text_color(theme.colors.muted_foreground)
    })
}

/// Creates a styled context-menu separator.
pub fn context_menu_separator(cx: &App) -> ContextMenuSeparator {
    let theme = UiTheme::read(cx).clone();
    ContextMenuSeparator::new().style_with_state(move |_state, base| {
        base.h(px(1.))
            .mx(px(-4.))
            .my(px(4.))
            .bg(theme.colors.border)
    })
}

/// Creates a context-menu submenu root with a caller-owned stable ID.
pub fn context_menu_submenu_root<P: Clone + 'static>(
    id: impl Into<ElementId>,
) -> ContextMenuSubmenuRoot<P> {
    ContextMenuSubmenuRoot::new().id(id)
}

/// Creates a styled context-menu submenu trigger.
pub fn context_menu_submenu_trigger<P: Clone + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> ContextMenuSubmenuTrigger<P> {
    let theme = UiTheme::read(cx).clone();
    ContextMenuSubmenuTrigger::new()
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
}
