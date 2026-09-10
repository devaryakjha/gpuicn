//! Controlled checkbox sets with clickable labels.
use super::theme::{ThemeMode, UiTheme, apply_style};
use gpui_icons::{LucideIcon, lucide};
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
                let radius = theme.radius.sm * (2. / 3.);
                let square = div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .flex_shrink_0()
                    .size(theme.space(4.))
                    .rounded(radius)
                    .border_1()
                    .border_color(if checked {
                        colors.primary
                    } else {
                        colors.input
                    })
                    .bg(if checked {
                        colors.primary
                    } else if theme.mode == ThemeMode::Dark {
                        colors.input.opacity(0.3)
                    } else {
                        colors.background.opacity(0.)
                    })
                    .when(focused, |d| {
                        super::theme::focus_outline(
                            d,
                            colors.ring.opacity(0.5),
                            gpui_kit::Corners::all(radius),
                        )
                    })
                    .when(checked, |d| {
                        d.child(
                            lucide(LucideIcon::Check)
                                .size(theme.space(3.5))
                                .text_color(colors.primary_foreground),
                        )
                    });
                let checkbox = gpui_kit::base::Checkbox::new(item.id)
                    .checked(checked)
                    .disabled(disabled)
                    .accessibility_label(item.label.clone())
                    .track_focus(&focus)
                    .flex()
                    .items_center()
                    .gap(theme.space(2.))
                    .font_family(theme.fonts.body.clone())
                    .text_size(theme.text(14.))
                    .line_height(theme.text(20.))
                    .text_color(colors.foreground)
                    .when(disabled, |d| d.opacity(0.5).cursor_not_allowed())
                    .when(!disabled, |d| d.cursor_pointer())
                    .child(square)
                    .child(gpui_kit::Text::new_inaccessible(item.label))
                    .when_some(change, |checkbox, handler| {
                        checkbox
                            .on_change(move |_, _, window, cx| handler(next.clone(), window, cx))
                    });
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
