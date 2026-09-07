#![allow(missing_docs)]
//! Nova-styled Checkbox Group backed by Base GPUI group state.

use std::rc::Rc;

use base_gpui::checkbox::CheckboxRoot;
use base_gpui::checkbox_group::{
    CheckboxGroup as BaseCheckboxGroup, CheckboxGroupValueChangeDetails,
};
use gpui::{
    AnyElement, App, ElementId, IntoElement, ParentElement as _, RenderOnce, SharedString, Styled,
    Window, div, prelude::FluentBuilder as _, px,
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
    label: Option<SharedString>,
}
impl CheckboxGroupItem {
    pub fn new(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self {
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
    pub fn label(mut self, value: impl Into<SharedString>) -> Self {
        self.label = Some(value.into());
        self
    }
    fn render(self, theme: &UiTheme) -> AnyElement {
        let colors = theme.colors;
        let mode = theme.mode;
        let font = theme.fonts.body.clone();
        let label = self.label.clone();
        let mut checkbox = CheckboxRoot::new()
            .id(self.id)
            .value(self.value)
            .disabled(self.disabled)
            .style_with_state(move |state, base| {
                let selected = state.checked || state.indeterminate;
                let square = div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .flex_shrink_0()
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
                        colors.input.opacity(0.30)
                    } else {
                        colors.background.opacity(0.)
                    })
                    .when(state.focused && !state.disabled, |base| {
                        super::theme::focus_outline(
                            base.border_color(colors.ring),
                            colors.ring.opacity(0.50),
                            gpui::Corners::all(px(4.)),
                        )
                    })
                    .when(selected, |base| {
                        base.child(
                            lucide(if state.indeterminate {
                                LucideIcon::Minus
                            } else {
                                LucideIcon::Check
                            })
                            .size(px(14.))
                            .text_color(colors.primary_foreground),
                        )
                    });
                base.flex()
                    .items_center()
                    .gap(px(8.))
                    .font_family(font.clone())
                    .text_size(px(14.))
                    .line_height(px(20.))
                    .text_color(colors.foreground)
                    .when(!state.disabled && !state.read_only, |base| {
                        base.cursor_pointer()
                    })
                    .when(state.disabled, |base| {
                        base.opacity(0.50).cursor_not_allowed()
                    })
                    .child(square)
                    .when_some(label.clone(), |base, label| {
                        base.child(gpui::Text::new_inaccessible(label))
                    })
            });
        if let Some(label) = self.aria_label.or(self.label) {
            checkbox = checkbox.aria_label(label);
        }
        checkbox.into_any_element()
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

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Context, Modifiers, Render, TestAppContext, VisualTestContext, point};
    use std::cell::RefCell;

    struct View {
        group_disabled: bool,
        item_disabled: bool,
        changes: Rc<RefCell<Vec<Vec<SharedString>>>>,
    }

    impl Render for View {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let changes = self.changes.clone();
            div().w(px(200.)).child(
                CheckboxGroup::new("group")
                    .disabled(self.group_disabled)
                    .on_value_change(move |values, _, _, _| changes.borrow_mut().push(values))
                    .item(
                        CheckboxGroupItem::new("updates", "updates")
                            .label("Updates")
                            .disabled(self.item_disabled),
                    ),
            )
        }
    }

    #[test]
    fn label_and_square_share_toggle_and_disabled_guards() {
        for theme in [UiTheme::neutral_light(), UiTheme::neutral_dark()] {
            for (group_disabled, item_disabled) in [(false, false), (true, false), (false, true)] {
                let mut cx = TestAppContext::single();
                cx.update(|cx| {
                    crate::init(cx);
                    UiTheme::set(cx, theme.clone());
                });
                let changes = Rc::new(RefCell::new(Vec::new()));
                let captured = changes.clone();
                let window = cx.add_window(move |_, _| View {
                    group_disabled,
                    item_disabled,
                    changes: captured,
                });
                let mut visual = VisualTestContext::from_window(window.into(), &cx);
                visual.simulate_click(point(px(40.), px(10.)), Modifiers::default());
                visual.run_until_parked();
                visual.simulate_click(point(px(8.), px(10.)), Modifiers::default());
                visual.run_until_parked();
                let changes = changes.borrow();
                if group_disabled || item_disabled {
                    assert!(
                        changes.is_empty(),
                        "disabled label or box changed the value"
                    );
                } else {
                    assert_eq!(*changes, vec![vec![SharedString::from("updates")], vec![]]);
                }
            }
        }
    }
}
