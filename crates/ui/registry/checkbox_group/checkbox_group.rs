//! Controlled checkbox sets with clickable labels.
use super::theme::{
    UiTheme, apply_style,
    primitives::{control, disabled, focus, text},
};
use gpui_kit::{
    App, ElementId, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce, Role,
    SharedString, StatefulInteractiveElement as _, StyleRefinement, Styled, Window, div,
    prelude::FluentBuilder as _,
};
use std::rc::Rc;

type ChangeHandler = Rc<dyn Fn(Vec<SharedString>, &mut Window, &mut App)>;
/// One checkbox option with a stable ID and selected value.
pub struct CheckboxGroupItem {
    id: ElementId,
    value: SharedString,
    label: SharedString,
    disabled: bool,
    style: StyleRefinement,
}
impl CheckboxGroupItem {
    /// Creates a `CheckboxGroupItem` with a stable caller-owned ID.
    pub fn new(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        let value = value.into();
        Self {
            id: id.into(),
            label: value.clone(),
            value,
            disabled: false,
            style: Default::default(),
        }
    }
    /// Disables interaction and applies the disabled appearance.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    /// Sets the visible label.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }
}
impl Styled for CheckboxGroupItem {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
#[derive(IntoElement)]
/// A named collection of controlled checkboxes.
pub struct CheckboxGroup {
    id: ElementId,
    value: Vec<SharedString>,
    label: Option<SharedString>,
    disabled: bool,
    items: Vec<CheckboxGroupItem>,
    on_change: Option<ChangeHandler>,
    style: StyleRefinement,
}
impl CheckboxGroup {
    /// Creates a `CheckboxGroup` with a stable caller-owned ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: Vec::new(),
            label: None,
            disabled: false,
            items: Vec::new(),
            on_change: None,
            style: Default::default(),
        }
    }
    /// Sets the caller-owned value displayed by this control.
    pub fn value(mut self, values: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.value = values.into_iter().map(Into::into).collect();
        self
    }
    /// Sets the accessible name of the control.
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
    /// Disables interaction and applies the disabled appearance.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    /// Appends an option to the group.
    pub fn item(mut self, item: CheckboxGroupItem) -> Self {
        self.items.push(item);
        self
    }
    /// Reports a requested value change; retain the next value in the owning view.
    pub fn on_change(
        mut self,
        handler: impl Fn(Vec<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}
impl Styled for CheckboxGroup {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
impl RenderOnce for CheckboxGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let children = self
            .items
            .into_iter()
            .map(|item| {
                let checked = self.value.contains(&item.value);
                let disabled = self.disabled || item.disabled;
                let focus = window
                    .use_keyed_state((item.id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
                    .read(cx)
                    .clone();
                let focused = focus.is_focused(window) && !disabled;
                let change = self.on_change.clone();
                let mut next = self.value.clone();
                next.retain(|value| value != &item.value);
                if !checked {
                    next.push(item.value);
                }
                let square = focus::shown(
                    control::checkbox_box(div(), &theme, checked, false),
                    &theme,
                    focused,
                );
                let checkbox = gpui_kit::base::Checkbox::new(item.id)
                    .checked(checked)
                    .disabled(disabled)
                    .accessibility_label(item.label.clone())
                    .track_focus(&focus)
                    .flex()
                    .items_center()
                    .gap(theme.space(2.))
                    .text_color(colors.foreground)
                    .child(square)
                    .child(gpui_kit::Text::new_inaccessible(item.label))
                    .when_some(change, |checkbox, handler| {
                        checkbox
                            .on_change(move |_, _, window, cx| handler(next.clone(), window, cx))
                    });
                let checkbox = disabled::interactive(text::sm(checkbox, &theme), disabled);
                apply_style(checkbox, &item.style)
            })
            .collect::<Vec<_>>();
        let root = div()
            .id(self.id)
            .role(Role::Group)
            .flex()
            .flex_col()
            .gap(theme.space(3.))
            .when_some(self.label, |root, label| root.aria_label(label))
            .children(children);
        apply_style(root, &self.style)
    }
}
