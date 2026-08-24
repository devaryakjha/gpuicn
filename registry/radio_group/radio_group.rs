#![allow(missing_docs)]
//! Nova-styled Radio Group backed by Base GPUI roving focus and selection behavior.

use std::rc::Rc;

use base_gpui::radio_group::{
    RadioGroupIndicator, RadioGroupRadio, RadioGroupRoot, RadioGroupValueChangeDetails,
};
use gpui::{
    App, BoxShadow, ElementId, InteractiveElement as _, IntoElement, ParentElement as _,
    RenderOnce, SharedString, Styled, Window, prelude::FluentBuilder as _, px,
};

use super::theme::{ThemeMode, UiTheme};

type ChangeHandler = Rc<
    dyn Fn(Option<SharedString>, &mut RadioGroupValueChangeDetails, &mut Window, &mut App)
        + 'static,
>;

pub struct RadioItem {
    id: ElementId,
    value: SharedString,
    disabled: bool,
    aria_label: Option<SharedString>,
}
impl RadioItem {
    pub fn new(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            value: value.into(),
            disabled: false,
            aria_label: None,
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
    fn render(self, theme: &UiTheme) -> RadioGroupRadio<SharedString> {
        let colors = theme.colors;
        let mode = theme.mode;
        let mut radio = RadioGroupRadio::new()
            .id(self.id)
            .value(self.value)
            .disabled(self.disabled)
            .style_with_state(move |state, base| {
                let background = if state.checked {
                    colors.primary
                } else if mode == ThemeMode::Dark {
                    colors.input.alpha(0.30)
                } else {
                    colors.background.alpha(0.)
                };
                base.relative()
                    .flex_shrink_0()
                    .size(px(16.))
                    .rounded_full()
                    .border_1()
                    .border_color(if state.checked {
                        colors.primary
                    } else {
                        colors.input
                    })
                    .bg(background)
                    .focus_visible(move |style| {
                        style.border_color(colors.ring).shadow(vec![
                            BoxShadow::new(px(0.), px(0.), colors.ring.alpha(0.50).into())
                                .spread_radius(px(3.)),
                        ])
                    })
                    .when(state.disabled, |base| {
                        base.opacity(0.50).cursor_not_allowed()
                    })
            })
            .child(
                RadioGroupIndicator::new()
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        gpui::div()
                            .size(px(8.))
                            .rounded_full()
                            .bg(colors.primary_foreground),
                    ),
            );
        if let Some(label) = self.aria_label {
            radio = radio.aria_label(label);
        }
        radio
    }
}
#[derive(IntoElement)]
pub struct RadioGroup {
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
        let mut root = RadioGroupRoot::new()
            .id(self.id)
            .disabled(self.disabled)
            .flex()
            .flex_col()
            .gap(px(8.));
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
        root.children(self.items.into_iter().map(|item| item.render(&theme)))
    }
}
