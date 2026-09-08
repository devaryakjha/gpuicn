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
use gpui::{App, Div, ElementId, ParentElement as _, Styled, prelude::FluentBuilder as _, px};
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
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    SelectTrigger::new()
        .id(id)
        .style_with_state(move |state, base| {
            let focus_ring = if state.root.invalid {
                theme.destructive_focus_ring()
            } else {
                theme.focus_ring()
            };
            let border = if state.root.invalid {
                theme.colors.destructive
            } else {
                theme.colors.input
            };
            base.flex()
                .flex_row_reverse()
                .items_center()
                .justify_between()
                .h(spacing * 8_f32)
                .gap(spacing * 1.5_f32)
                .rounded(theme.radius.lg)
                .border_1()
                .border_color(border)
                .px(spacing * 2.5_f32)
                .py(spacing * 1_f32)
                .bg(theme.colors.background)
                .font_family(theme.fonts.body.clone())
                .text_size(px(14.) * text_scale)
                .text_color(if state.placeholder {
                    theme.colors.muted_foreground
                } else {
                    theme.colors.foreground
                })
                .when(state.root.focused, |base| {
                    base.border_color(if state.root.invalid {
                        border
                    } else {
                        theme.colors.ring
                    })
                    .shadow(focus_ring.clone())
                })
                .when(!state.root.disabled && !state.root.read_only, |base| {
                    base.cursor_pointer()
                })
                .when(state.root.disabled, |base| {
                    base.opacity(0.5).cursor_not_allowed()
                })
        })
        .child(select_icon(cx))
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

/// Creates Nova's select chevron icon.
pub fn select_icon<T: Clone + Eq + 'static>(cx: &App) -> SelectIcon<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    SelectIcon::new()
        .style_with_state(move |_state, base| {
            base.flex_shrink_0()
                .text_color(theme.colors.muted_foreground)
        })
        .child(
            lucide(LucideIcon::ChevronDown)
                .size(spacing * 4_f32)
                .text_color(theme.colors.muted_foreground),
        )
}

/// Creates the in-canvas select portal.
pub fn select_portal<T: Clone + Eq + 'static>() -> SelectPortal<T> {
    SelectPortal::new()
}

/// Creates a select positioner with the pinned 4px content offset.
pub fn select_positioner<T: Clone + Eq + 'static>(cx: &App) -> SelectPositioner<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    SelectPositioner::new()
        .side_offset(spacing * 1_f32)
        .style_with_state(|state, base| {
            base.when_some(state.anchor_width, |base, width| base.min_w(width))
        })
}

/// Creates the styled select popup.
pub fn select_popup<T: Clone + Eq + 'static>(cx: &App) -> SelectPopup<T> {
    let theme = UiTheme::read(cx).clone();
    SelectPopup::new().style_with_state(move |_state, base| popup_style(base, &theme))
}

/// Creates the styled select list.
pub fn select_list<T: Clone + Eq + 'static>(cx: &App) -> SelectList<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    SelectList::new().style_with_state(move |_state, base| base.p(spacing * 1_f32))
}

/// Creates a styled select group.
pub fn select_group<T: Clone + Eq + 'static>(cx: &App) -> SelectGroup<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    SelectGroup::new().style_with_state(move |_state, base| base.p(spacing * 1_f32))
}

/// Creates a styled select group label.
pub fn select_group_label<T: Clone + Eq + 'static>(cx: &App) -> SelectGroupLabel<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    SelectGroupLabel::new().style_with_state(move |_state, base| {
        base.px(spacing * 1.5_f32)
            .py(spacing * 1_f32)
            .font_family(theme.fonts.body.clone())
            .text_size(px(12.) * text_scale)
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
        .child(select_item_indicator(cx))
}

/// Creates the text region within a select item.
pub fn select_item_text<T: Clone + Eq + 'static>() -> SelectItemText<T> {
    SelectItemText::new().style_with_state(move |_state, base| base.flex_1().whitespace_nowrap())
}

/// Creates the selected-item check indicator.
pub fn select_item_indicator<T: Clone + Eq + 'static>(cx: &App) -> SelectItemIndicator<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    SelectItemIndicator::new()
        .keep_mounted(true)
        .style_with_state(move |state, base| {
            base.absolute()
                .right(spacing * 2_f32)
                .flex()
                .size(spacing * 4_f32)
                .items_center()
                .justify_center()
                .opacity(if state.selected { 1.0 } else { 0.0 })
        })
        .child(
            lucide(LucideIcon::Check)
                .size(spacing * 4_f32)
                .text_color(theme.colors.foreground),
        )
}

/// Creates a styled select separator.
pub fn select_separator(cx: &App) -> SelectSeparator {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    SelectSeparator::new().style_with_state(move |_state, base| {
        base.h(px(1.))
            .mx(spacing * -1_f32)
            .my(spacing * 1_f32)
            .bg(theme.colors.border)
    })
}

/// Creates the styled select scroll-up affordance.
pub fn select_scroll_up_arrow<T: Clone + Eq + 'static>(cx: &App) -> SelectScrollUpArrow<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let icon_color = theme.colors.muted_foreground;
    SelectScrollUpArrow::new()
        .style_with_state(move |_state, base| scroll_arrow_style(base, &theme))
        .child(
            lucide(LucideIcon::ChevronUp)
                .size(spacing * 4_f32)
                .text_color(icon_color),
        )
}

/// Creates the styled select scroll-down affordance.
pub fn select_scroll_down_arrow<T: Clone + Eq + 'static>(cx: &App) -> SelectScrollDownArrow<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let icon_color = theme.colors.muted_foreground;
    SelectScrollDownArrow::new()
        .style_with_state(move |_state, base| scroll_arrow_style(base, &theme))
        .child(
            lucide(LucideIcon::ChevronDown)
                .size(spacing * 4_f32)
                .text_color(icon_color),
        )
}

fn popup_style(base: Div, theme: &UiTheme) -> Div {
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    base.min_w(spacing * 36_f32)
        .max_h(spacing * 63_f32)
        .overflow_hidden()
        .rounded(theme.radius.lg)
        .bg(theme.colors.popover)
        .text_color(theme.colors.popover_foreground)
        .font_family(theme.fonts.body.clone())
        .text_size(px(14.) * text_scale)
        .border_1()
        .border_color(theme.colors.foreground.opacity(0.10))
        .shadow(theme.shadows.md.clone())
}

fn item_style(base: Div, highlighted: bool, disabled: bool, theme: &UiTheme) -> Div {
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    base.relative()
        .flex()
        .items_center()
        .gap(spacing * 1.5_f32)
        .rounded(theme.radius.sm)
        .py(spacing * 1_f32)
        .pr(spacing * 8_f32)
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

fn scroll_arrow_style(base: Div, theme: &UiTheme) -> Div {
    let spacing = theme.spacing.unit;
    base.flex()
        .h(spacing * 7_f32)
        .items_center()
        .justify_center()
        .bg(theme.colors.popover)
        .text_color(theme.colors.popover_foreground)
}
