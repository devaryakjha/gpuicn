#![allow(missing_docs)]
//! Nova-styled Meter backed by Base GPUI's clamped numeric semantics.

use base_gpui::meter::{MeterIndicator, MeterRoot, MeterTrack};
use gpui::{App, ElementId, IntoElement, RenderOnce, SharedString, Styled, Window};

use super::theme::UiTheme;

#[derive(IntoElement)]
pub struct Meter {
    style: gpui::StyleRefinement,
    id: ElementId,
    value: f64,
    min: f64,
    max: f64,
    aria_label: Option<SharedString>,
}

impl Meter {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            style: gpui::StyleRefinement::default(),
            id: id.into(),
            value: 0.,
            min: 0.,
            max: 100.,
            aria_label: None,
        }
    }
    pub fn value(mut self, value: f64) -> Self {
        self.value = value;
        self
    }
    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }
}

impl RenderOnce for Meter {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let spacing = theme.spacing.unit;
        let root = MeterRoot::new()
            .id(self.id)
            .value(self.value)
            .min(self.min)
            .max(self.max)
            .w_full();
        let mut root = super::theme::apply_style(root, &self.style);
        if let Some(label) = self.aria_label {
            root = root.aria_label(label);
        }
        root.child(
            MeterTrack::new()
                .relative()
                .w_full()
                .h(spacing * 1_f32)
                .rounded_full()
                .overflow_hidden()
                .bg(theme.colors.muted)
                .child(
                    MeterIndicator::new()
                        .h_full()
                        .rounded_full()
                        .bg(theme.colors.primary),
                ),
        )
    }
}

impl gpui::Styled for Meter {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
