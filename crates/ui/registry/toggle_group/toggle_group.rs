//! Nova toggle sets with controlled selection and arrow-key focus.
use super::theme::{RovingFocus, UiTheme, apply_style};
use gpui_kit::{
    AnyElement, App, ElementId, InteractiveElement as _, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement as _, StyleRefinement, Styled, Window,
    prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

type ChangeHandler = Rc<dyn Fn(Vec<SharedString>, &mut Window, &mut App)>;
/// A labeled toggle option with custom child content.
pub struct ToggleGroupItem {
    id: ElementId,
    value: SharedString,
    label: Option<SharedString>,
    children: Vec<AnyElement>,
    disabled: bool,
    style: StyleRefinement,
}
impl ToggleGroupItem {
    /// Creates a `ToggleGroupItem` with a stable caller-owned ID.
    pub fn new(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            value: value.into(),
            label: None,
            children: Vec::new(),
            disabled: false,
            style: Default::default(),
        }
    }
    /// Disables interaction and applies the disabled appearance.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    /// Sets the accessible name of the control.
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}
impl Styled for ToggleGroupItem {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
impl ParentElement for ToggleGroupItem {
    fn extend(&mut self, children: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(children);
    }
}
#[derive(IntoElement)]
/// A controlled collection of toggles with roving keyboard focus.
pub struct ToggleGroup {
    id: ElementId,
    value: Vec<SharedString>,
    label: Option<SharedString>,
    disabled: bool,
    multiple: bool,
    joined: bool,
    items: Vec<ToggleGroupItem>,
    on_change: Option<ChangeHandler>,
    style: StyleRefinement,
}
impl ToggleGroup {
    /// Creates a `ToggleGroup` with a stable caller-owned ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: Vec::new(),
            label: None,
            disabled: false,
            multiple: false,
            joined: true,
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
    /// Allows multiple selected values instead of one.
    pub fn multiple(mut self, value: bool) -> Self {
        self.multiple = value;
        self
    }
    /// Removes gaps between adjacent toggles.
    pub fn joined(mut self, value: bool) -> Self {
        self.joined = value;
        self
    }
    /// Appends an option to the group.
    pub fn item(mut self, item: ToggleGroupItem) -> Self {
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
impl Styled for ToggleGroup {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
impl RenderOnce for ToggleGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let focus = RovingFocus::new(
            self.id.clone(),
            &self
                .items
                .iter()
                .map(|i| (i.id.clone(), self.disabled || i.disabled))
                .collect::<Vec<_>>(),
            None,
            window,
            cx,
        );
        let count = self.items.len();
        let children = self
            .items
            .into_iter()
            .enumerate()
            .map(|(index, item)| {
                let pressed = self.value.contains(&item.value);
                let disabled = self.disabled || item.disabled;
                let handle = focus.handles[index].as_ref();
                let change = self.on_change.clone();
                let mut next = if self.multiple {
                    self.value.clone()
                } else {
                    Vec::new()
                };
                next.retain(|value| value != &item.value);
                if !pressed {
                    next.push(item.value);
                }
                let toggle = gpui_kit::base::Toggle::new(item.id)
                    .pressed(pressed)
                    .disabled(disabled)
                    .when_some(item.label, |toggle, label| {
                        toggle.accessibility_label(label)
                    })
                    .when_some(handle, |toggle, h| {
                        toggle.track_focus(h).tab_stop(h.tab_stop)
                    })
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap(theme.space(1.))
                    .h(theme.space(8.))
                    .min_w(theme.space(8.))
                    .px(theme.space(2.5))
                    .rounded(if self.joined { px(0.) } else { theme.radius.lg })
                    .when(self.joined && index == 0, |t| t.rounded_l(theme.radius.lg))
                    .when(self.joined && index + 1 == count, |t| {
                        t.rounded_r(theme.radius.lg)
                    })
                    .border_1()
                    .border_color(colors.background.opacity(0.))
                    .text_size(theme.text(14.))
                    .text_color(colors.foreground)
                    .bg(if pressed {
                        colors.muted
                    } else {
                        colors.background.opacity(0.)
                    })
                    .focus_visible(move |s| s.border_color(colors.ring))
                    .when(disabled, |t| t.opacity(0.5).cursor_not_allowed())
                    .when(!disabled, |t| {
                        t.cursor_pointer().hover(move |s| s.bg(colors.muted))
                    })
                    .children(item.children)
                    .when_some(change, |toggle, handler| {
                        toggle.on_change(move |_, _, window, cx| handler(next.clone(), window, cx))
                    });
                apply_style(toggle, &item.style)
            })
            .collect::<Vec<_>>();
        let root = gpui_kit::base::ToggleGroup::new(self.id)
            .flex()
            .gap(theme.space(if self.joined { 0. } else { 2. }))
            .when_some(self.label, |root, label| root.aria_label(label))
            .on_key_down(move |event, window, cx| {
                focus.key(event, Some(gpui_kit::Axis::Horizontal), window, cx);
            })
            .children(children);
        apply_style(root, &self.style)
    }
}
