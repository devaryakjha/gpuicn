#![allow(missing_docs)]
//! Nova-styled Number Field backed by Base GPUI parsing, steppers, and keyboard behavior.

use std::rc::Rc;

use base_gpui::number_field::{
    NumberFieldChangeDetails, NumberFieldCommitDetails, NumberFieldDecrement, NumberFieldGroup,
    NumberFieldIncrement, NumberFieldInput, NumberFieldRoot,
};
use gpui::{
    App, ElementId, IntoElement, RenderOnce, SharedString, Styled, Window,
    prelude::FluentBuilder as _, px,
};
use gpui_icons::{LucideIcon, lucide};

use super::theme::{ThemeMode, UiTheme, focus_outline, input_text_layout};

type ChangeHandler = Rc<dyn Fn(Option<f64>, NumberFieldChangeDetails, &mut Window, &mut App)>;
type CommitHandler = Rc<dyn Fn(Option<f64>, NumberFieldCommitDetails, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct NumberField {
    id: ElementId,
    default_value: Option<f64>,
    value: Option<Option<f64>>,
    min: Option<f64>,
    max: Option<f64>,
    step: f64,
    placeholder: Option<SharedString>,
    disabled: bool,
    read_only: bool,
    on_value_change: Option<ChangeHandler>,
    on_value_committed: Option<CommitHandler>,
}
impl NumberField {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            default_value: None,
            value: None,
            min: None,
            max: None,
            step: 1.,
            placeholder: None,
            disabled: false,
            read_only: false,
            on_value_change: None,
            on_value_committed: None,
        }
    }
    pub fn default_value(mut self, value: f64) -> Self {
        self.default_value = Some(value);
        self
    }
    pub fn value(mut self, value: Option<f64>) -> Self {
        self.value = Some(value);
        self
    }
    pub fn range(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    pub fn step(mut self, value: f64) -> Self {
        self.step = value;
        self
    }
    pub fn placeholder(mut self, value: impl Into<SharedString>) -> Self {
        self.placeholder = Some(value.into());
        self
    }
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    pub fn read_only(mut self, value: bool) -> Self {
        self.read_only = value;
        self
    }
    pub fn on_value_change(
        mut self,
        handler: impl Fn(Option<f64>, NumberFieldChangeDetails, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(handler));
        self
    }
    pub fn on_value_committed(
        mut self,
        handler: impl Fn(Option<f64>, NumberFieldCommitDetails, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_committed = Some(Rc::new(handler));
        self
    }
}
impl RenderOnce for NumberField {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let background = if theme.mode == ThemeMode::Dark {
            colors.background.blend(colors.input.opacity(0.30))
        } else {
            colors.background
        };
        let input = NumberFieldInput::new().style_with_state(move |state, base| {
            input_text_layout(base)
                .flex_1()
                .min_w_0()
                .h(px(30.))
                .px(px(10.))
                .text_color(colors.foreground)
                .text_size(px(14.))
                .when(state.root.disabled, |base| base.opacity(0.50))
        });
        let input = if let Some(placeholder) = self.placeholder {
            input.placeholder(placeholder)
        } else {
            input
        };
        let mut root = NumberFieldRoot::new()
            .id(self.id)
            .default_value(self.default_value)
            .step(self.step)
            .disabled(self.disabled)
            .read_only(self.read_only)
            .style_with_state(move |state, base| {
                let ring = if state.invalid {
                    theme.destructive_focus_ring()[0].color
                } else {
                    theme.focus_ring()[0].color
                };
                base.w_full()
                    .h(px(32.))
                    .rounded(theme.radius.lg)
                    .border_1()
                    .border_color(if state.invalid {
                        colors.destructive
                    } else {
                        colors.input
                    })
                    .bg(background)
                    .when(state.focused, |base| {
                        focus_outline(
                            base.border_color(if state.invalid {
                                colors.destructive
                            } else {
                                colors.ring
                            }),
                            ring.into(),
                            gpui::Corners::all(theme.radius.lg),
                        )
                    })
                    .when(state.disabled, |base| base.cursor_not_allowed())
            })
            .child(
                NumberFieldGroup::new()
                    .w_full()
                    .h_full()
                    .flex()
                    .items_center()
                    .child(input)
                    .child(
                        NumberFieldDecrement::new()
                            .flex()
                            .size(px(30.))
                            .items_center()
                            .justify_center()
                            .border_l_1()
                            .border_color(colors.input)
                            .text_color(colors.muted_foreground)
                            .child(
                                lucide(LucideIcon::Minus)
                                    .size(px(14.))
                                    .text_color(colors.muted_foreground),
                            ),
                    )
                    .child(
                        NumberFieldIncrement::new()
                            .flex()
                            .size(px(30.))
                            .items_center()
                            .justify_center()
                            .border_l_1()
                            .border_color(colors.input)
                            .text_color(colors.muted_foreground)
                            .child(
                                lucide(LucideIcon::Plus)
                                    .size(px(14.))
                                    .text_color(colors.muted_foreground),
                            ),
                    ),
            );
        if let Some(value) = self.value {
            root = root.value(value);
        }
        if let Some(min) = self.min {
            root = root.min(min);
        }
        if let Some(max) = self.max {
            root = root.max(max);
        }
        if let Some(handler) = self.on_value_change {
            root = root.on_value_change(move |value, details, window, cx| {
                handler(value, details, window, cx)
            });
        }
        if let Some(handler) = self.on_value_committed {
            root = root.on_value_committed(move |value, details, window, cx| {
                handler(value, details, window, cx)
            });
        }
        root
    }
}
