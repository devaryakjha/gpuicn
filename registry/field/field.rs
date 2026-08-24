//! shadcn-style field composition backed by Base GPUI Field primitives.
//!
//! Visual source: shadcn/ui Field (`new-york-v4/ui/field.tsx`). Base GPUI
//! retains field registration, validation, label focus, and form integration.

use base_gpui::field::{
    FieldControl, FieldDescription, FieldError, FieldItem, FieldLabel, FieldRoot, FieldValidity,
};
use gpui::{App, BoxShadow, Div, ElementId, FontWeight, Styled, prelude::FluentBuilder as _, px};

use super::theme::{ThemeMode, UiTheme};

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
    FieldRoot::new()
        .id(id)
        .style_with_state(move |state, base| {
            let base = base
                .flex()
                .w_full()
                .gap(px(8.0))
                .font_family(theme.fonts.body.clone())
                .text_size(px(14.0))
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
    FieldLabel::new().style_with_state(move |state, base| {
        base.flex()
            .gap(px(8.0))
            .font_family(theme.fonts.body.clone())
            .font_weight(FontWeight::MEDIUM)
            .text_size(px(14.0))
            .line_height(px(20.0))
            .text_color(theme.colors.foreground)
            .when(state.disabled, |base| base.opacity(0.50))
    })
}

/// Creates muted help text for a Field.
pub fn field_description(cx: &App) -> FieldDescription {
    let theme = UiTheme::read(cx).clone();
    FieldDescription::new().style_with_state(move |state, base| {
        base.font_family(theme.fonts.body.clone())
            .text_size(px(14.0))
            .line_height(px(20.0))
            .text_color(theme.colors.muted_foreground)
            .when(state.disabled, |base| base.opacity(0.50))
    })
}

/// Creates a destructive validation message. It only renders when an error exists.
pub fn field_error(cx: &App) -> FieldError {
    let theme = UiTheme::read(cx).clone();
    FieldError::new().style_with_state(move |_state, base| {
        base.font_family(theme.fonts.body.clone())
            .text_size(px(14.0))
            .line_height(px(20.0))
            .text_color(theme.colors.destructive)
    })
}

/// Creates a field item for grouped controls such as checkboxes and radios.
pub fn field_item(cx: &App) -> FieldItem {
    let theme = UiTheme::read(cx).clone();
    FieldItem::new().style_with_state(move |state, base| {
        base.flex()
            .flex_col()
            .gap(px(6.0))
            .font_family(theme.fonts.body.clone())
            .text_size(px(14.0))
            .when(state.disabled, |base| base.opacity(0.50))
    })
}

/// Creates a Field group, the shadcn visual counterpart to a plain GPUI Div.
pub fn field_group() -> Div {
    gpui::div().flex().flex_col().w_full().gap(px(28.0))
}

/// Creates the flex column used beside a checkbox, radio, or switch.
pub fn field_content() -> Div {
    gpui::div().flex().flex_col().flex_1().gap(px(6.0))
}

/// Creates label-styled text for `field_content` when it is not interactive.
pub fn field_title(cx: &App) -> Div {
    let theme = UiTheme::read(cx).clone();
    gpui::div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .font_family(theme.fonts.body)
        .font_weight(FontWeight::MEDIUM)
        .text_size(px(14.0))
        .line_height(px(20.0))
        .text_color(theme.colors.foreground)
}

/// Creates a visual break between Field group sections.
pub fn field_separator(cx: &App) -> Div {
    let theme = UiTheme::read(cx).clone();
    gpui::div()
        .w_full()
        .h(px(1.0))
        .bg(theme.colors.border)
        .my(px(4.0))
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
    let colors = theme.colors;
    let border = if state.invalid {
        colors.destructive
    } else if state.focused {
        colors.ring
    } else {
        colors.input
    };
    let background = match theme.mode {
        ThemeMode::Light => colors.background,
        ThemeMode::Dark => colors.input.alpha(0.30),
    };

    base.w_full()
        .h(px(32.0))
        .px(px(10.0))
        .rounded(theme.radius.base)
        .border_1()
        .border_color(border)
        .bg(background)
        .font_family(theme.fonts.body.clone())
        .text_size(px(14.0))
        .text_color(colors.foreground)
        .when(state.focused, |base| {
            base.shadow(vec![
                BoxShadow::new(px(0.0), px(0.0), colors.ring.alpha(0.50).into())
                    .spread_radius(px(3.0)),
            ])
        })
        .when(state.invalid, |base| {
            base.shadow(vec![
                BoxShadow::new(px(0.0), px(0.0), colors.destructive.alpha(0.20).into())
                    .spread_radius(px(3.0)),
            ])
        })
        .when(state.disabled, |base| {
            base.opacity(0.50).cursor_not_allowed()
        })
}
