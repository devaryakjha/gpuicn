//! Nova-styled loading spinner.

use gpui_icons::{LucideIcon, lucide};
use gpui_kit::{
    Animation, AnimationExt as _, App, ElementId, InteractiveElement as _, IntoElement,
    ParentElement as _, RenderOnce, Rgba, Role, SharedString, StatefulInteractiveElement as _,
    Styled, Transformation, Window, div, percentage,
};

use super::theme::UiTheme;

/// One full turn of the spinner, matching Tailwind's `animate-spin`.
const TURN: std::time::Duration = std::time::Duration::from_secs(1);

/// A rotating loader icon announced as a status.
#[derive(IntoElement)]
pub struct Spinner {
    style: gpui_kit::StyleRefinement,
    id: ElementId,
    label: SharedString,
    color: Option<Rgba>,
}

impl Spinner {
    /// Creates a `Spinner` with a stable caller-owned ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            style: gpui_kit::StyleRefinement::default(),
            id: id.into(),
            label: "Loading".into(),
            color: None,
        }
    }

    /// Sets the accessible status text. Defaults to "Loading".
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// Sets the icon color. Defaults to the muted foreground.
    pub fn color(mut self, color: impl Into<Rgba>) -> Self {
        self.color = Some(color.into());
        self
    }
}

impl RenderOnce for Spinner {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx);
        let icon = lucide(LucideIcon::LoaderCircle)
            .size_full()
            .text_color(self.color.unwrap_or(theme.colors.muted_foreground));
        let icon = if theme.motion.reduced || cx.reduce_motion() {
            icon.into_any_element()
        } else {
            icon.with_animation(
                (self.id.clone(), "spin"),
                Animation::new(TURN).repeat(),
                |icon, delta| icon.with_transformation(Transformation::rotate(percentage(delta))),
            )
            .into_any_element()
        };
        let root = div()
            .id(self.id)
            .role(Role::Status)
            .aria_label(self.label)
            .size(theme.space(4.))
            .flex_shrink_0()
            .child(icon);
        super::theme::apply_style(root, &self.style)
    }
}

impl Styled for Spinner {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}
