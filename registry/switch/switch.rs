#![allow(missing_docs)]
//! Nova-styled Switch backed by Base GPUI toggle, focus, and keyboard behavior.

use std::rc::Rc;

use base_gpui::switch::{SwitchCheckedChangeDetails, SwitchRoot, SwitchThumb};
use gpui::{
    App, BoxShadow, ElementId, InteractiveElement as _, IntoElement, RenderOnce, SharedString,
    Styled, Window, prelude::FluentBuilder as _, px,
};

use super::theme::{ThemeMode, UiTheme};

type ChangeHandler =
    Rc<dyn Fn(bool, &mut SwitchCheckedChangeDetails, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct Switch {
    id: ElementId,
    default_checked: bool,
    checked: Option<bool>,
    disabled: bool,
    read_only: bool,
    aria_label: Option<SharedString>,
    on_checked_change: Option<ChangeHandler>,
}

impl Switch {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            default_checked: false,
            checked: None,
            disabled: false,
            read_only: false,
            aria_label: None,
            on_checked_change: None,
        }
    }
    pub fn default_checked(mut self, checked: bool) -> Self {
        self.default_checked = checked;
        self
    }
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }
    pub fn on_checked_change(
        mut self,
        handler: impl Fn(bool, &mut SwitchCheckedChangeDetails, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_checked_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Switch {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let unchecked = match theme.mode {
            ThemeMode::Light => colors.input,
            ThemeMode::Dark => colors.input.alpha(0.80),
        };
        let mut root = SwitchRoot::new()
            .id(self.id)
            .default_checked(self.default_checked)
            .disabled(self.disabled)
            .read_only(self.read_only)
            .relative()
            .flex_shrink_0()
            .style_with_state(move |state, base| {
                base.w(px(32.))
                    .h(px(18.4))
                    .rounded_full()
                    .border_1()
                    .border_color(colors.background.alpha(0.0))
                    .bg(if state.checked {
                        colors.primary
                    } else {
                        unchecked
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
            .child(SwitchThumb::new().style_with_state(move |state, base| {
                base.absolute()
                    .top(px(1.2))
                    .left(px(if state.root.checked { 16.8 } else { 1.2 }))
                    .size(px(16.))
                    .rounded_full()
                    .bg(match (theme.mode, state.root.checked) {
                        (ThemeMode::Dark, false) => colors.foreground,
                        (ThemeMode::Dark, true) => colors.primary_foreground,
                        _ => colors.background,
                    })
            }));
        if let Some(checked) = self.checked {
            root = root.checked(Some(checked));
        }
        if let Some(label) = self.aria_label {
            root = root.aria_label(label);
        }
        if let Some(handler) = self.on_checked_change {
            root = root.on_checked_change(move |checked, details, window, cx| {
                handler(checked, details, window, cx)
            });
        }
        root
    }
}
