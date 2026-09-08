//! shadcn-style Fieldset composition backed by Base GPUI Fieldset primitives.

use base_gpui::fieldset::{FieldsetLegend, FieldsetRoot};
use gpui::{App, ElementId, FontWeight, Styled, prelude::FluentBuilder as _, px};

use super::theme::UiTheme;

/// shadcn's two FieldLegend visual variants.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FieldsetLegendVariant {
    /// Section heading styling.
    #[default]
    Legend,
    /// Compact label styling for nested field groups.
    Label,
}

/// Creates a styled Fieldset root with Base GPUI disabled-state cascading.
pub fn fieldset_root(id: impl Into<ElementId>, cx: &App) -> FieldsetRoot {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    FieldsetRoot::new()
        .id(id)
        .style_with_state(move |state, base| {
            base.flex()
                .flex_col()
                .w_full()
                .gap(spacing * 6_f32)
                .font_family(theme.fonts.body.clone())
                .text_color(theme.colors.foreground)
                .when(state.disabled, |base| base.opacity(0.50))
        })
}

/// Creates a Fieldset legend. Give the root the same literal `aria_label`.
pub fn fieldset_legend(variant: FieldsetLegendVariant, cx: &App) -> FieldsetLegend {
    let theme = UiTheme::read(cx).clone();
    let text_scale = theme.text_scale;
    FieldsetLegend::new().style_with_state(move |state, base| {
        let text_size = match variant {
            FieldsetLegendVariant::Legend => 16.0,
            FieldsetLegendVariant::Label => 14.0,
        };

        base.font_family(theme.fonts.body.clone())
            .font_weight(FontWeight::MEDIUM)
            .text_size(px(text_size) * text_scale)
            .line_height(px(20.0) * text_scale)
            .text_color(theme.colors.foreground)
            .when(state.disabled, |base| base.opacity(0.50))
    })
}
