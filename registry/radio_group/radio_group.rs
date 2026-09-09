#![allow(missing_docs)]
//! Nova-styled Radio Group backed by Base GPUI roving focus and selection behavior.

use std::rc::Rc;

use base_gpui::radio_group::{RadioGroupRadio, RadioGroupRoot, RadioGroupValueChangeDetails};
use gpui::{
    App, ElementId, IntoElement, ParentElement as _, RenderOnce, SharedString, Styled, Window,
    prelude::FluentBuilder as _, px,
};

use super::theme::{ThemeMode, UiTheme};

type ChangeHandler = Rc<
    dyn Fn(Option<SharedString>, &mut RadioGroupValueChangeDetails, &mut Window, &mut App)
        + 'static,
>;

pub struct RadioItem {
    style: gpui::StyleRefinement,
    id: ElementId,
    value: SharedString,
    disabled: bool,
    aria_label: Option<SharedString>,
    label: Option<SharedString>,
}
impl RadioItem {
    pub fn new(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self {
            style: gpui::StyleRefinement::default(),
            id: id.into(),
            value: value.into(),
            disabled: false,
            aria_label: None,
            label: None,
        }
    }
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    pub fn aria_label(mut self, value: impl Into<SharedString>) -> Self {
        self.aria_label = Some(value.into());
        self
    }
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
    fn render(self, theme: &UiTheme) -> RadioGroupRadio<SharedString> {
        let spacing = theme.spacing.unit;
        let text_scale = theme.text_scale;
        let colors = theme.colors;
        let mode = theme.mode;
        let label = self.label.clone();
        let font = theme.fonts.body.clone();
        let mut radio = RadioGroupRadio::new()
            .id(self.id)
            .value(self.value)
            .disabled(self.disabled)
            .style_with_state(move |state, base| {
                let base = {
                    let circle = gpui::div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .flex_shrink_0()
                        .size(spacing * 4_f32)
                        .rounded_full()
                        .border_1()
                        .border_color(if state.checked {
                            colors.primary
                        } else {
                            colors.input
                        })
                        .bg(if state.checked {
                            colors.primary
                        } else if mode == ThemeMode::Dark {
                            colors.input.opacity(0.30)
                        } else {
                            colors.background.opacity(0.)
                        })
                        .when(state.focused && !state.disabled, |base| {
                            super::theme::focus_outline(
                                base,
                                colors.ring.opacity(0.50),
                                gpui::Corners::all(spacing * 2.),
                            )
                        })
                        .when(state.checked, |base| {
                            base.child(
                                gpui::div()
                                    .size(spacing * 2_f32)
                                    .rounded_full()
                                    .bg(colors.primary_foreground),
                            )
                        });
                    base.flex()
                        .items_center()
                        .gap(spacing * 2_f32)
                        .font_family(font.clone())
                        .text_size(px(14.) * text_scale)
                        .line_height(px(20.) * text_scale)
                        .text_color(colors.foreground)
                        .when(!state.disabled, |base| base.cursor_pointer())
                        .when(state.disabled, |base| {
                            base.opacity(0.50).cursor_not_allowed()
                        })
                        .child(circle)
                        .when_some(label.clone(), |base, label| {
                            base.child(gpui::Text::new_inaccessible(label))
                        })
                };
                super::theme::apply_style(base, &self.style)
            });
        if let Some(label) = self.aria_label.or(self.label) {
            radio = radio.aria_label(label);
        }
        radio
    }
}
#[derive(IntoElement)]
pub struct RadioGroup {
    style: gpui::StyleRefinement,
    id: ElementId,
    default_value: Option<SharedString>,
    value: Option<SharedString>,
    disabled: bool,
    aria_label: Option<SharedString>,
    items: Vec<RadioItem>,
    on_value_change: Option<ChangeHandler>,
}
impl RadioGroup {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            style: gpui::StyleRefinement::default(),
            id: id.into(),
            default_value: None,
            value: None,
            disabled: false,
            aria_label: None,
            items: Vec::new(),
            on_value_change: None,
        }
    }
    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default_value = Some(value.into());
        self
    }
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }
    pub fn item(mut self, item: RadioItem) -> Self {
        self.items.push(item);
        self
    }
    pub fn on_value_change(
        mut self,
        handler: impl Fn(Option<SharedString>, &mut RadioGroupValueChangeDetails, &mut Window, &mut App)
        + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(handler));
        self
    }
}
impl RenderOnce for RadioGroup {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let spacing = theme.spacing.unit;
        let mut root = RadioGroupRoot::new()
            .id(self.id)
            .disabled(self.disabled)
            .flex()
            .flex_col()
            .gap(spacing * 2_f32);
        if let Some(value) = self.default_value {
            root = root.default_value(Some(value));
        }
        if let Some(value) = self.value {
            root = root.value(Some(value));
        }
        if let Some(label) = self.aria_label {
            root = root.aria_label(label);
        }
        if let Some(handler) = self.on_value_change {
            root = root.on_value_change(move |value, details, window, cx| {
                handler(value.cloned(), details, window, cx)
            });
        }
        super::theme::apply_style(root, &self.style)
            .children(self.items.into_iter().map(|item| item.render(&theme)))
    }
}

impl gpui::Styled for RadioItem {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl gpui::Styled for RadioGroup {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
