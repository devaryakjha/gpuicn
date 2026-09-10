//! Nova-styled semantic numeric meter.

use gpui_kit::{
    App, ElementId, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce, Role,
    SharedString, StatefulInteractiveElement as _, Styled, Window, div, relative,
};

use super::theme::UiTheme;

#[derive(IntoElement)]
/// A Nova meter for a bounded scalar value.
pub struct Meter {
    style: gpui_kit::StyleRefinement,
    id: ElementId,
    value: f64,
    min: f64,
    max: f64,
    aria_label: Option<SharedString>,
}

impl Meter {
    /// Creates a `Meter` with a stable caller-owned ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            style: gpui_kit::StyleRefinement::default(),
            id: id.into(),
            value: 0.,
            min: 0.,
            max: 100.,
            aria_label: None,
        }
    }
    /// Sets the caller-owned value displayed by this control.
    pub fn value(mut self, value: f64) -> Self {
        self.value = value;
        self
    }
    /// Sets the minimum and maximum for the displayed value.
    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    /// Sets the accessible name of the control.
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }
}

impl RenderOnce for Meter {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let spacing = theme.spacing.unit;
        assert!(
            self.value.is_finite()
                && self.min.is_finite()
                && self.max.is_finite()
                && self.min < self.max,
            "meter needs a finite value and an increasing finite range"
        );
        let fraction =
            ((self.value.clamp(self.min, self.max) - self.min) / (self.max - self.min)) as f32;
        let root = div()
            .id(self.id)
            .role(Role::Meter)
            .aria_numeric_value(self.value.clamp(self.min, self.max))
            .aria_min_numeric_value(self.min)
            .aria_max_numeric_value(self.max)
            .w_full();
        let mut root = super::theme::apply_style(root, &self.style);
        if let Some(label) = self.aria_label {
            root = root.aria_label(label);
        }
        root.child(
            div()
                .relative()
                .w_full()
                .h(spacing * 1_f32)
                .rounded_full()
                .overflow_hidden()
                .bg(theme.colors.muted)
                .child(
                    div()
                        .w(relative(fraction))
                        .h_full()
                        .rounded_full()
                        .bg(theme.colors.primary),
                ),
        )
    }
}

impl gpui_kit::Styled for Meter {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}
