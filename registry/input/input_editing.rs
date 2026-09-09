// Native text layout and IME handling adapted from Base GPUI 64b22337.
// MIT License
//
// Copyright (c) 2026 Luke Tandjung
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use super::{
    InputDeleteToEnd, InputDeleteToStart, InputDeleteWordLeft, InputDeleteWordRight, InputRedo,
    InputSelectEnd, InputSelectHome, InputSelectWordLeft, InputSelectWordRight, InputUndo,
    InputWordLeft, InputWordRight,
};
use std::{collections::VecDeque, ops::Range};
use web_time::Instant;

use gpui::{
    ClipboardItem, Context, EntityInputHandler, FocusHandle, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, Point, ShapedLine, SharedString, Subscription, UTF16Selection, Window,
};
use unicode_segmentation::UnicodeSegmentation;

use base_gpui::primitives::input::{
    InputBackspace, InputCopy, InputCut, InputDelete, InputEnd, InputEnter, InputHome, InputLeft,
    InputPaste, InputRight, InputSelectAll, InputSelectLeft, InputSelectRight, InputStyleState,
};

#[derive(Clone)]
struct Snapshot {
    value: SharedString,
    selection: Range<usize>,
    reversed: bool,
}

pub struct Editing {
    pub(super) composite: super::CompositeInput,
    focus_handle: FocusHandle,
    value: SharedString,
    initial_value: SharedString,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    last_layout: Option<ShapedLine>,
    last_bounds: Option<gpui::Bounds<gpui::Pixels>>,
    last_scroll_offset: gpui::Pixels,
    selecting: bool,
    disabled: bool,
    read_only: bool,
    required: bool,
    controlled: bool,
    on_value_change: Option<super::ChangeHandler>,
    on_enter: Option<super::ChangeHandler>,
    undo: VecDeque<Snapshot>,
    redo: VecDeque<Snapshot>,
    last_insertion: Option<(Instant, usize)>,
    _subscriptions: Vec<Subscription>,
}

impl Editing {
    pub fn new(value: SharedString, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self::new_with_focus_handle(value, cx.focus_handle(), window, cx)
    }

    pub fn new_with_focus_handle(
        value: SharedString,
        focus_handle: FocusHandle,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let subscriptions = vec![
            cx.on_focus(&focus_handle, window, Self::on_focus),
            cx.on_blur(&focus_handle, window, Self::on_blur),
        ];
        let cursor = value.len();

        Self {
            composite: super::CompositeInput::default(),
            focus_handle,
            value: value.clone(),
            initial_value: value,
            selected_range: cursor..cursor,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
            last_scroll_offset: gpui::px(0.0),
            selecting: false,
            disabled: false,
            read_only: false,
            required: false,
            controlled: false,
            on_value_change: None,
            on_enter: None,
            undo: VecDeque::new(),
            redo: VecDeque::new(),
            last_insertion: None,
            _subscriptions: subscriptions,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn sync_props(
        &mut self,
        controlled_value: Option<SharedString>,
        disabled: bool,
        read_only: bool,
        required: bool,
        on_value_change: Option<super::ChangeHandler>,
        on_enter: Option<super::ChangeHandler>,
        cx: &mut Context<Self>,
    ) {
        let mut changed = false;
        self.controlled = controlled_value.is_some();
        if let Some(value) = controlled_value
            && self.value != value
        {
            self.undo.clear();
            self.redo.clear();
            self.last_insertion = None;
            self.value = value;
            self.clamp_selection_to_value();
            self.marked_range = None;
            changed = true;
        }
        changed |= self.disabled != disabled;
        changed |= self.read_only != read_only;
        changed |= self.required != required;
        self.disabled = disabled;
        self.read_only = read_only;
        self.required = required;
        self.on_value_change = on_value_change;
        self.on_enter = on_enter;

        if changed {
            cx.notify();
        }
    }

    pub fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }

    pub fn value(&self) -> SharedString {
        self.value.clone()
    }

    pub fn style_state(&self, window: &Window, valid: Option<bool>) -> InputStyleState {
        InputStyleState::new(
            self.value.clone(),
            self.disabled,
            self.read_only,
            self.required,
            self.focus_handle.is_focused(window),
            self.dirty(),
            self.controlled,
            valid,
        )
    }

    pub fn dirty(&self) -> bool {
        self.value != self.initial_value
    }

    pub fn disabled(&self) -> bool {
        self.disabled
    }

    pub fn selected_range(&self) -> Range<usize> {
        self.selected_range.clone()
    }

    pub fn marked_range(&self) -> Option<Range<usize>> {
        self.marked_range.clone()
    }

    pub fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    pub fn set_last_layout(
        &mut self,
        layout: ShapedLine,
        bounds: gpui::Bounds<gpui::Pixels>,
        scroll_offset: gpui::Pixels,
        _cx: &mut Context<Self>,
    ) {
        self.last_layout = Some(layout);
        self.last_bounds = Some(bounds);
        self.last_scroll_offset = scroll_offset;
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            value: self.value.clone(),
            selection: self.selected_range.clone(),
            reversed: self.selection_reversed,
        }
    }

