//! Nova radio selection with GPUI Kit controls and roving focus.
use super::theme::{
    RovingFocus, UiTheme, apply_style,
    primitives::{control, disabled, focus, text},
};
use gpui_kit::{
    App, ElementId, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    SharedString, StatefulInteractiveElement as _, StyleRefinement, Styled, Window, div,
    prelude::FluentBuilder as _,
};
use std::rc::Rc;

type ChangeHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;
/// One radio option with a stable ID and selected value.
pub struct RadioItem {
    id: ElementId,
    value: SharedString,
    label: SharedString,
    disabled: bool,
    style: StyleRefinement,
}
impl RadioItem {
    /// Creates a `RadioItem` with a stable caller-owned ID.
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
impl Styled for RadioItem {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
#[derive(IntoElement)]
/// A controlled, single-choice group with roving keyboard focus.
pub struct RadioGroup {
    id: ElementId,
    value: Option<SharedString>,
    label: Option<SharedString>,
    disabled: bool,
    items: Vec<RadioItem>,
    on_change: Option<ChangeHandler>,
    style: StyleRefinement,
}
impl RadioGroup {
    /// Creates a `RadioGroup` with a stable caller-owned ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: None,
            label: None,
            disabled: false,
            items: Vec::new(),
            on_change: None,
            style: Default::default(),
        }
    }
    /// Sets the caller-owned value displayed by this control.
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
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
    pub fn item(mut self, item: RadioItem) -> Self {
        self.items.push(item);
        self
    }
    /// Reports a requested value change; retain the next value in the owning view.
    pub fn on_change(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}
impl Styled for RadioGroup {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
impl RenderOnce for RadioGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let selected = self
            .items
            .iter()
            .position(|i| Some(&i.value) == self.value.as_ref());
        let focus = RovingFocus::new(
            self.id.clone(),
            &self
                .items
                .iter()
                .map(|i| (i.id.clone(), self.disabled || i.disabled))
                .collect::<Vec<_>>(),
            selected,
            window,
            cx,
        );
        let values: Vec<_> = self.items.iter().map(|i| i.value.clone()).collect();
        let count = self.items.len();
        let children = self
            .items
            .into_iter()
            .enumerate()
            .map(|(index, item)| {
                let checked = Some(index) == selected;
                let disabled = self.disabled || item.disabled;
                let handle = focus.handles[index].as_ref();
                let focused = handle.is_some_and(|h| h.is_focused(window));
                let change = self.on_change.clone();
                let circle =
                    focus::shown(control::radio_dot(div(), &theme, checked), &theme, focused);
                let radio = gpui_kit::base::Radio::new(item.id)
                    .checked(checked)
                    .disabled(disabled)
                    .accessibility_label(item.label.clone())
                    .set_position(index + 1, count)
                    .when_some(handle, |radio, h| radio.track_focus(h).tab_stop(h.tab_stop))
                    .flex()
                    .items_center()
                    .gap(theme.space(2.))
                    .text_color(colors.foreground)
                    .child(circle)
                    .child(gpui_kit::Text::new_inaccessible(item.label))
                    .when_some(change, |radio, handler| {
                        radio.on_change(move |_, _, window, cx| {
                            handler(item.value.clone(), window, cx)
                        })
                    });
                let radio = disabled::interactive(text::sm(radio, &theme), disabled);
                apply_style(radio, &item.style)
            })
            .collect::<Vec<_>>();
        let on_change = self.on_change;
        let root = gpui_kit::base::RadioGroup::new(self.id)
            .flex()
            .flex_col()
            .gap(theme.space(2.))
            .when_some(self.label, |root, label| root.aria_label(label))
            .on_key_down(move |event, window, cx| {
                if let Some(index) = focus.key(event, None, window, cx)
                    && let Some(change) = &on_change
                {
                    change(values[index].clone(), window, cx);
                }
            })
            .children(children);
        apply_style(root, &self.style)
    }
}

impl super::field::FieldControl for RadioGroup {
    fn field_label_focus(&self, _: &App) -> Option<gpui_kit::FocusHandle> {
        None
    }

    fn into_field_control(
        self,
        label: SharedString,
        disabled: bool,
        _invalid: bool,
    ) -> gpui_kit::AnyElement {
        let disabled = self.disabled || disabled;
        self.aria_label(label)
            .disabled(disabled)
            .into_any_element()
    }
}
