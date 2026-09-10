//! Nova-styled controlled Switch using GPUI Kit.

use std::{rc::Rc, sync::Arc};

use gpui_kit::base::Switch as BaseSwitch;
use gpui_kit::{
    App, ClickEvent, ElementId, InteractiveElement as _, IntoElement, ParentElement as _,
    RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};

use super::theme::{ThemeMode, UiTheme, transition_value};

type ChangeHandler = Rc<dyn Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
/// A controlled Nova switch with a themed thumb transition.
pub struct Switch {
    style: gpui_kit::StyleRefinement,
    id: ElementId,
    checked: bool,
    disabled: bool,
    read_only: bool,
    aria_label: Option<SharedString>,
    on_change: Option<ChangeHandler>,
}

impl Switch {
    /// Creates a `Switch` with a stable caller-owned ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            style: gpui_kit::StyleRefinement::default(),
            id: id.into(),
            checked: false,
            disabled: false,
            read_only: false,
            aria_label: None,
            on_change: None,
        }
    }
    /// Sets the caller-owned checked state.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }
    /// Disables interaction and applies the disabled appearance.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Prevents user changes while preserving focus and reading.
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
    /// Sets the accessible name of the control.
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
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
        let root = BaseSwitch::new(self.id)
            .checked(self.checked)
            .disabled(self.disabled)
            .when(self.read_only, |root| root.aria_description("Read only"))
            .relative()
            .flex_shrink_0()
            .w(spacing * 8_f32)
            .h(spacing * 4.6_f32)
            .rounded_full()
            .border_1()
            .border_color(colors.background.opacity(0.))
            .bg(if self.checked {
                colors.primary
            } else {
                unchecked
            })
            .focus_visible(move |style| style.border_color(colors.ring).shadow(focus_ring.clone()))
            .when(!self.disabled && !self.read_only, |root| {
                root.cursor_pointer()
            })
            .when(self.disabled, |root| {
                root.opacity(0.50).cursor_not_allowed()
            })
            .when_some(self.aria_label, |root, label| {
                root.accessibility_label(label)
            })
            .child(AnimatedThumb {
                id: thumb_id,
                checked: self.checked,
            })
            .when(!self.read_only, |root| {
                root.when_some(self.on_change, |root, handler| {
                    root.on_change(move |checked, event, window, cx| {
                        handler(checked, event, window, cx)
                    })
                })
            });
        super::theme::apply_style(root, &self.style)
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

impl gpui_kit::Styled for Switch {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}
