#![allow(missing_docs)]
//! Nova-styled Toggle Group backed by Base GPUI roving focus and selection rules.

use std::rc::Rc;

use base_gpui::toggle::Toggle as BaseToggle;
use base_gpui::toggle_group::{ToggleGroup as BaseToggleGroup, ToggleGroupValueChangeDetails};
use gpui::{
    AnyElement, App, BoxShadow, ElementId, InteractiveElement as _, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, Window, prelude::FluentBuilder as _, px,
};

use super::theme::UiTheme;

type ChangeHandler = Rc<
    dyn Fn(&[SharedString], &mut ToggleGroupValueChangeDetails, &mut Window, &mut App) + 'static,
>;

pub struct ToggleGroupItem {
    id: ElementId,
    value: SharedString,
    disabled: bool,
    aria_label: Option<SharedString>,
    children: Vec<AnyElement>,
}
impl ToggleGroupItem {
    pub fn new(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            value: value.into(),
            disabled: false,
            aria_label: None,
            children: Vec::new(),
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
    fn render(self, theme: &UiTheme, joined: bool) -> BaseToggle<SharedString> {
        let colors = theme.colors;
        let mut toggle = BaseToggle::new()
            .id(self.id)
            .value(self.value)
            .disabled(self.disabled)
            .style_with_state(move |state, base| {
                base.flex()
                    .items_center()
                    .justify_center()
                    .gap(px(4.))
                    .h(px(32.))
                    .min_w(px(32.))
                    .px(px(10.))
                    .rounded(px(if joined { 0. } else { 10. }))
                    .text_size(px(14.))
                    .text_color(colors.muted_foreground)
                    .bg(if state.pressed {
                        colors.muted
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
            .children(self.children);
        if let Some(label) = self.aria_label {
            toggle = toggle.aria_label(label);
        }
        toggle
    }
}
impl ParentElement for ToggleGroupItem {
    fn extend(&mut self, children: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(children);
    }
}

#[derive(IntoElement)]
pub struct ToggleGroup {
    id: ElementId,
    default_value: Vec<SharedString>,
    value: Option<Vec<SharedString>>,
    multiple: bool,
    disabled: bool,
    aria_label: Option<SharedString>,
    joined: bool,
    items: Vec<ToggleGroupItem>,
    on_value_change: Option<ChangeHandler>,
}
impl ToggleGroup {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            default_value: Vec::new(),
            value: None,
            multiple: false,
            disabled: false,
            aria_label: None,
            joined: true,
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
    pub fn multiple(mut self, value: bool) -> Self {
        self.multiple = value;
        self
    }
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    pub fn aria_label(mut self, value: impl Into<SharedString>) -> Self {
        self.aria_label = Some(value.into());
        self
    }
    pub fn joined(mut self, value: bool) -> Self {
        self.joined = value;
        self
    }
    pub fn item(mut self, item: ToggleGroupItem) -> Self {
        self.items.push(item);
        self
    }
    pub fn on_value_change(
        mut self,
        handler: impl Fn(&[SharedString], &mut ToggleGroupValueChangeDetails, &mut Window, &mut App)
        + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(handler));
        self
    }
}
impl RenderOnce for ToggleGroup {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let mut group = BaseToggleGroup::new()
            .id(self.id)
            .default_value(self.default_value)
            .multiple(self.multiple)
            .disabled(self.disabled)
            .flex()
            .gap(px(if self.joined { 0. } else { 8. }));
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
        group.children(
            self.items
                .into_iter()
                .map(|item| item.render(&theme, self.joined)),
        )
    }
}
