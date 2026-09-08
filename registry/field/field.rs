//! shadcn-style field composition backed by Base GPUI Field primitives.
//!
//! Visual source: shadcn/ui Field (`new-york-v4/ui/field.tsx`). Base GPUI
//! retains field registration, validation, label focus, and form integration.

use base_gpui::field::{
    FieldControl, FieldDescription, FieldError, FieldItem, FieldLabel, FieldRoot, FieldValidity,
};
use gpui::{App, Div, ElementId, FontWeight, Styled, prelude::FluentBuilder as _, px};

use super::theme::{ThemeMode, UiTheme, input_text_layout};

pub use base_gpui::field::{
    FieldErrorMatch, FieldValidationMode, FieldValidationResult, FieldValidityData,
    FieldValidityKey, FieldValidityState, FieldValue,
};

/// shadcn's field layout choice.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FieldOrientation {
    /// Stack the label, control, and supporting text.
    #[default]
    Vertical,
    /// Put the label and control on one row.
    Horizontal,
    /// Use the vertical layout until a future GPUI container-query API exists.
    Responsive,
}

/// Creates a styled Field root with Base GPUI validation and form wiring.
pub fn field_root(id: impl Into<ElementId>, orientation: FieldOrientation, cx: &App) -> FieldRoot {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    FieldRoot::new()
        .id(id)
        .style_with_state(move |state, base| {
            let base = base
                .flex()
                .w_full()
                .gap(spacing * 2_f32)
                .font_family(theme.fonts.body.clone())
                .text_size(px(14.0) * text_scale)
                .text_color(if state.invalid {
                    theme.colors.destructive
                } else {
                    theme.colors.foreground
                })
                .when(state.disabled, |base| base.opacity(0.50));

            match orientation {
                FieldOrientation::Vertical | FieldOrientation::Responsive => base.flex_col(),
                FieldOrientation::Horizontal => base.flex_row().items_center(),
            }
        })
}

/// Creates the text control used by a Field.
pub fn field_control(id: impl Into<ElementId>, cx: &App) -> FieldControl {
    let theme = UiTheme::read(cx).clone();
    FieldControl::new()
        .id(id)
        .style_with_state(move |state, base| style_field_control(base, state, &theme))
}

/// Creates a label that focuses its registered Field control on pointer press.
pub fn field_label(cx: &App) -> FieldLabel {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    FieldLabel::new().style_with_state(move |state, base| {
        base.flex()
            .gap(spacing * 2_f32)
            .font_family(theme.fonts.body.clone())
            .font_weight(FontWeight::MEDIUM)
            .text_size(px(14.0) * text_scale)
            .line_height(px(20.0) * text_scale)
            .text_color(theme.colors.foreground)
            .when(state.disabled, |base| base.opacity(0.50))
    })
}

/// Creates muted help text for a Field.
pub fn field_description(cx: &App) -> FieldDescription {
    let theme = UiTheme::read(cx).clone();
    let text_scale = theme.text_scale;
    FieldDescription::new().style_with_state(move |state, base| {
        base.font_family(theme.fonts.body.clone())
            .text_size(px(14.0) * text_scale)
            .line_height(px(20.0) * text_scale)
            .text_color(theme.colors.muted_foreground)
            .when(state.disabled, |base| base.opacity(0.50))
    })
}

/// Creates a destructive validation message. It only renders when an error exists.
pub fn field_error(cx: &App) -> FieldError {
    let theme = UiTheme::read(cx).clone();
    let text_scale = theme.text_scale;
    FieldError::new().style_with_state(move |_state, base| {
        base.font_family(theme.fonts.body.clone())
            .text_size(px(14.0) * text_scale)
            .line_height(px(20.0) * text_scale)
            .text_color(theme.colors.destructive)
    })
}

/// Creates a field item for grouped controls such as checkboxes and radios.
pub fn field_item(cx: &App) -> FieldItem {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    FieldItem::new().style_with_state(move |state, base| {
        base.flex()
            .flex_col()
            .gap(spacing * 1.5_f32)
            .font_family(theme.fonts.body.clone())
            .text_size(px(14.0) * text_scale)
            .when(state.disabled, |base| base.opacity(0.50))
    })
}

/// Creates a Field group, the shadcn visual counterpart to a plain GPUI Div.
pub fn field_group(cx: &App) -> Div {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    gpui::div().flex().flex_col().w_full().gap(spacing * 7_f32)
}