    fn push_history(history: &mut VecDeque<Snapshot>, snapshot: Snapshot) {
        history.push_back(snapshot);
        // ponytail: 100 edits or 8 MiB of snapshots; use text deltas for large-document editing.
        let mut bytes: usize = history.iter().map(|entry| entry.value.len()).sum();
        while history.len() > 100 || bytes > 8 * 1024 * 1024 {
            bytes -= history.pop_front().unwrap().value.len();
        }
    }

    fn restore(&mut self, snapshot: Snapshot, window: &mut Window, cx: &mut Context<Self>) {
        self.value = snapshot.value;
        self.selected_range = snapshot.selection;
        self.selection_reversed = snapshot.reversed;
        self.marked_range = None;
        self.last_insertion = None;
        if let Some(handler) = self.on_value_change.clone() {
            handler(self.value.clone(), window, cx);
        }
        cx.notify();
    }

    pub fn undo(&mut self, _: &InputUndo, window: &mut Window, cx: &mut Context<Self>) {
        if !self.can_edit() {
            return;
        }
        if let Some(snapshot) = self.undo.pop_back() {
            let current = self.snapshot();
            Self::push_history(&mut self.redo, current);
            self.restore(snapshot, window, cx);
        }
    }

    pub fn redo(&mut self, _: &InputRedo, window: &mut Window, cx: &mut Context<Self>) {
        if !self.can_edit() {
            return;
        }
        if let Some(snapshot) = self.redo.pop_back() {
            let current = self.snapshot();
            Self::push_history(&mut self.undo, current);
            self.restore(snapshot, window, cx);
        }
    }

    fn word_boundary(&self, forward: bool) -> usize {
        let cursor = self.cursor_offset();
        if forward {
            self.value
                .unicode_word_indices()
                .map(|(start, word)| start + word.len())
                .find(|end| *end > cursor)
                .unwrap_or(self.value.len())
        } else {
            self.value
                .unicode_word_indices()
                .map(|(start, _)| start)
                .take_while(|start| *start < cursor)
                .last()
                .unwrap_or(0)
        }
    }

