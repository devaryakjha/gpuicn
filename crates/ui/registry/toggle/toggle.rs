//! Nova-styled controlled Toggle using GPUI Kit.

use std::rc::Rc;

use gpui_kit::base::Toggle as BaseToggle;
use gpui_kit::{
    AnyElement, App, ClickEvent, ElementId, InteractiveElement as _, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, Window, prelude::FluentBuilder as _,
};

use super::theme::UiTheme;

type ChangeHandler = Rc<dyn Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
/// Nova toggle surface styles.
pub enum ToggleVariant {
    #[default]
    /// The default Nova presentation.
    Default,
    /// Draw a border around the toggle.
    Outline,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
/// Nova toggle dimensions.
pub enum ToggleSize {
    /// Compact dimensions.
    Sm,
    #[default]
    /// The default Nova presentation.
    Default,
    /// Large dimensions.
    Lg,
}

#[derive(IntoElement)]
/// A controlled button that represents a pressed state.
pub struct Toggle {
    style: gpui_kit::StyleRefinement,
    id: ElementId,
    pressed: bool,
    disabled: bool,
    aria_label: Option<SharedString>,
    variant: ToggleVariant,
    size: ToggleSize,
    children: Vec<AnyElement>,
    on_change: Option<ChangeHandler>,
}

impl Toggle {
    /// Creates a `Toggle` with a stable caller-owned ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            style: gpui_kit::StyleRefinement::default(),
            id: id.into(),
            pressed: false,
            disabled: false,
            aria_label: None,
            variant: ToggleVariant::Default,
            size: ToggleSize::Default,
            children: Vec::new(),
            on_change: None,
        }
    }
    /// Sets the caller-owned pressed state.
    pub fn pressed(mut self, pressed: bool) -> Self {
        self.pressed = pressed;
        self
    }
    /// Disables interaction and applies the disabled appearance.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Sets the accessible name of the control.
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }
    /// Selects the Nova surface style.
    pub fn variant(mut self, variant: ToggleVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Selects the Nova control dimensions.
    pub fn size(mut self, size: ToggleSize) -> Self {
        self.size = size;
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

impl ParentElement for Toggle {
    fn extend(&mut self, children: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(children);
    }
}

impl RenderOnce for Toggle {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let spacing = theme.spacing.unit;
        let colors = theme.colors;
        let variant = self.variant;
        let (height, radius, text_size) = match self.size {
            ToggleSize::Sm => (28., theme.radius.md, 12.8),
            ToggleSize::Default => (32., theme.radius.lg, 14.),
            ToggleSize::Lg => (36., theme.radius.lg, 14.),
        };
        let pressed = self.pressed;
        let disabled = self.disabled;
        let base = BaseToggle::new(self.id).pressed(pressed).disabled(disabled);
        let base = base
            .flex()
            .items_center()
            .justify_center()
            .gap(spacing * 1_f32)
            .min_w(spacing * (height / 4.))
            .h(spacing * (height / 4.))
            .px(spacing * 2.5_f32)
            .rounded(radius)
            .border_1()
            .border_color(colors.background.opacity(0.0))
            .text_size(theme.text(text_size))
            .text_color(colors.foreground)
            .bg(if pressed {
                colors.muted
            } else {
                colors.background.opacity(0.)
            })
            .focus_visible(move |style| style.border_color(colors.ring))
            .when(disabled, |base| base.opacity(0.50).cursor_not_allowed())
            .when(!disabled, |base| {
                base.cursor_pointer()
                    .hover(move |style| style.bg(colors.muted))
            });
        let base = match variant {
            ToggleVariant::Default => base,
            ToggleVariant::Outline => base.border_1().border_color(colors.input),
        };
        let mut toggle = super::theme::apply_style(base, &self.style).children(self.children);
        if let Some(label) = self.aria_label {
            toggle = toggle.accessibility_label(label);
        }
        if let Some(handler) = self.on_change {
            toggle = toggle.on_change(move |pressed, details, window, cx| {
                handler(pressed, details, window, cx)
            });
        }
        toggle
    }
}

impl gpui_kit::Styled for Toggle {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}
