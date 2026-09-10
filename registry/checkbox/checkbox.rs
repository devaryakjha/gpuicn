//! Nova-styled controlled checkbox using GPUI Kit's interaction and accessibility.

use super::theme::{ThemeMode, UiTheme};
use gpui_icons::{LucideIcon, lucide};
use gpui_kit::base::{Checkbox as BaseCheckbox, CheckboxState};
use gpui_kit::{
    App, ClickEvent, ElementId, InteractiveElement as _, IntoElement, ParentElement as _,
    RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window,
    prelude::FluentBuilder as _,
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
        let colors = theme.colors;
        let selected = self.checked || self.indeterminate;
        let background = if selected {
            colors.primary
        } else {
            match theme.mode {
                ThemeMode::Light => colors.background.opacity(0.),
                ThemeMode::Dark => colors.input.opacity(0.30),
            }
        };
        let root = BaseCheckbox::new(self.id)
            .checked(self.checked)
            .indeterminate(self.indeterminate)
            .disabled(self.disabled)
            .when(self.read_only, |root| root.aria_description("Read only"))
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .size(theme.space(4.))
            .rounded(theme.radius.sm * (2. / 3.))
            .border_1()
            .border_color(if selected {
                colors.primary
            } else {
                colors.input
            })
            .bg(background)
            .focus_visible(move |style| style.border_color(colors.ring))
            .when(!self.disabled && !self.read_only, |root| {
                root.cursor_pointer()
            })
            .when(self.disabled, |root| root.opacity(0.5).cursor_not_allowed())
            .when_some(self.label, |root, label| root.accessibility_label(label))
            .when(selected, |root| {
                root.child(
                    lucide(if self.indeterminate {
                        LucideIcon::Minus
                    } else {
                        LucideIcon::Check
                    })
                    .size(theme.space(3.5))
                    .text_color(colors.primary_foreground),
                )
            })
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