    pub fn word_left(&mut self, _: &InputWordLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.word_boundary(false), cx);
    }
    pub fn word_right(&mut self, _: &InputWordRight, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.word_boundary(true), cx);
    }
    pub fn select_word_left(
        &mut self,
        _: &InputSelectWordLeft,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_to(self.word_boundary(false), cx);
    }
    pub fn select_word_right(
        &mut self,
        _: &InputSelectWordRight,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_to(self.word_boundary(true), cx);
    }
    pub fn select_home(&mut self, _: &InputSelectHome, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(0, cx);
    }
    pub fn select_end(&mut self, _: &InputSelectEnd, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.value.len(), cx);
    }
    fn delete_to(&mut self, offset: usize, window: &mut Window, cx: &mut Context<Self>) {
        if !self.can_edit() {
            return;
        }
        if self.selected_range.is_empty() {
            self.select_to(offset, cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }
    pub fn delete_word_left(
        &mut self,
        _: &InputDeleteWordLeft,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.delete_to(self.word_boundary(false), window, cx);
    }
    pub fn delete_word_right(
        &mut self,
        _: &InputDeleteWordRight,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.delete_to(self.word_boundary(true), window, cx);
    }
    pub fn delete_to_start(
        &mut self,
        _: &InputDeleteToStart,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.delete_to(0, window, cx);
    }
    pub fn delete_to_end(
        &mut self,
        _: &InputDeleteToEnd,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.delete_to(self.value.len(), window, cx);
    }

    pub fn backspace(&mut self, _: &InputBackspace, window: &mut Window, cx: &mut Context<Self>) {
        if !self.can_edit() {
            return;
        }
        if self.selected_range.is_empty() {
            let previous = self.previous_boundary(self.cursor_offset());
            if previous == self.cursor_offset() {
                window.play_system_bell();
                return;
            }
            self.select_to(previous, cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    pub fn delete(&mut self, _: &InputDelete, window: &mut Window, cx: &mut Context<Self>) {
        if !self.can_edit() {
            return;
        }
        if self.selected_range.is_empty() {
            let next = self.next_boundary(self.cursor_offset());
            if next == self.cursor_offset() {
                window.play_system_bell();
                return;
            }
            self.select_to(next, cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    pub fn left(&mut self, _: &InputLeft, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            if self.cursor_offset() == 0
                && let Some(handler) = self.composite.on_edge_left.clone()
            {
                handler(self.value.clone(), window, cx);
                return;
            }
            self.move_to(self.previous_boundary(self.cursor_offset()), cx);
        } else {
            self.move_to(self.selected_range.start, cx);
        }
    }

    pub fn right(&mut self, _: &InputRight, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            if self.cursor_offset() == self.value.len()
                && let Some(handler) = self.composite.on_edge_right.clone()
            {
                handler(self.value.clone(), window, cx);
                return;
            }
            self.move_to(self.next_boundary(self.cursor_offset()), cx);
        } else {
            self.move_to(self.selected_range.end, cx);
        }
    }

    pub fn select_left(&mut self, _: &InputSelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.previous_boundary(self.cursor_offset()), cx);
    }

    pub fn select_right(&mut self, _: &InputSelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.next_boundary(self.cursor_offset()), cx);
    }

    pub fn select_all(&mut self, _: &InputSelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.last_insertion = None;
        self.selected_range = 0..self.value.len();
        self.selection_reversed = false;
        cx.notify();
    }

    pub fn home(&mut self, _: &InputHome, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(handler) = self.composite.on_home.clone() {
            handler(self.value.clone(), window, cx);
            return;
        }
        self.move_to(0, cx);
    }

    pub fn end(&mut self, _: &InputEnd, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(handler) = self.composite.on_end.clone() {
            handler(self.value.clone(), window, cx);
            return;
        }
        self.move_to(self.value.len(), cx);
    }

    pub fn copy(&mut self, _: &InputCopy, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.value[self.selected_range.clone()].to_string(),
            ));
        }
    }

    pub fn cut(&mut self, _: &InputCut, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            return;
        }
        cx.write_to_clipboard(ClipboardItem::new_string(
            self.value[self.selected_range.clone()].to_string(),
        ));
        if self.can_edit() {
            self.replace_text_in_range(None, "", window, cx);
        }
    }

    pub fn paste(&mut self, _: &InputPaste, window: &mut Window, cx: &mut Context<Self>) {
        if !self.can_edit() {
            return;
        }
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.last_insertion = None;
            self.replace_text_in_range(None, &normalize_single_line(&text), window, cx);
            self.last_insertion = None;
        }
    }

    pub fn enter(&mut self, _: &InputEnter, window: &mut Window, cx: &mut Context<Self>) {
        if !self.disabled
            && self.marked_range.is_none()
            && let Some(on_enter) = self.on_enter.clone()
        {
            on_enter(self.value.clone(), window, cx);
        }
    }

    pub fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }

        self.focus_handle.focus(window, cx);
        self.selecting = true;
        let offset = self.index_for_mouse_position(event.position);
        if event.click_count >= 3 {
            self.select_all(&InputSelectAll, window, cx);
        } else if event.click_count == 2 {
            // Word hit-testing uses the glyph under the pointer; caret placement
            // uses its nearest boundary and can otherwise select adjacent space.
            let offset = self
                .last_bounds
                .zip(self.last_layout.as_ref())
                .and_then(|(bounds, line)| {
                    line.index_for_x(event.position.x - bounds.left() - self.last_scroll_offset)
                })
                .unwrap_or_else(|| self.previous_boundary(self.value.len()));
            let range = self
                .value
                .split_word_bound_indices()
                .find_map(|(start, word)| {
                    (offset >= start && offset < start + word.len())
                        .then_some(start..start + word.len())
                })
                .unwrap_or(offset..offset);
            self.move_to(range.start, cx);
            self.select_to(range.end, cx);
        } else if event.modifiers.shift {
            self.select_to(offset, cx);
        } else {
            self.move_to(offset, cx);
        }
    }

    pub fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.selecting = false;
        cx.notify();
    }

    pub fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selecting {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        }
    }

    fn on_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.composite.select_all_on_focus && !self.disabled {
            self.select_all(&InputSelectAll, window, cx);
        }
        if let Some(handler) = self.composite.on_focus_change.clone() {
            handler(true, window, cx);
        }
        cx.notify();
    }

    fn on_blur(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(handler) = self.composite.on_focus_change.clone() {
            handler(false, window, cx);
        }
        self.last_insertion = None;
        self.selecting = false;
        cx.notify();
    }

    fn can_edit(&self) -> bool {
        !self.disabled && !self.read_only
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.last_insertion = None;
        let offset = self.clamp_offset(offset);
        self.selected_range = offset..offset;
        self.selection_reversed = false;
        cx.notify();
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.last_insertion = None;
        let offset = self.clamp_offset(offset);
        if self.selection_reversed {
            self.selected_range.start = offset;
        } else {
            self.selected_range.end = offset;
        }
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        cx.notify();
    }

    fn index_for_mouse_position(&self, position: Point<gpui::Pixels>) -> usize {
        if self.value.is_empty() {
            return 0;
        }

        let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return self.value.len();
        };

        if position.x <= bounds.left() {
            return 0;
        }
        if position.x >= bounds.right() && self.last_scroll_offset == gpui::px(0.0) {
            return self.value.len();
        }
        line.closest_index_for_x(position.x - bounds.left() - self.last_scroll_offset)
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.value
            .grapheme_indices(true)
            .rev()
            .find_map(|(index, _)| (index < offset).then_some(index))
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.value
            .grapheme_indices(true)
            .find_map(|(index, _)| (index > offset).then_some(index))
            .unwrap_or(self.value.len())
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;

        for ch in self.value.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }

        utf8_offset
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;

        for ch in self.value.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }

        utf16_offset
    }

    fn range_from_utf16(&self, range: &Range<usize>) -> Range<usize> {
        let start = self.offset_from_utf16(range.start);
        let end = self.offset_from_utf16(range.end);
        start.min(end)..start.max(end)
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn clamp_offset(&self, offset: usize) -> usize {
        let mut offset = offset.min(self.value.len());
        while !self.value.is_char_boundary(offset) {
            offset -= 1;
        }
        offset
    }

    fn clamp_selection_to_value(&mut self) {
        let start = self.clamp_offset(self.selected_range.start);
        let end = self.clamp_offset(self.selected_range.end);
        self.selected_range = start.min(end)..start.max(end);
    }

    fn replacement_range(&self, range_utf16: Option<Range<usize>>) -> Range<usize> {
        range_utf16
            .as_ref()
            .map(|range| self.range_from_utf16(range))
            .or_else(|| self.marked_range.clone())
            .unwrap_or_else(|| self.selected_range.clone())
    }

    fn replace_selected_text(
        &mut self,
        range: Range<usize>,
        new_text: &str,
        mark: Option<Option<Range<usize>>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let new_text = normalize_single_line(new_text);
        let mut next = String::new();
        next.push_str(&self.value[..range.start]);
        next.push_str(&new_text);
        next.push_str(&self.value[range.end..]);
        let next_value = SharedString::from(next);
        if self.marked_range.is_none() && (next_value != self.value || mark.is_some()) {
            let insertion = mark.is_none() && new_text.graphemes(true).count() == 1;
            let now = Instant::now();
            let coalesce = insertion
                && range.is_empty()
                && self.last_insertion.is_some_and(|(time, end)| {
                    end == range.start && now.duration_since(time).as_secs_f32() < 1.0
                });
            if !coalesce {
                let snapshot = self.snapshot();
                Self::push_history(&mut self.undo, snapshot);
            }
            self.redo.clear();
            self.last_insertion = insertion.then_some((now, range.start + new_text.len()));
        } else if mark.is_some() {
            self.last_insertion = None;
        }
        let next_offset = range.start + new_text.len();

        self.value = next_value;
        match mark {
            Some(new_selected_range_utf16) => {
                if new_text.is_empty() {
                    self.marked_range = None;
                    self.selected_range = range.start..range.start;
                } else {
                    let marked_range = range.start..range.start + new_text.len();
                    self.marked_range = Some(marked_range.clone());
                    self.selected_range = new_selected_range_utf16
                        .as_ref()
                        .map(|range_utf16| {
                            let offset = |units: usize| {
                                let mut count = 0;
                                new_text
                                    .char_indices()
                                    .find_map(|(index, ch)| {
                                        if count >= units {
                                            return Some(index);
                                        }
                                        count += ch.len_utf16();
                                        None
                                    })
                                    .unwrap_or(new_text.len())
                            };
                            let start = offset(range_utf16.start);
                            let end = offset(range_utf16.end);
                            marked_range.start + start.min(end)..marked_range.start + start.max(end)
                        })
                        .unwrap_or_else(|| marked_range.end..marked_range.end);
                }
            }
            None => {
                self.marked_range = None;
                self.selected_range = next_offset..next_offset;
            }
        }
        self.selection_reversed = false;
        if let Some(on_value_change) = self.on_value_change.clone() {
            on_value_change(self.value.clone(), window, cx);
        }
        cx.notify();
    }
}

