//! Nova-styled controlled checkbox using GPUI Kit's interaction and accessibility.

use super::theme::{
    UiTheme,
    primitives::{control, disabled, focus},
};
use gpui_kit::base::{Checkbox as BaseCheckbox, CheckboxState};
use gpui_kit::{
    App, ClickEvent, ElementId, InteractiveElement as _, IntoElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, Window, prelude::FluentBuilder as _,
};
use std::rc::Rc;

type ChangeHandler = Rc<dyn Fn(bool, &ClickEvent, &mut Window, &mut App)>;

#[derive(IntoElement)]
/// A controlled Nova checkbox with optional mixed state.
pub struct Checkbox {
    style: gpui_kit::StyleRefinement,
    id: ElementId,
    checked: bool,
    indeterminate: bool,
    disabled: bool,
    read_only: bool,
    label: Option<SharedString>,
    on_change: Option<ChangeHandler>,
}

impl Checkbox {
    /// Creates a `Checkbox` with a stable caller-owned ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            style: Default::default(),
            id: id.into(),
            checked: false,
            indeterminate: false,
            disabled: false,
            read_only: false,
            label: None,
            on_change: None,
        }
    }
    /// Sets the caller-owned checked state.
    pub fn checked(mut self, value: bool) -> Self {
        self.checked = value;
        self
    }
    /// Shows mixed or unknown state.
    pub fn indeterminate(mut self, value: bool) -> Self {
        self.indeterminate = value;
        self
    }
    /// Disables interaction and applies the disabled appearance.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    /// Prevents user changes while preserving focus and reading.
    pub fn read_only(mut self, value: bool) -> Self {
        self.read_only = value;
        self
    }
    /// Sets the accessible name of the control.
    pub fn aria_label(mut self, value: impl Into<SharedString>) -> Self {
        self.label = Some(value.into());
        self
    }
    /// Reports a requested value change; retain the next value in the owning view.
    pub fn on_change(
        mut self,
        handler: impl Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Checkbox {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let focus_theme = theme.clone();
        let root = BaseCheckbox::new(self.id)
            .checked(self.checked)
            .indeterminate(self.indeterminate)
            .disabled(self.disabled)
            .when(self.read_only, |root| root.aria_description(super::theme::locale::text(cx, super::theme::UiText::ReadOnly)));
        let root = control::checkbox_box(root, &theme, self.checked, self.indeterminate)
            .focus_visible(move |style| focus::ring(style, &focus_theme))
            .when(!self.disabled && !self.read_only, |root| {
                root.cursor_pointer()
            })
            .when(self.disabled, disabled::style)
            .when_some(self.label, |root, label| root.accessibility_label(label))
            .when(!self.read_only, |root| {
                root.when_some(self.on_change, |root, handler| {
                    root.on_change(move |state, event, window, cx| {
                        handler(state == CheckboxState::Checked, event, window, cx)
                    })
                })
            });
        super::theme::apply_style(root, &self.style)
    }
}
impl Styled for Checkbox {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}

impl super::field::FieldControl for Checkbox {
    fn field_label_focus(&self, _: &App) -> Option<gpui_kit::FocusHandle> {
        None
    }

    fn field_label_click(&self) -> Option<super::field::FieldLabelClick> {
        if self.disabled || self.read_only {
            return None;
        }
        let handler = self.on_change.clone()?;
        let next = self.indeterminate || !self.checked;
        Some(Rc::new(move |event, window, cx| {
            handler(next, event, window, cx)
        }))
    }

    fn field_inline(&self) -> bool {
        true
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
            .mt(gpui_kit::px(2.))
            .into_any_element()
    }
}
