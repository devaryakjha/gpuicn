//! Nova-styled determinate and indeterminate Progress using GPUI Kit.

use gpui_kit::base::{Progress as BaseProgress, ProgressIndicator, ProgressTrack};
use gpui_kit::{
    App, ElementId, IntoElement, ParentElement as _, RenderOnce, SharedString, Styled, Window,
    relative,
};

use super::theme::UiTheme;

#[derive(IntoElement)]
/// A Nova track for determinate or indeterminate progress.
pub struct Progress {
    style: gpui_kit::StyleRefinement,
    id: ElementId,
    value: Option<f64>,
    min: f64,
    max: f64,
    label: Option<SharedString>,
}

impl Progress {
    /// Creates a `Progress` with a stable caller-owned ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            style: gpui_kit::StyleRefinement::default(),
            id: id.into(),
            value: None,
            min: 0.,
            max: 100.,
            label: None,
        }
    }
    /// Sets the caller-owned value displayed by this control.
    pub fn value(mut self, value: f64) -> Self {
        self.value = Some(value);
        self
    }
    /// Shows progress without a known completion value.
    pub fn indeterminate(mut self) -> Self {
        self.value = None;
        self
    }
    /// Sets the minimum and maximum for the displayed value.
    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    /// Sets the accessible label for progress.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl RenderOnce for Progress {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let spacing = theme.spacing.unit;
        let progress = self.value.map(|value| ratio(value, self.min, self.max));
        let root = BaseProgress::new(self.id)
            .value(progress.unwrap_or(0.) * 100.)
            .indeterminate(progress.is_none())
            .w_full()
            .flex()
            .flex_col()
            .gap(spacing * 1.5_f32);
        let mut root = super::theme::apply_style(root, &self.style);
        if let Some(label) = self.label {
            root = root.accessibility_label(label);
        }
        root.child(
            ProgressTrack::new()
                .relative()
                .w_full()
                .h(spacing * 1_f32)
                .rounded_full()
                .overflow_hidden()
                .bg(theme.colors.muted)
                .child(
                    ProgressIndicator::new()
                        .w(relative(progress.unwrap_or(0.33)))
                        .h_full()
                        .rounded_full()
                        .bg(theme.colors.primary),
                ),
        )
    }
}

impl gpui_kit::Styled for Progress {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}

/// Normalizes a finite numeric value and clamps it to a valid range.
fn ratio(value: f64, min: f64, max: f64) -> f32 {
    assert!(
        value.is_finite() && min.is_finite() && max.is_finite() && min < max,
        "progress needs a finite value and an increasing finite range"
    );
    ((value.clamp(min, max) - min) / (max - min)) as f32
}

#[cfg(test)]
mod tests {
    use super::ratio;
    #[test]
    fn numeric_ranges_clamp_at_both_ends() {
        assert_eq!(ratio(-5., 10., 30.), 0.);
        assert_eq!(ratio(20., 10., 30.), 0.5);
        assert_eq!(ratio(40., 10., 30.), 1.);
    }
}