/// Creates the flex column used beside a checkbox, radio, or switch.
pub fn field_content(cx: &App) -> Div {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    gpui::div()
        .flex()
        .flex_col()
        .flex_1()
        .gap(spacing * 1.5_f32)
}

/// Creates label-styled text for `field_content` when it is not interactive.
pub fn field_title(cx: &App) -> Div {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    gpui::div()
        .flex()
        .items_center()
        .gap(spacing * 2_f32)
        .font_family(theme.fonts.body)
        .font_weight(FontWeight::MEDIUM)
        .text_size(px(14.0) * text_scale)
        .line_height(px(20.0) * text_scale)
        .text_color(theme.colors.foreground)
}

/// Creates a visual break between Field group sections.
pub fn field_separator(cx: &App) -> Div {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    gpui::div()
        .w_full()
        .h(px(1.0))
        .bg(theme.colors.border)
        .my(spacing * 1_f32)
}

/// Exposes Base GPUI validity state for custom indicators.
pub fn field_validity() -> FieldValidity {
    FieldValidity::new()
}

fn style_field_control(
    base: Div,
    state: base_gpui::primitives::InputStyleState,
    theme: &UiTheme,
) -> Div {
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    let colors = theme.colors;
    let focus_ring = theme.focus_ring();
    let destructive_focus_ring = theme.destructive_focus_ring();
    let border = if state.invalid {
        colors.destructive
    } else if state.focused {
        colors.ring
    } else {
        colors.input
    };
    let background = match theme.mode {
        ThemeMode::Light => colors.background,
        ThemeMode::Dark => colors.input.opacity(0.30),
    };

    input_text_layout(base, text_scale)
        .w_full()
        .h(spacing * 8_f32)
        .px(spacing * 2.5_f32)
        .rounded(theme.radius.lg)
        .border_1()
        .border_color(border)
        .bg(background)
        .font_family(theme.fonts.body.clone())
        .text_size(px(14.0) * text_scale)
        .text_color(colors.foreground)
        .when(state.focused, |base| base.shadow(focus_ring.clone()))
        .when(state.invalid, |base| {
            base.shadow(destructive_focus_ring.clone())
        })
        .when(state.disabled, |base| {
            base.opacity(0.50).cursor_not_allowed()
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{
        AppContext as _, Bounds, Context, IntoElement, ParentElement as _, Pixels, Render,
        TestAppContext, Window,
    };
    use std::{cell::RefCell, rc::Rc};

    struct View {
        bounds: Rc<RefCell<Vec<Bounds<Pixels>>>>,
        value: &'static str,
        inherited_line_height: f32,
    }
    impl Render for View {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let bounds = self.bounds.clone();
            let theme = UiTheme::read(cx).clone();
            gpui::div()
                .w(px(240.))
                .h(px(32.))
                .line_height(px(self.inherited_line_height))
                .child(
                    field_control("alignment", cx)
                        .value(self.value)
                        .placeholder("Ada Lovelace")
                        .style_with_state(move |state, base| {
                            let bounds = bounds.clone();
                            style_field_control(base, state, &theme).on_children_prepainted(
                                move |children, _, _| *bounds.borrow_mut() = children.to_vec(),
                            )
                        }),
                )
        }
    }
    #[test]
    fn field_text_is_vertically_centered() {
        for theme in [UiTheme::neutral_light(), UiTheme::neutral_dark()] {
            for value in ["", "Ada Lovelace"] {
                for inherited_line_height in [12., 40.] {
                    let mut cx = TestAppContext::single();
                    cx.update(|cx| UiTheme::set(cx, theme.clone()));
                    let bounds = Rc::new(RefCell::new(Vec::new()));
                    let captured = bounds.clone();
                    let window = cx.add_window(move |_, _| View {
                        bounds: captured,
                        value,
                        inherited_line_height,
                    });
                    cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
                        .unwrap();
                    let bounds = bounds.borrow();
                    assert_eq!(bounds.len(), 1);
                    assert_eq!(
                        bounds[0].center().y,
                        px(16.),
                        "value={value:?}, inherited line height={inherited_line_height}: {:?}",
                        bounds[0]
                    );
                    assert_eq!(
                        bounds[0].size.height,
                        px(20.),
                        "text and caret must ignore inherited line height"
                    );
                }
            }
        }
    }
}
