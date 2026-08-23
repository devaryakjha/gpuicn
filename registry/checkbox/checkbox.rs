//! The shadcn Nova Checkbox visual port.
//!
//! Visual source: shadcn/ui 4.19.0 `checkbox.tsx` and `style-nova.css` at
//! `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`. Interaction and state come from
//! the pinned Base GPUI Checkbox primitives.

use std::rc::Rc;

use base_gpui::checkbox::{
    CheckboxCheckedChangeDetails, CheckboxIndicator, CheckboxRoot, CheckboxRootStyleState,
};
use gpui::ParentElement as _;
use gpui::{
    App, BoxShadow, ElementId, InteractiveElement as _, IntoElement, RenderOnce, SharedString,
    Styled, Window, prelude::FluentBuilder as _, px,
};
use gpui_icons::{LucideIcon, lucide};

use super::theme::{ThemeMode, UiTheme};

type CheckedChangeHandler =
    Rc<dyn Fn(bool, &mut CheckboxCheckedChangeDetails, &mut Window, &mut App) + 'static>;

/// A 16px styled Checkbox backed by Base GPUI state and actions.
#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    default_checked: bool,
    checked: Option<bool>,
    indeterminate: bool,
    disabled: bool,
    read_only: bool,
    aria_label: Option<SharedString>,
    on_checked_change: Option<CheckedChangeHandler>,
}

impl Checkbox {
    /// Creates an uncontrolled Checkbox with a caller-owned stable ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            default_checked: false,
            checked: None,
            indeterminate: false,
            disabled: false,
            read_only: false,
            aria_label: None,
            on_checked_change: None,
        }
    }

    /// Sets the initial value for an uncontrolled Checkbox.
    pub fn default_checked(mut self, checked: bool) -> Self {
        self.default_checked = checked;
        self
    }

    /// Makes the Checkbox controlled with the given value.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    /// Shows the mixed-state Minus indicator.
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }

    /// Prevents input and removes the Checkbox from tab order.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Prevents value changes while keeping normal visual styling.
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Sets the accessible name.
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// Runs after an accepted pointer or Space toggle request.
    pub fn on_checked_change(
        mut self,
        handler: impl Fn(bool, &mut CheckboxCheckedChangeDetails, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_checked_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Checkbox {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let mut root = CheckboxRoot::new()
            .id(self.id)
            .default_checked(self.default_checked)
            .indeterminate(self.indeterminate)
            .disabled(self.disabled)
            .read_only(self.read_only)
            .relative()
            .style_with_state(move |state, base| style_checkbox(base, state, &theme))
            .child(
                CheckboxIndicator::new()
                    .keep_mounted(true)
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .style_with_state(|state, base| {
                        base.opacity(if show_check(state.root) { 1.0 } else { 0.0 })
                    })
                    .child(lucide(LucideIcon::Check).size(px(14.0))),
            )
            .child(
                CheckboxIndicator::new()
                    .keep_mounted(true)
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .style_with_state(|state, base| {
                        base.opacity(if show_minus(state.root) { 1.0 } else { 0.0 })
                    })
                    .child(lucide(LucideIcon::Minus).size(px(14.0))),
            );

        if let Some(checked) = self.checked {
            root = root.checked(Some(checked));
        }
        if let Some(label) = self.aria_label {
            root = root.aria_label(label);
        }
        if let Some(handler) = self.on_checked_change {
            root = root.on_checked_change(move |checked, details, window, cx| {
                handler(checked, details, window, cx);
            });
        }

        root
    }
}

fn style_checkbox(base: gpui::Div, state: CheckboxRootStyleState, theme: &UiTheme) -> gpui::Div {
    let colors = theme.colors;
    let selected = state.checked || state.indeterminate;
    let background = if selected {
        colors.primary
    } else {
        match theme.mode {
            ThemeMode::Light => colors.background.alpha(0.0),
            ThemeMode::Dark => colors.input.alpha(0.30),
        }
    };
    let border = if selected {
        colors.primary
    } else {
        colors.input
    };

    base.flex()
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .w(px(16.0))
        .h(px(16.0))
        .rounded(px(4.0))
        .border_1()
        .border_color(border)
        .bg(background)
        .text_color(if selected {
            colors.primary_foreground
        } else {
            colors.foreground
        })
        .focus_visible(move |style| {
            style.border_color(colors.ring).shadow(vec![
                BoxShadow::new(px(0.0), px(0.0), colors.ring.alpha(0.50).into())
                    .spread_radius(px(3.0)),
            ])
        })
        .when(state.disabled, |base| {
            base.opacity(0.50).cursor_not_allowed()
        })
}

fn show_check(state: CheckboxRootStyleState) -> bool {
    state.checked && !state.indeterminate
}

fn show_minus(state: CheckboxRootStyleState) -> bool {
    state.indeterminate
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indicator_selection_is_unambiguous() {
        let unchecked = CheckboxRootStyleState::default();
        let checked = CheckboxRootStyleState::new(true, false, false, false, false, false);
        let mixed = CheckboxRootStyleState::new(false, false, false, false, true, false);

        assert!(!show_check(unchecked));
        assert!(!show_minus(unchecked));
        assert!(show_check(checked));
        assert!(!show_minus(checked));
        assert!(!show_check(mixed));
        assert!(show_minus(mixed));
    }
}
