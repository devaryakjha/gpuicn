//! The shadcn Nova Combobox visual port.
//!
//! Visual source: shadcn/ui 4.19.0 `combobox.tsx` and `style-nova.css` at
//! `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`. Interaction comes from the
//! pinned Base GPUI combobox primitives.

pub use base_gpui::combobox::{
    ComboboxAlign, ComboboxArrow, ComboboxBackdrop, ComboboxChip, ComboboxChipRemove,
    ComboboxChips, ComboboxClear, ComboboxCollection, ComboboxEmpty, ComboboxGroup,
    ComboboxGroupLabel, ComboboxIcon, ComboboxInput, ComboboxInputGroup, ComboboxItem,
    ComboboxItemIndicator, ComboboxLabel, ComboboxList, ComboboxPopup, ComboboxPortal,
    ComboboxPositioner, ComboboxRoot, ComboboxSelectionMode, ComboboxSeparator, ComboboxSide,
    ComboboxStatus, ComboboxTrigger, ComboboxValue,
};
use gpui::{
    App, BoxShadow, Div, ElementId, FontWeight, InteractiveElement as _, ParentElement as _,
    Styled, prelude::FluentBuilder as _, px,
};
use gpui_icons::{LucideIcon, lucide};

use super::theme::UiTheme;

/// Creates a combobox root with a caller-owned stable ID.
pub fn combobox_root<T: Clone + Eq + 'static>(id: impl Into<ElementId>) -> ComboboxRoot<T> {
    ComboboxRoot::new().id(id)
}

/// Creates the styled editable combobox input.
pub fn combobox_input<T: Clone + Eq + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> ComboboxInput<T> {
    let theme = UiTheme::read(cx).clone();
    ComboboxInput::new()
        .id(id)
        .style_with_state(move |state, base| {
            let ring = theme.colors.ring.alpha(0.50);
            base.h(px(32.))
                .rounded(theme.radius.base)
                .border_1()
                .border_color(theme.colors.input)
                .bg(theme.colors.background)
                .when(state.root.focused, |base| {
                    base.border_color(theme.colors.ring).shadow(vec![
                        BoxShadow::new(px(0.), px(0.), ring.into()).spread_radius(px(3.)),
                    ])
                })
                .when(state.root.disabled, |base| base.opacity(0.5))
        })
        .input_style_with_state(move |_state, base| {
            base.w_full()
                .h_full()
                .px(px(10.))
                .font_family(theme.fonts.body.clone())
                .text_size(px(14.))
                .text_color(theme.colors.foreground)
        })
}

/// Creates the borderless input used inside [`combobox_input_group`].
pub fn combobox_group_input<T: Clone + Eq + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> ComboboxInput<T> {
    let theme = UiTheme::read(cx).clone();
    ComboboxInput::new()
        .id(id)
        .style_with_state(move |_state, base| base.h(px(30.)).flex_1())
        .input_style_with_state(move |_state, base| {
            base.w_full()
                .h_full()
                .font_family(theme.fonts.body.clone())
                .text_size(px(14.))
                .text_color(theme.colors.foreground)
        })
}

/// Creates the styled input group used by chips and custom combobox layouts.
pub fn combobox_input_group<T: Clone + Eq + 'static>(cx: &App) -> ComboboxInputGroup<T> {
    let theme = UiTheme::read(cx).clone();
    ComboboxInputGroup::new().style_with_state(move |state, base| {
        base.flex()
            .items_center()
            .min_h(px(32.))
            .gap(px(4.))
            .rounded(theme.radius.base)
            .border_1()
            .border_color(theme.colors.input)
            .px(px(10.))
            .py(px(4.))
            .bg(if theme.mode == super::theme::ThemeMode::Dark {
                theme.colors.input.alpha(0.30)
            } else {
                theme.colors.background
            })
            .when(state.root.focused, |base| {
                base.border_color(theme.colors.ring).shadow(vec![
                    BoxShadow::new(px(0.), px(0.), theme.colors.ring.alpha(0.50).into())
                        .spread_radius(px(3.)),
                ])
            })
            .when(state.root.disabled, |base| base.opacity(0.5))
    })
}

/// Creates the styled combobox trigger with Nova's chevron.
pub fn combobox_trigger<T: Clone + Eq + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> ComboboxTrigger<T> {
    let theme = UiTheme::read(cx).clone();
    let icon_color = theme.colors.muted_foreground;
    ComboboxTrigger::new()
        .id(id)
        .style_with_state(move |state, base| {
            base.flex()
                .size(px(24.))
                .items_center()
                .justify_center()
                .rounded(px(6.))
                .text_color(theme.colors.muted_foreground)
                .when(state.root.open, |base| base.bg(theme.colors.muted))
                .when(!state.root.disabled, |base| {
                    base.hover(move |style| style.bg(theme.colors.muted))
                })
        })
        .child(
            lucide(LucideIcon::ChevronDown)
                .size(px(16.))
                .text_color(icon_color),
        )
}

/// Creates the styled clear control for a combobox.
pub fn combobox_clear<T: Clone + Eq + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> ComboboxClear<T> {
    let theme = UiTheme::read(cx).clone();
    ComboboxClear::new()
        .id(id)
        .style_with_state(move |state, base| {
            base.flex()
                .size(px(24.))
                .items_center()
                .justify_center()
                .rounded(px(6.))
                .text_color(theme.colors.muted_foreground)
                .when(!state.disabled, |base| {
                    base.hover(move |style| style.bg(theme.colors.muted))
                })
                .when(state.disabled, |base| base.opacity(0.5))
        })
        .child(
            lucide(LucideIcon::X)
                .size(px(14.))
                .text_color(theme.colors.muted_foreground),
        )
}

