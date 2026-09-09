//! The shadcn Nova Navigation Menu visual port.
//!
//! Visual source: shadcn/ui 4.19.0 `navigation-menu.tsx` and
//! `style-nova.css` at `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`.
//! Hover delays, safe-polygon handling, keyboard navigation, and popup
//! positioning come from the pinned Base GPUI Navigation Menu primitives.

pub use base_gpui::navigation_menu::{
    NavigationMenuAlign, NavigationMenuArrow, NavigationMenuBackdrop, NavigationMenuContent,
    NavigationMenuIcon, NavigationMenuItem, NavigationMenuLink, NavigationMenuList,
    NavigationMenuPopup, NavigationMenuPortal, NavigationMenuPositioner, NavigationMenuRoot,
    NavigationMenuTrigger, NavigationMenuViewport,
};
use gpui::{
    App, FontWeight, InteractiveElement as _, ParentElement as _, Styled,
    prelude::FluentBuilder as _, px,
};
use gpui_icons::{LucideIcon, lucide};

use super::theme::UiTheme;

/// Creates a horizontal Navigation Menu root. Set an accessible label when a window has more than one navigation landmark.
pub fn navigation_menu<T: Clone + Eq + 'static>(cx: &App) -> NavigationMenuRoot<T> {
    let theme = UiTheme::read(cx).clone();
    NavigationMenuRoot::new()
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .font_family(theme.fonts.body)
}

/// Creates the visible list container.
pub fn navigation_menu_list<T: Clone + Eq + 'static>() -> NavigationMenuList<T> {
    NavigationMenuList::new().flex().items_center()
}

/// Creates a positioning item container.
pub fn navigation_menu_item<T: Clone + Eq + 'static>() -> NavigationMenuItem<T> {
    NavigationMenuItem::new().relative()
}

/// Creates a Nova trigger with a chevron that reflects its open state.
pub fn navigation_menu_trigger<T: Clone + Eq + 'static>(cx: &App) -> NavigationMenuTrigger<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    let icon_color = theme.colors.muted_foreground;
    let focus_ring = theme.focus_ring();
    NavigationMenuTrigger::new().style_with_state(move |state, base| {
        let colors = theme.colors;
        let focus_ring = focus_ring.clone();
        base.flex()
            .flex_row_reverse()
            .gap(spacing * 1_f32)
            .items_center()
            .justify_center()
            .h(spacing * 8_f32)
            .rounded(theme.radius.lg)
            .border_1()
            .border_color(colors.background.opacity(0.0))
            .px(spacing * 2.5_f32)
            .py(spacing * 1.5_f32)
            .font_family(theme.fonts.body.clone())
            .font_weight(FontWeight::MEDIUM)
            .text_size(px(14.0) * text_scale)
            .line_height(px(20.0) * text_scale)
            .text_color(colors.foreground)
            .when(state.open, |base| base.bg(colors.muted.opacity(0.50)))
            .when(!state.disabled, |base| {
                base.cursor_pointer()
                    .hover(move |style| style.bg(colors.muted))
            })
            .when(state.disabled, |base| {
                base.opacity(0.50).cursor_not_allowed()
            })
            .focus_visible(move |style| {
                style
                    .bg(colors.muted)
                    .border_color(colors.ring)
                    .shadow(focus_ring.clone())
            })
            .child(super::theme::disclosure_icon(
                lucide(LucideIcon::ChevronDown)
                    .size(spacing * 3_f32)
                    .text_color(icon_color),
                state.open,
            ))
    })
}

/// Creates the popup content container.
pub fn navigation_menu_content<T: Clone + Eq + 'static>(cx: &App) -> NavigationMenuContent<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    NavigationMenuContent::new().style_with_state(move |_state, base| {
        base.p(spacing * 1_f32)
            .font_family(theme.fonts.body.clone())
            .text_size(px(14.0) * text_scale)
            .line_height(px(20.0) * text_scale)
            .text_color(theme.colors.popover_foreground)
    })
}

/// Creates the portal that hosts popup geometry outside the menu root.
pub fn navigation_menu_portal<T: Clone + Eq + 'static>() -> NavigationMenuPortal<T> {
    NavigationMenuPortal::new()
}

/// Creates a positioned popup surface.
pub fn navigation_menu_positioner<T: Clone + Eq + 'static>(
    cx: &App,
) -> NavigationMenuPositioner<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    NavigationMenuPositioner::new()
        .align(NavigationMenuAlign::Start)
        .side_offset(spacing * 1_f32)
}

/// Creates the Nova popup card.
pub fn navigation_menu_popup<T: Clone + Eq + 'static>(cx: &App) -> NavigationMenuPopup<T> {
    let theme = UiTheme::read(cx).clone();
    NavigationMenuPopup::new().style_with_state(move |_state, base| {
        base.rounded(theme.radius.lg)
            .bg(theme.colors.popover)
            .text_color(theme.colors.popover_foreground)
            .border_1()
            .border_color(theme.colors.foreground.opacity(0.10))
            .shadow(theme.shadows.sm.clone())
    })
}

/// Creates the animated viewport inside a Navigation Menu popup.
pub fn navigation_menu_viewport<T: Clone + Eq + 'static>(cx: &App) -> NavigationMenuViewport<T> {
    let theme = UiTheme::read(cx).clone();
    NavigationMenuViewport::new().style_with_state(move |_state, base| {
        base.overflow_hidden()
            .rounded(theme.radius.lg)
            .bg(theme.colors.popover)
            .text_color(theme.colors.popover_foreground)
    })
}

/// Creates a Nova navigation link.
pub fn navigation_menu_link<T: Clone + Eq + 'static>(cx: &App) -> NavigationMenuLink<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    let focus_ring = theme.focus_ring();
    NavigationMenuLink::new().style_with_state(move |state, base| {
        let colors = theme.colors;
        let focus_ring = focus_ring.clone();
        base.flex()
            .items_center()
            .gap(spacing * 2_f32)
            .rounded(theme.radius.lg)
            .border_1()
            .border_color(colors.popover.opacity(0.0))
            .p(spacing * 2_f32)
            .font_family(theme.fonts.body.clone())
            .text_size(px(14.0) * text_scale)
            .line_height(px(20.0) * text_scale)
            .text_color(colors.popover_foreground)
            .cursor_pointer()
            .when(state.active, |base| base.bg(colors.muted.opacity(0.50)))
            .hover(move |style| style.bg(colors.muted))
            .focus_visible(move |style| style.border_color(colors.ring).shadow(focus_ring.clone()))
    })
}

/// Creates a small icon slot. Callers own its contents.
pub fn navigation_menu_icon<T: Clone + Eq + 'static>(cx: &App) -> NavigationMenuIcon<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    NavigationMenuIcon::new().style_with_state(move |_state, base| {
        base.ml(spacing * 1_f32)
            .size(spacing * 3_f32)
            .text_color(theme.colors.muted_foreground)
    })
}

/// Creates the optional popup arrow treatment.
pub fn navigation_menu_arrow<T: Clone + Eq + 'static>(cx: &App) -> NavigationMenuArrow<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    NavigationMenuArrow::new().style_with_state(move |_state, base| {
        base.size(spacing * 2_f32)
            .bg(theme.colors.border)
            .rounded(theme.radius.sm / 3.)
    })
}

/// Creates the optional transparent backdrop, retained for outside-click dismissal.
pub fn navigation_menu_backdrop<T: Clone + Eq + 'static>() -> NavigationMenuBackdrop<T> {
    NavigationMenuBackdrop::new()
}
