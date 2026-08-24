//! The shadcn Nova Navigation Menu visual port.
//!
//! Visual source: shadcn/ui 4.19.0 `navigation-menu.tsx` and
//! `style-nova.css` at `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`.
//! Hover delays, safe-polygon handling, keyboard navigation, and popup
//! positioning come from the pinned Base GPUI Navigation Menu primitives.

pub use base_gpui::navigation_menu::{
    NavigationMenuArrow, NavigationMenuBackdrop, NavigationMenuContent, NavigationMenuIcon,
    NavigationMenuItem, NavigationMenuLink, NavigationMenuList, NavigationMenuPopup,
    NavigationMenuPortal, NavigationMenuPositioner, NavigationMenuRoot, NavigationMenuTrigger,
    NavigationMenuViewport,
};
use gpui::{
    App, BoxShadow, FontWeight, InteractiveElement as _, ParentElement as _, Styled,
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

/// Creates a Nova trigger with its rotating-chevron slot.
pub fn navigation_menu_trigger<T: Clone + Eq + 'static>(cx: &App) -> NavigationMenuTrigger<T> {
    let theme = UiTheme::read(cx).clone();
    let icon_color = theme.colors.muted_foreground;
    NavigationMenuTrigger::new()
        .style_with_state(move |state, base| {
            let colors = theme.colors;
            base.flex()
                .items_center()
                .justify_center()
                .h(px(32.0))
                .rounded(theme.radius.base)
                .border_1()
                .border_color(colors.background.alpha(0.0))
                .px(px(10.0))
                .py(px(6.0))
                .font_family(theme.fonts.body.clone())
                .font_weight(FontWeight::MEDIUM)
                .text_size(px(14.0))
                .text_color(colors.foreground)
                .when(state.open, |base| base.bg(colors.muted.alpha(0.50)))
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
                        .shadow(vec![
                            BoxShadow::new(px(0.0), px(0.0), colors.ring.alpha(0.50).into())
                                .spread_radius(px(3.0)),
                        ])
                })
        })
        .child(
            navigation_menu_icon(cx).child(
                lucide(LucideIcon::ChevronDown)
                    .size(px(12.0))
                    .text_color(icon_color),
            ),
        )
}

/// Creates the popup content container.
pub fn navigation_menu_content<T: Clone + Eq + 'static>(cx: &App) -> NavigationMenuContent<T> {
    let theme = UiTheme::read(cx).clone();
    NavigationMenuContent::new().style_with_state(move |_state, base| {
        base.p(px(4.0))
            .font_family(theme.fonts.body.clone())
            .text_color(theme.colors.popover_foreground)
    })
}

/// Creates the portal that hosts popup geometry outside the menu root.
pub fn navigation_menu_portal<T: Clone + Eq + 'static>() -> NavigationMenuPortal<T> {
    NavigationMenuPortal::new()
}

/// Creates a positioned popup surface.
pub fn navigation_menu_positioner<T: Clone + Eq + 'static>() -> NavigationMenuPositioner<T> {
    NavigationMenuPositioner::new()
}

/// Creates the Nova popup card.
pub fn navigation_menu_popup<T: Clone + Eq + 'static>(cx: &App) -> NavigationMenuPopup<T> {
    let theme = UiTheme::read(cx).clone();
    NavigationMenuPopup::new().style_with_state(move |_state, base| {
        base.rounded(theme.radius.base)
            .bg(theme.colors.popover)
            .text_color(theme.colors.popover_foreground)
            .shadow(vec![
                BoxShadow::new(px(0.0), px(0.0), theme.colors.foreground.alpha(0.10).into())
                    .spread_radius(px(1.0)),
            ])
    })
}

/// Creates the animated viewport inside a Navigation Menu popup.
pub fn navigation_menu_viewport<T: Clone + Eq + 'static>(cx: &App) -> NavigationMenuViewport<T> {
    let theme = UiTheme::read(cx).clone();
    NavigationMenuViewport::new().style_with_state(move |_state, base| {
        base.overflow_hidden()
            .rounded(theme.radius.base)
            .bg(theme.colors.popover)
            .text_color(theme.colors.popover_foreground)
    })
}

/// Creates a Nova navigation link.
pub fn navigation_menu_link<T: Clone + Eq + 'static>(cx: &App) -> NavigationMenuLink<T> {
    let theme = UiTheme::read(cx).clone();
    NavigationMenuLink::new().style_with_state(move |state, base| {
        let colors = theme.colors;
        base.flex()
            .items_center()
            .gap(px(8.0))
            .rounded(theme.radius.base)
            .border_1()
            .border_color(colors.popover.alpha(0.0))
            .p(px(8.0))
            .font_family(theme.fonts.body.clone())
            .text_size(px(14.0))
            .text_color(colors.popover_foreground)
            .when(state.active, |base| base.bg(colors.muted.alpha(0.50)))
            .hover(move |style| style.bg(colors.muted))
            .focus_visible(move |style| {
                style.border_color(colors.ring).shadow(vec![
                    BoxShadow::new(px(0.0), px(0.0), colors.ring.alpha(0.50).into())
                        .spread_radius(px(3.0)),
                ])
            })
    })
}

/// Creates a small chevron/icon slot that rotates with its trigger state.
pub fn navigation_menu_icon<T: Clone + Eq + 'static>(cx: &App) -> NavigationMenuIcon<T> {
    let theme = UiTheme::read(cx).clone();
    NavigationMenuIcon::new().style_with_state(move |_state, base| {
        base.ml(px(4.0))
            .size(px(12.0))
            .text_color(theme.colors.muted_foreground)
    })
}

/// Creates the optional popup arrow treatment.
pub fn navigation_menu_arrow<T: Clone + Eq + 'static>(cx: &App) -> NavigationMenuArrow<T> {
    let theme = UiTheme::read(cx).clone();
    NavigationMenuArrow::new().style_with_state(move |_state, base| {
        base.size(px(8.0)).bg(theme.colors.border).rounded(px(2.0))
    })
}

/// Creates the optional transparent backdrop, retained for outside-click dismissal.
pub fn navigation_menu_backdrop<T: Clone + Eq + 'static>() -> NavigationMenuBackdrop<T> {
    NavigationMenuBackdrop::new()
}
