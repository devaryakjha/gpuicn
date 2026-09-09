// Toolbar composition adapted from Base GPUI 64b22337.
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

use super::super::input::{CompositeInput, Input};
use std::rc::Rc;

use gpui::{
    App, Div, ElementId, FocusHandle, IntoElement, RenderOnce, SharedString, StyleRefinement,
    Styled, Window,
};

use super::{
    ToolbarContext, ToolbarInputStyleState, ToolbarMove, ToolbarOrientation,
    toolbar_wiring::move_focus,
};
use base_gpui::utils::direction::{HorizontalArrowKey, HorizontalDirection, current_direction};

/// A toolbar text field wrapping the ported `Input` component: text editing,
/// value state, and change callbacks come from the reused input. As a
/// composite item it occupies one roving slot; a plain forward/backward
/// arrow only leaves the input when the caret sits at the matching text
/// boundary with no selection, and roving focus entering the input selects
/// its whole text.
#[derive(IntoElement)]
pub struct ToolbarInput {
    id: ElementId,
    inner: Input,
    disabled: bool,
    focusable_when_disabled: bool,
    style_with_state: Option<Rc<dyn Fn(ToolbarInputStyleState, Div) -> Div + 'static>>,
    toolbar: Option<(ToolbarContext, usize, FocusHandle, bool)>,
}

impl Default for ToolbarInput {
    fn default() -> Self {
        let id = ElementId::from("toolbar-input");

        Self {
            inner: Input::new(id.clone()),
            id,
            disabled: false,
            focusable_when_disabled: true,
            style_with_state: None,
            toolbar: None,
        }
    }
}

impl Styled for ToolbarInput {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl RenderOnce for ToolbarInput {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (context, index, focus_handle, cascade_disabled) = match self.toolbar {
            Some((context, index, focus_handle, cascade_disabled)) => (
                Some(context),
                Some(index),
                Some(focus_handle),
                cascade_disabled,
            ),
            None => (None, None, None, false),
        };

        let disabled = self.disabled || cascade_disabled;
        let focusable = self.focusable_when_disabled;
        let orientation = context
            .as_ref()
            .map(|context| context.read(cx, |_runtime, props| props.orientation()))
            .unwrap_or(ToolbarOrientation::Horizontal);
        let tab_stop = match (&context, index) {
            (Some(context), Some(index)) => {
                context.read(cx, |runtime, _props| runtime.is_tab_stop(index))
                    && (!disabled || focusable)
            }
            _ => !disabled || focusable,
        };

        let mut composite = CompositeInput {
            focus: focus_handle,
            tab_stop: Some(tab_stop),
            select_all_on_focus: context.is_some(),
            register_field: true,
            ..Default::default()
        };
        if let Some(context) = context.clone()
            && orientation == ToolbarOrientation::Horizontal
        {
            let direction = current_direction();
            let backward_context = context.clone();
            composite.on_edge_left = Some(Rc::new(move |_, window, cx| {
                move_focus(
                    &backward_context,
                    horizontal_move(direction.horizontal_arrow(HorizontalArrowKey::Left)),
                    window,
                    cx,
                );
            }));
            composite.on_edge_right = Some(Rc::new(move |_, window, cx| {
                move_focus(
                    &context,
                    horizontal_move(direction.horizontal_arrow(HorizontalArrowKey::Right)),
                    window,
                    cx,
                );
            }));
        }
        let input = self.inner.disabled(disabled).composite(composite);

        let style_with_state = self.style_with_state;

        let field_state = base_gpui::field::current_field_context()
            .map(|context| context.read(cx, |runtime, props| runtime.root_state(props)));
        input.style_with_state(move |input_state, base| {
            let state = ToolbarInputStyleState::new(
                disabled,
                orientation,
                focusable,
                base_gpui::input::InputStyleState::new(input_state, field_state),
            );

            match style_with_state.as_ref() {
                Some(style_with_state) => style_with_state(state, base),
                None => base,
            }
        })
    }
}

impl ToolbarInput {
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn focusable_when_disabled(mut self, focusable_when_disabled: bool) -> Self {
        self.focusable_when_disabled = focusable_when_disabled;
        self
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        let id = id.into();
        self.inner = self.inner.id(id.clone());
        self.id = id;
        self
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.inner = self.inner.value(value);
        self
    }

    pub fn default_value(mut self, default_value: impl Into<SharedString>) -> Self {
        self.inner = self.inner.default_value(default_value);
        self
    }

    /// Sets the accessible name of the shared editor.
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.inner = self.inner.aria_label(label);
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.inner = self.inner.placeholder(placeholder);
        self
    }

    pub fn on_value_change(mut self, on_value_change: impl Fn(SharedString) + 'static) -> Self {
        self.inner = self.inner.on_value_change(on_value_change);
        self
    }

    pub fn on_enter(mut self, on_enter: impl Fn(SharedString) + 'static) -> Self {
        self.inner = self.inner.on_enter(on_enter);
        self
    }

    pub fn style_with_state(
        mut self,
        style: impl Fn(ToolbarInputStyleState, Div) -> Div + 'static,
    ) -> Self {
        self.style_with_state = Some(Rc::new(style));
        self
    }

    /// The input's own disabled prop, consumed by the toolbar child wiring
    /// when resolving the effective per-item disabled fact.
    pub fn own_disabled(&self) -> bool {
        self.disabled
    }

    /// The input's `focusable_when_disabled` flag, consumed by the toolbar
    /// child wiring for item metadata.
    pub fn own_focusable_when_disabled(&self) -> bool {
        self.focusable_when_disabled
    }

    /// The input's element id, consumed by the toolbar child wiring to key
    /// the roving focus handle.
    pub fn item_id(&self) -> &ElementId {
        &self.id
    }

    /// Attaches this input to a toolbar as a composite item. Called by the
    /// toolbar child wiring; not intended for direct use.
    pub fn with_toolbar(
        mut self,
        context: ToolbarContext,
        index: usize,
        focus_handle: FocusHandle,
        cascade_disabled: bool,
    ) -> Self {
        self.toolbar = Some((context, index, focus_handle, cascade_disabled));
        self
    }
}

fn horizontal_move(direction: HorizontalDirection) -> ToolbarMove {
    match direction {
        HorizontalDirection::Previous => ToolbarMove::Previous,
        HorizontalDirection::Next => ToolbarMove::Next,
    }
}
