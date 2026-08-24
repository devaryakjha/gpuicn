#![allow(missing_docs)]
//! Nova-styled determinate and indeterminate Progress backed by Base GPUI.

use base_gpui::progress::{ProgressIndicator, ProgressRoot, ProgressTrack};
use gpui::{App, ElementId, IntoElement, RenderOnce, SharedString, Styled, Window, px};

use super::theme::UiTheme;

#[derive(IntoElement)]
pub struct Progress {
    id: ElementId,
    value: Option<f64>,
    min: f64,
    max: f64,
    label: Option<SharedString>,
}

impl Progress {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: None,
            min: 0.,
            max: 100.,
            label: None,
        }
    }
    pub fn value(mut self, value: f64) -> Self {
        self.value = Some(value);
        self
    }
    pub fn indeterminate(mut self) -> Self {
        self.value = None;
        self
    }
    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl RenderOnce for Progress {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let mut root = ProgressRoot::new()
            .id(self.id)
            .value(self.value)
            .min(self.min)
            .max(self.max)
            .w_full()
            .flex()
            .flex_col()
            .gap(px(6.));
        if let Some(label) = self.label {
            root = root.label(label);
        }
        root.child(
            ProgressTrack::new()
                .relative()
                .w_full()
                .h(px(4.))
                .rounded_full()
                .overflow_hidden()
                .bg(theme.colors.muted)
                .child(
                    ProgressIndicator::new()
                        .h_full()
                        .rounded_full()
                        .bg(theme.colors.primary),
                ),
        )
    }
}
