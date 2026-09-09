#![allow(missing_docs)]
//! Nova-styled Switch backed by Base GPUI toggle, focus, and keyboard behavior.

use std::{rc::Rc, sync::Arc};

use base_gpui::switch::{SwitchCheckedChangeDetails, SwitchRoot, SwitchThumb};
use gpui::{
    App, ElementId, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    SharedString, Styled, Window, div, prelude::FluentBuilder as _, px,
};

use super::theme::{ThemeMode, UiTheme, transition_value};

type ChangeHandler =
    Rc<dyn Fn(bool, &mut SwitchCheckedChangeDetails, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct Switch {
    style: gpui::StyleRefinement,
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
            style: gpui::StyleRefinement::default(),
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
        let spacing = theme.spacing.unit;
        let colors = theme.colors;
        let unchecked = match theme.mode {
            ThemeMode::Light => colors.input,
            ThemeMode::Dark => colors.input.opacity(0.80),
        };
        let focus_ring = theme.focus_ring();
        let thumb_id = ElementId::NamedChild(Arc::new(self.id.clone()), "thumb-motion".into());
        let mut root = SwitchRoot::new()
            .id(self.id)
            .default_checked(self.default_checked)
            .disabled(self.disabled)
            .read_only(self.read_only)
            .relative()
            .flex_shrink_0()
            .style_with_state(move |state, base| {
                let base = {
                    let focus_ring = focus_ring.clone();
                    base.w(spacing * 8_f32)
                        .h(spacing * 4.6_f32)
                        .rounded_full()
                        .border_1()
                        .border_color(colors.background.opacity(0.0))
                        .bg(if state.checked {
                            colors.primary
                        } else {
                            unchecked
                        })
                        .focus_visible(move |style| {
                            style.border_color(colors.ring).shadow(focus_ring.clone())
                        })
                        .when(!state.disabled && !state.read_only, |base| {
                            base.cursor_pointer()
                        })
                        .when(state.disabled, |base| {
                            base.opacity(0.50).cursor_not_allowed()
                        })
                };
                super::theme::apply_style(base, &self.style)
            })
            .child(SwitchThumb::new().style_with_state(move |state, base| {
                base.absolute().inset_0().child(AnimatedThumb {
                    id: thumb_id.clone(),
                    checked: state.root.checked,
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

#[derive(IntoElement)]
struct AnimatedThumb {
    id: ElementId,
    checked: bool,
}
impl RenderOnce for AnimatedThumb {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let progress = transition_value(
            self.id,
            if self.checked { 1. } else { 0. },
            theme.motion.fast,
            window,
            cx,
        );
        let thumb = theme.space(4.);
        let inset = ((theme.space(4.6) - px(2.) - thumb) / 2.).max(px(0.));
        let travel = (theme.space(8.) - px(2.) - thumb - inset * 2.).max(px(0.));
        div()
            .absolute()
            .top(inset)
            .left(inset + travel * progress)
            .size(thumb)
            .rounded_full()
            .bg(match (theme.mode, self.checked) {
                (ThemeMode::Dark, false) => theme.colors.foreground,
                (ThemeMode::Dark, true) => theme.colors.primary_foreground,
                _ => theme.colors.background,
            })
    }
}

impl gpui::Styled for Switch {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
