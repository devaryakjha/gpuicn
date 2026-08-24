#![allow(missing_docs)]
//! Nova-styled Meter backed by Base GPUI's clamped numeric semantics.

use base_gpui::meter::{MeterIndicator, MeterRoot, MeterTrack};
use gpui::{App, ElementId, IntoElement, RenderOnce, SharedString, Styled, Window, px};

use super::theme::UiTheme;

#[derive(IntoElement)]
pub struct Meter {
    id: ElementId,
    value: f64,
    min: f64,
    max: f64,
    aria_label: Option<SharedString>,
}

impl Meter {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
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
        let mut root = MeterRoot::new()
            .id(self.id)
            .value(self.value)
            .min(self.min)
            .max(self.max)
            .w_full();
        if let Some(label) = self.aria_label {
            root = root.aria_label(label);
        }
        root.child(
            MeterTrack::new()
                .relative()
                .w_full()
                .h(px(4.))
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
