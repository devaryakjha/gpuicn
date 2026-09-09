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

use gpui::{
    App, Bounds, ContentMask, Element, ElementId, ElementInputHandler, Entity, GlobalElementId,
    InspectorElementId, IntoElement, LayoutId, PaintQuad, Pixels, ShapedLine, SharedString, Style,
    TextAlign, TextRun, UnderlineStyle, Window, fill, point, px, size,
};

use super::{super::theme::UiTheme, editing::Editing};

pub struct InputTextElement {
    state: Entity<Editing>,
    placeholder: SharedString,
}

impl InputTextElement {
    pub fn new(state: Entity<Editing>, placeholder: SharedString) -> Self {
        Self { state, placeholder }
    }
}

pub struct InputTextPrepaintState {
    line: Option<ShapedLine>,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
    scroll_offset: Pixels,
}

impl IntoElement for InputTextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for InputTextElement {
    type RequestLayoutState = ();
    type PrepaintState = InputTextPrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = gpui::relative(1.0).into();
        style.size.height = window.line_height().into();

        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.state.read(cx);
        let value = input.value();
        let selected_range = input.selected_range();
        let marked_range = input.marked_range();
        let cursor = input.cursor_offset();
        let style = window.text_style();
        let disabled = input.disabled();

        let (display_text, text_color) = if value.is_empty() {
            (
                self.placeholder.clone(),
                UiTheme::read(cx).colors.muted_foreground.into(),
            )
        } else {
            (value.clone(), style.color)
        };
        let text_color = if disabled {
            text_color.opacity(0.5)
        } else {
            text_color
        };

        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let runs = if !value.is_empty() {
            if let Some(marked_range) = marked_range.as_ref() {
                vec![
                    TextRun {
                        len: marked_range.start,
                        ..run.clone()
                    },
                    TextRun {
                        len: marked_range.end - marked_range.start,
                        underline: Some(UnderlineStyle {
                            color: Some(run.color),
                            thickness: px(1.0),
                            wavy: false,
                        }),
                        ..run.clone()
                    },
                    TextRun {
                        len: display_text.len() - marked_range.end,
                        ..run.clone()
                    },
                ]
                .into_iter()
                .filter(|run| run.len > 0)
                .collect()
            } else {
                vec![run]
            }
        } else {
            vec![run]
        };

        let font_size = style.font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(display_text, font_size, &runs, None);
        let cursor_x = line.x_for_index(cursor);
        // Keep the cursor visible when the shaped line is wider than the
        // element: shift painting left so the cursor stays inside bounds.
        let visible_width = bounds.right() - bounds.left();
        let scroll_offset = if cursor_x + px(2.0) > visible_width {
            visible_width - cursor_x - px(2.0)
        } else {
            px(0.0)
        };
        let (selection, cursor) = if !value.is_empty() && !selected_range.is_empty() {
            (
                Some(fill(
                    Bounds::from_corners(
                        point(
                            bounds.left() + scroll_offset + line.x_for_index(selected_range.start),
                            bounds.top(),
                        ),
                        point(
                            bounds.left() + scroll_offset + line.x_for_index(selected_range.end),
                            bounds.bottom(),
                        ),
                    ),
                    UiTheme::read(cx).colors.foreground.opacity(0.20),
                )),
                None,
            )
        } else {
            (
                None,
                Some(fill(
                    Bounds::new(
                        point(bounds.left() + scroll_offset + cursor_x, bounds.top()),
                        size(px(1.0), bounds.bottom() - bounds.top()),
                    ),
                    style.color,
                )),
            )
        };

        InputTextPrepaintState {
            line: Some(line),
            cursor,
            selection,
            scroll_offset,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.state.read(cx).focus_handle();
        if focus_handle.is_focused(window) {
            window.handle_input(
                &focus_handle,
                ElementInputHandler::new(bounds, self.state.clone()),
                cx,
            );
        }

        let scroll_offset = prepaint.scroll_offset;
        let selection = prepaint.selection.take();
        let line = prepaint
            .line
            .take()
            .expect("input text should be shaped during prepaint");
        let cursor = prepaint.cursor.take();
        let line_for_paint = line.clone();

        // Clip painting to the element so overflowing text never draws over
        // adjacent siblings (clear buttons, icons, group borders).
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            if focus_handle.is_focused(window)
                && let Some(selection) = selection
            {
                window.paint_quad(selection);
            }

            line_for_paint
                .paint(
                    bounds.origin + point(scroll_offset, px(0.0)),
                    window.line_height(),
                    TextAlign::Left,
                    None,
                    window,
                    cx,
                )
                .expect("input text should paint");

            if focus_handle.is_focused(window)
                && let Some(cursor) = cursor
            {
                window.paint_quad(cursor);
            }
        });

        self.state.update(cx, |input, cx| {
            input.set_last_layout(line, bounds, scroll_offset, cx);
        });
    }
}