/// Creates the in-canvas combobox portal.
pub fn combobox_portal<T: Clone + Eq + 'static>() -> ComboboxPortal<T> {
    ComboboxPortal::new()
}

/// Creates a combobox positioner with the pinned 6px content offset.
pub fn combobox_positioner<T: Clone + Eq + 'static>() -> ComboboxPositioner<T> {
    ComboboxPositioner::new().side_offset(px(6.))
}

/// Creates the styled combobox popup.
pub fn combobox_popup<T: Clone + Eq + 'static>(cx: &App) -> ComboboxPopup<T> {
    let theme = UiTheme::read(cx).clone();
    ComboboxPopup::new().style_with_state(move |_state, base| popup_style(base, &theme))
}

/// Creates the styled combobox list.
pub fn combobox_list<T: Clone + Eq + 'static>() -> ComboboxList<T> {
    ComboboxList::new()
        .style_with_state(move |_state, base| base.max_h(px(252.)).overflow_hidden().p(px(4.)))
}

/// Creates a styled combobox item.
pub fn combobox_item<T: Clone + Eq + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> ComboboxItem<T> {
    let theme = UiTheme::read(cx).clone();
    ComboboxItem::new()
        .id(id)
        .style_with_state(move |state, base| {
            base.relative()
                .flex()
                .items_center()
                .gap(px(8.))
                .rounded(px(6.))
                .py(px(4.))
                .pr(px(32.))
                .pl(px(6.))
                .font_family(theme.fonts.body.clone())
                .text_size(px(14.))
                .text_color(theme.colors.popover_foreground)
                .when(state.highlighted, |base| {
                    base.bg(theme.colors.accent)
                        .text_color(theme.colors.accent_foreground)
                })
                .when(state.disabled, |base| base.opacity(0.5))
        })
        .child(combobox_item_indicator(cx))
}

/// Creates the selected-item check indicator.
pub fn combobox_item_indicator<T: Clone + Eq + 'static>(cx: &App) -> ComboboxItemIndicator<T> {
    let theme = UiTheme::read(cx).clone();
    ComboboxItemIndicator::new()
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

/// Creates a styled combobox group.
pub fn combobox_group<T: Clone + Eq + 'static>() -> ComboboxGroup<T> {
    ComboboxGroup::new().style_with_state(move |_state, base| base.p(px(4.)))
}

/// Creates a styled combobox group label.
pub fn combobox_group_label<T: Clone + Eq + 'static>(cx: &App) -> ComboboxGroupLabel<T> {
    let theme = UiTheme::read(cx).clone();
    ComboboxGroupLabel::new().style_with_state(move |_state, base| {
        base.px(px(8.))
            .py(px(6.))
            .font_family(theme.fonts.body.clone())
            .text_size(px(12.))
            .text_color(theme.colors.muted_foreground)
    })
}

/// Creates the styled empty state container.
pub fn combobox_empty<T: Clone + Eq + 'static>(cx: &App) -> ComboboxEmpty<T> {
    let theme = UiTheme::read(cx).clone();
    ComboboxEmpty::new().style_with_state(move |_state, base| {
        base.flex()
            .w_full()
            .justify_center()
            .py(px(8.))
            .font_family(theme.fonts.body.clone())
            .text_size(px(14.))
            .text_color(theme.colors.muted_foreground)
    })
}

/// Creates a styled combobox separator.
pub fn combobox_separator(cx: &App) -> ComboboxSeparator {
    let theme = UiTheme::read(cx).clone();
    ComboboxSeparator::new().style_with_state(move |_state, base| {
        base.h(px(1.))
            .mx(px(-4.))
            .my(px(4.))
            .bg(theme.colors.border)
    })
}

/// Creates the styled container for selected chips.
pub fn combobox_chips<T: Clone + Eq + 'static>(cx: &App) -> ComboboxChips<T> {
    let theme = UiTheme::read(cx).clone();
    ComboboxChips::new().style_with_state(move |_state, base| {
        base.flex()
            .flex_wrap()
            .items_center()
            .gap(px(4.))
            .font_family(theme.fonts.body.clone())
    })
}

/// Creates one styled selected-value chip.
pub fn combobox_chip<T: Clone + Eq + 'static>(cx: &App) -> ComboboxChip<T> {
    let theme = UiTheme::read(cx).clone();
    ComboboxChip::new().style_with_state(move |state, base| {
        base.flex()
            .items_center()
            .gap(px(4.))
            .h(px(21.))
            .rounded(px(4.))
            .px(px(6.))
            .bg(theme.colors.muted)
            .font_family(theme.fonts.body.clone())
            .font_weight(FontWeight::MEDIUM)
            .text_size(px(12.))
            .text_color(theme.colors.foreground)
            .when(state.highlighted, |base| base.bg(theme.colors.accent))
            .when(state.disabled, |base| base.opacity(0.5))
    })
}

/// Creates the styled remove control for a selected-value chip.
pub fn combobox_chip_remove<T: Clone + Eq + 'static>(cx: &App) -> ComboboxChipRemove<T> {
    let theme = UiTheme::read(cx).clone();
    ComboboxChipRemove::new()
        .style_with_state(move |state, base| {
            base.flex()
                .size(px(16.))
                .items_center()
                .justify_center()
                .rounded(px(3.))
                .text_color(theme.colors.muted_foreground)
                .when(!state.disabled, |base| {
                    base.hover(move |style| style.bg(theme.colors.background))
                })
        })
        .child(
            lucide(LucideIcon::X)
                .size(px(12.))
                .text_color(theme.colors.muted_foreground),
        )
}

/// Creates the styled text value shown by non-chip comboboxes.
pub fn combobox_value<T: Clone + Eq + 'static>() -> ComboboxValue<T> {
    ComboboxValue::new().style_with_state(move |_state, base| base.flex_1())
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
