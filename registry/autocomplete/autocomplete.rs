//! The Base UI Autocomplete visual port using the shadcn Nova Combobox skin.
//!
//! Base GPUI implements Autocomplete by reusing its Combobox parts. shadcn/ui
//! 4.19.0 only ships the Nova wrapper for Combobox, so this file applies that
//! same editable-list treatment without changing Base GPUI autocomplete mode
//! or selection behavior.

pub use base_gpui::autocomplete::{
    AutocompleteArrow, AutocompleteBackdrop, AutocompleteClear, AutocompleteCollection,
    AutocompleteEmpty, AutocompleteGroup, AutocompleteGroupLabel, AutocompleteIcon,
    AutocompleteInput, AutocompleteInputGroup, AutocompleteItem, AutocompleteList,
    AutocompleteMode, AutocompletePopup, AutocompletePortal, AutocompletePositioner,
    AutocompleteRoot, AutocompleteSeparator, AutocompleteStatus, AutocompleteTrigger,
    AutocompleteValue,
};
use gpui::{App, ElementId, Styled, px};

use super::combobox;

/// Creates an autocomplete root with a caller-owned stable ID.
pub fn autocomplete_root<T: Clone + Eq + 'static>(id: impl Into<ElementId>) -> AutocompleteRoot<T> {
    AutocompleteRoot::new().id(id)
}

/// Creates the styled editable autocomplete input.
pub fn autocomplete_input<T: Clone + Eq + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> AutocompleteInput<T> {
    combobox::combobox_input(id, cx)
}

/// Creates the styled input group for custom autocomplete layouts.
pub fn autocomplete_input_group<T: Clone + Eq + 'static>(cx: &App) -> AutocompleteInputGroup<T> {
    combobox::combobox_input_group(cx)
}

/// Creates the styled autocomplete trigger.
pub fn autocomplete_trigger<T: Clone + Eq + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> AutocompleteTrigger<T> {
    combobox::combobox_trigger(id, cx)
}

/// Creates the styled autocomplete clear control.
pub fn autocomplete_clear<T: Clone + Eq + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> AutocompleteClear<T> {
    combobox::combobox_clear(id, cx)
}

/// Creates the in-canvas autocomplete portal.
pub fn autocomplete_portal<T: Clone + Eq + 'static>() -> AutocompletePortal<T> {
    combobox::combobox_portal()
}

/// Creates an autocomplete positioner with the pinned Nova offset.
pub fn autocomplete_positioner<T: Clone + Eq + 'static>() -> AutocompletePositioner<T> {
    combobox::combobox_positioner()
}

/// Creates the styled autocomplete popup.
pub fn autocomplete_popup<T: Clone + Eq + 'static>(cx: &App) -> AutocompletePopup<T> {
    combobox::combobox_popup(cx)
}

/// Creates the styled autocomplete list.
pub fn autocomplete_list<T: Clone + Eq + 'static>() -> AutocompleteList<T> {
    combobox::combobox_list()
}

/// Creates a styled autocomplete item.
pub fn autocomplete_item<T: Clone + Eq + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> AutocompleteItem<T> {
    combobox::combobox_item(id, cx)
}

/// Creates the selected-item check indicator.
pub fn autocomplete_item_indicator<T: Clone + Eq + 'static>(
    cx: &App,
) -> base_gpui::combobox::ComboboxItemIndicator<T> {
    combobox::combobox_item_indicator(cx)
}

/// Creates a styled autocomplete group.
pub fn autocomplete_group<T: Clone + Eq + 'static>() -> AutocompleteGroup<T> {
    combobox::combobox_group()
}

/// Creates a styled autocomplete group label.
pub fn autocomplete_group_label<T: Clone + Eq + 'static>(cx: &App) -> AutocompleteGroupLabel<T> {
    combobox::combobox_group_label(cx)
}

/// Creates the styled autocomplete empty state container.
pub fn autocomplete_empty<T: Clone + Eq + 'static>(cx: &App) -> AutocompleteEmpty<T> {
    combobox::combobox_empty(cx)
}

/// Creates a styled autocomplete separator.
pub fn autocomplete_separator(cx: &App) -> AutocompleteSeparator {
    combobox::combobox_separator(cx)
}

/// Creates the visual autocomplete value mirror.
pub fn autocomplete_value<T: Clone + Eq + 'static>() -> AutocompleteValue<T> {
    AutocompleteValue::new().flex_1().text_size(px(14.))
}
