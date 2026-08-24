//! The shadcn Nova Select visual port.
//!
//! Visual source: shadcn/ui 4.19.0 `select.tsx` and `style-nova.css` at
//! `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`. Interaction comes from the
//! pinned Base GPUI select primitives.

pub use base_gpui::select::{
    SelectAlign, SelectArrow, SelectBackdrop, SelectGroup, SelectGroupLabel, SelectIcon,
    SelectItem, SelectItemIndicator, SelectItemText, SelectLabel, SelectList, SelectPopup,
    SelectPortal, SelectPositioner, SelectRoot, SelectScrollDownArrow, SelectScrollUpArrow,
    SelectSelectionMode, SelectSeparator, SelectSide, SelectTrigger, SelectValue,
};
use gpui::{
    App, BoxShadow, Div, ElementId, ParentElement as _, Styled, prelude::FluentBuilder as _, px,
};
use gpui_icons::{LucideIcon, lucide};

use super::theme::UiTheme;

/// Creates a select root with a caller-owned stable ID.
pub fn select_root<T: Clone + Eq + 'static>(id: impl Into<ElementId>) -> SelectRoot<T> {
    SelectRoot::new().id(id)
}

/// Creates the styled select trigger.
pub fn select_trigger<T: Clone + Eq + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> SelectTrigger<T> {
    let theme = UiTheme::read(cx).clone();
    SelectTrigger::new()
        .id(id)
        .style_with_state(move |state, base| {
            let ring = theme.colors.ring.alpha(0.50);
            base.flex()
                .items_center()
                .justify_between()
                .h(px(32.))
                .gap(px(6.))
                .rounded(theme.radius.base)
                .border_1()
                .border_color(theme.colors.input)
                .px(px(10.))
                .py(px(4.))
                .bg(theme.colors.background)
                .font_family(theme.fonts.body.clone())
                .text_size(px(14.))
                .text_color(if state.placeholder {
                    theme.colors.muted_foreground
                } else {
                    theme.colors.foreground
                })
                .when(state.root.focused, |base| {
                    base.border_color(theme.colors.ring).shadow(vec![
                        BoxShadow::new(px(0.), px(0.), ring.into()).spread_radius(px(3.)),
                    ])
                })
                .when(state.root.disabled, |base| base.opacity(0.5))
        })
}

/// Creates the styled select value text.
pub fn select_value<T: Clone + Eq + 'static>(cx: &App) -> SelectValue<T> {
    let theme = UiTheme::read(cx).clone();
    SelectValue::new().style_with_state(move |state, base| {
        base.flex_1().text_color(if state.placeholder {
            theme.colors.muted_foreground
        } else {
            theme.colors.foreground
        })
    })
}

/// Creates a styled select icon with Base GPUI's default glyph.
pub fn select_icon<T: Clone + Eq + 'static>(cx: &App) -> SelectIcon<T> {
    let theme = UiTheme::read(cx).clone();
    SelectIcon::new().style_with_state(move |_state, base| {
        base.flex_shrink_0()
            .text_size(px(16.))
            .text_color(theme.colors.muted_foreground)
    })
}

/// Creates the in-canvas select portal.
pub fn select_portal<T: Clone + Eq + 'static>() -> SelectPortal<T> {
    SelectPortal::new()
}

/// Creates a select positioner with the pinned 4px content offset.
pub fn select_positioner<T: Clone + Eq + 'static>() -> SelectPositioner<T> {
    SelectPositioner::new().side_offset(px(4.))
}

/// Creates the styled select popup.
pub fn select_popup<T: Clone + Eq + 'static>(cx: &App) -> SelectPopup<T> {
    let theme = UiTheme::read(cx).clone();
    SelectPopup::new().style_with_state(move |_state, base| popup_style(base, &theme))
}

/// Creates the styled select list.
pub fn select_list<T: Clone + Eq + 'static>() -> SelectList<T> {
    SelectList::new().style_with_state(move |_state, base| base.p(px(4.)))
}

/// Creates a styled select group.
pub fn select_group<T: Clone + Eq + 'static>() -> SelectGroup<T> {
    SelectGroup::new().style_with_state(move |_state, base| base.p(px(4.)))
}

/// Creates a styled select group label.
pub fn select_group_label<T: Clone + Eq + 'static>(cx: &App) -> SelectGroupLabel<T> {
    let theme = UiTheme::read(cx).clone();
    SelectGroupLabel::new().style_with_state(move |_state, base| {
        base.px(px(6.))
            .py(px(4.))
            .font_family(theme.fonts.body.clone())
            .text_size(px(12.))
            .text_color(theme.colors.muted_foreground)
    })
}

/// Creates a styled select item.
pub fn select_item<T: Clone + Eq + 'static>(id: impl Into<ElementId>, cx: &App) -> SelectItem<T> {
    let theme = UiTheme::read(cx).clone();
    SelectItem::new()
        .id(id)
        .style_with_state(move |state, base| {
            item_style(
                base,
                state.highlighted || state.focused,
                state.disabled,
                &theme,
            )
        })
}

/// Creates the text region within a select item.
pub fn select_item_text<T: Clone + Eq + 'static>() -> SelectItemText<T> {
    SelectItemText::new().style_with_state(move |_state, base| base.flex_1().whitespace_nowrap())
}

/// Creates the selected-item check indicator.
pub fn select_item_indicator<T: Clone + Eq + 'static>(cx: &App) -> SelectItemIndicator<T> {
    let theme = UiTheme::read(cx).clone();
    SelectItemIndicator::new()
        .style_with_state(move |_state, base| {
            base.absolute()
                .right(px(8.))
                .flex()
                .size(px(16.))
                .items_center()
                .justify_center()
        })
        .child(
            lucide(LucideIcon::Check)
                .size(px(16.))
                .text_color(theme.colors.foreground),
        )
}

/// Creates a styled select separator.
pub fn select_separator(cx: &App) -> SelectSeparator {
    let theme = UiTheme::read(cx).clone();
    SelectSeparator::new().style_with_state(move |_state, base| {
        base.h(px(1.))
            .mx(px(-4.))
            .my(px(4.))
            .bg(theme.colors.border)
    })
}

/// Creates the styled select scroll-up affordance.
pub fn select_scroll_up_arrow<T: Clone + Eq + 'static>(cx: &App) -> SelectScrollUpArrow<T> {
    let theme = UiTheme::read(cx).clone();
    SelectScrollUpArrow::new()
        .style_with_state(move |_state, base| scroll_arrow_style(base, &theme))
        .child("⌃")
}

/// Creates the styled select scroll-down affordance.
pub fn select_scroll_down_arrow<T: Clone + Eq + 'static>(cx: &App) -> SelectScrollDownArrow<T> {
    let theme = UiTheme::read(cx).clone();
    SelectScrollDownArrow::new()
        .style_with_state(move |_state, base| scroll_arrow_style(base, &theme))
        .child("⌄")
}

fn popup_style(base: Div, theme: &UiTheme) -> Div {
    base.min_w(px(144.))
        .max_h(px(288.))
        .overflow_hidden()
        .rounded(theme.radius.base)
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

fn item_style(base: Div, highlighted: bool, disabled: bool, theme: &UiTheme) -> Div {
    base.relative()
        .flex()
        .items_center()
        .gap(px(6.))
        .rounded(px(6.))
        .py(px(4.))
        .pr(px(32.))
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

fn scroll_arrow_style(base: Div, theme: &UiTheme) -> Div {
    base.flex()
        .h(px(28.))
        .items_center()
        .justify_center()
        .bg(theme.colors.popover)
        .text_color(theme.colors.popover_foreground)
}
