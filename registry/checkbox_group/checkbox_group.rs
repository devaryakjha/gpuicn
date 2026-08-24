#![allow(missing_docs)]
//! Nova-styled Checkbox Group backed by Base GPUI group state.

use std::rc::Rc;

use base_gpui::checkbox::{CheckboxIndicator, CheckboxRoot};
use base_gpui::checkbox_group::{
    CheckboxGroup as BaseCheckboxGroup, CheckboxGroupValueChangeDetails,
};
use gpui::{
    App, BoxShadow, ElementId, InteractiveElement as _, IntoElement, ParentElement as _,
    RenderOnce, SharedString, Styled, Window, prelude::FluentBuilder as _, px,
};
use gpui_icons::{LucideIcon, lucide};

use super::theme::{ThemeMode, UiTheme};

type ChangeHandler = Rc<
    dyn Fn(Vec<SharedString>, &mut CheckboxGroupValueChangeDetails, &mut Window, &mut App)
        + 'static,
>;

pub struct CheckboxGroupItem {
    id: ElementId,
    value: SharedString,
    disabled: bool,
    aria_label: Option<SharedString>,
}
impl CheckboxGroupItem {
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
    fn render(self, theme: &UiTheme) -> CheckboxRoot {
        let colors = theme.colors;
        let mode = theme.mode;
        let mut checkbox = CheckboxRoot::new()
            .id(self.id)
            .value(self.value)
            .disabled(self.disabled)
            .relative()
            .style_with_state(move |state, base| {
                let selected = state.checked || state.indeterminate;
                base.flex_shrink_0()
                    .size(px(16.))
                    .rounded(px(4.))
                    .border_1()
                    .border_color(if selected {
                        colors.primary
                    } else {
                        colors.input
                    })
                    .bg(if selected {
                        colors.primary
                    } else if mode == ThemeMode::Dark {
                        colors.input.alpha(0.30)
                    } else {
                        colors.background.alpha(0.)
                    })
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
                CheckboxIndicator::new()
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        lucide(LucideIcon::Check)
                            .size(px(14.))
                            .text_color(colors.primary_foreground),
                    ),
            );
        if let Some(label) = self.aria_label {
            checkbox = checkbox.aria_label(label);
        }
        checkbox
    }
}

#[derive(IntoElement)]
pub struct CheckboxGroup {
    id: ElementId,
    default_value: Vec<SharedString>,
    value: Option<Vec<SharedString>>,
    all_values: Vec<SharedString>,
    disabled: bool,
    aria_label: Option<SharedString>,
    items: Vec<CheckboxGroupItem>,
    on_value_change: Option<ChangeHandler>,
}
impl CheckboxGroup {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            default_value: Vec::new(),
            value: None,
            all_values: Vec::new(),
            disabled: false,
            aria_label: None,
            items: Vec::new(),
            on_value_change: None,
        }
    }
    pub fn default_value(
        mut self,
        values: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.default_value = values.into_iter().map(Into::into).collect();
        self
    }
    pub fn value(mut self, values: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.value = Some(values.into_iter().map(Into::into).collect());
        self
    }
    pub fn all_values(mut self, values: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.all_values = values.into_iter().map(Into::into).collect();
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
    pub fn item(mut self, item: CheckboxGroupItem) -> Self {
        self.items.push(item);
        self
    }
    pub fn on_value_change(
        mut self,
        handler: impl Fn(Vec<SharedString>, &mut CheckboxGroupValueChangeDetails, &mut Window, &mut App)
        + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(handler));
        self
    }
}
impl RenderOnce for CheckboxGroup {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let mut group = BaseCheckboxGroup::new()
            .id(self.id)
            .default_value(self.default_value)
            .all_values(self.all_values)
            .disabled(self.disabled)
            .flex()
            .flex_col()
            .gap(px(12.));
        if let Some(value) = self.value {
            group = group.value(value);
        }
        if let Some(label) = self.aria_label {
            group = group.aria_label(label);
        }
        if let Some(handler) = self.on_value_change {
            group = group.on_value_change(move |values, details, window, cx| {
                handler(values, details, window, cx)
            });
        }
        group.children(self.items.into_iter().map(|item| item.render(&theme)))
    }
}