impl EntityInputHandler for Editing {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        adjusted_range: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        adjusted_range.replace(self.range_to_utf16(&range));
        Some(self.value[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        ignore_disabled_input: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        if !ignore_disabled_input && self.disabled {
            return None;
        }

        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.marked_range = None;
        cx.notify();
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.can_edit() {
            return;
        }

        let range = self.replacement_range(range_utf16);
        self.replace_selected_text(range, text, None, window, cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.can_edit() {
            return;
        }

        let range = self.replacement_range(range_utf16);
        self.replace_selected_text(range, new_text, Some(new_selected_range), window, cx);
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        element_bounds: gpui::Bounds<gpui::Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<gpui::Bounds<gpui::Pixels>> {
        let layout = self.last_layout.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);

        Some(gpui::Bounds::from_corners(
            gpui::point(
                element_bounds.left() + self.last_scroll_offset + layout.x_for_index(range.start),
                element_bounds.top(),
            ),
            gpui::point(
                element_bounds.left() + self.last_scroll_offset + layout.x_for_index(range.end),
                element_bounds.bottom(),
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<gpui::Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        let bounds = self.last_bounds?;
        let layout = self.last_layout.as_ref()?;
        if !bounds.contains(&point) {
            return None;
        }

        let utf8_index = layout.index_for_x(point.x - bounds.left() - self.last_scroll_offset)?;
        Some(self.offset_to_utf16(utf8_index))
    }

    fn accepts_text_input(&self, _: &mut Window, _: &mut Context<Self>) -> bool {
        self.can_edit()
    }
}

fn normalize_single_line(text: &str) -> String {
    text.replace(['\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{AppContext as _, Entity, IntoElement, Render, TestAppContext, div};

    struct View(Entity<Editing>);
    impl Render for View {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
        }
    }

    #[gpui::test]
    fn unicode_ime_commit_is_one_undoable_edit(cx: &mut TestAppContext) {
        let window =
            cx.add_window(|window, cx| View(cx.new(|cx| Editing::new("🦀a".into(), window, cx))));
        window
            .update(cx, |view, window, cx| {
                view.0.update(cx, |input, cx| {
                    input.replace_and_mark_text_in_range(None, "日本", Some(1..2), window, cx);
                    assert_eq!(input.value(), "🦀a日本");
                    assert_eq!(
                        input.selected_text_range(false, window, cx).unwrap().range,
                        4..5
                    );
                    input.replace_and_mark_text_in_range(None, "日本語", Some(3..3), window, cx);
                    input.replace_text_in_range(None, "日本語", window, cx);
                    assert!(input.marked_range().is_none());
                    assert_eq!(input.value(), "🦀a日本語");
                    input.undo(&InputUndo, window, cx);
                    assert_eq!(input.value(), "🦀a");
                    input.redo(&InputRedo, window, cx);
                    assert_eq!(input.value(), "🦀a日本語");
                    // A UTF-16 replacement spans the complete supplementary character.
                    input.replace_text_in_range(Some(0..2), "é", window, cx);
                    assert_eq!(input.value(), "éa日本語");
                    input.undo(&InputUndo, window, cx);
                    assert_eq!(input.value(), "🦀a日本語");
                    input.sync_props(Some("かな".into()), false, false, false, None, None, cx);
                    input.select_all(&InputSelectAll, window, cx);
                    input.replace_and_mark_text_in_range(None, "かな", Some(0..2), window, cx);
                    input.replace_and_mark_text_in_range(None, "カナ", Some(2..2), window, cx);
                    input.replace_text_in_range(None, "カナ", window, cx);
                    input.undo(&InputUndo, window, cx);
                    assert_eq!(input.value(), "かな");
                    input.redo(&InputRedo, window, cx);
                    assert_eq!(input.value(), "カナ");
                    input.read_only = true;
                    input.undo(&InputUndo, window, cx);
                    input.replace_text_in_range(None, "ignored", window, cx);
                    assert_eq!(input.value(), "カナ");
                })
            })
            .unwrap();
    }
}
