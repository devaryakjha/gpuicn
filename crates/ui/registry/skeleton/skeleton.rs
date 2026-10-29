//! Nova-styled loading placeholders.

use gpui_kit::{
    Animation, AnimationExt as _, App, ElementId, IntoElement, RenderOnce, Styled, Window, div,
    ease_in_out,
};

use super::theme::UiTheme;

/// One pulse cycle, matching Tailwind's `animate-pulse`.
const PULSE: std::time::Duration = std::time::Duration::from_secs(2);

/// A muted block that stands in for content while it loads.
///
/// Size it with `w`, `h` and `rounded` like any element. Skeletons are hidden from
/// assistive technology; announce loading with a nearby `Spinner` or label instead.
#[derive(IntoElement)]
pub struct Skeleton {
    style: gpui_kit::StyleRefinement,
    id: ElementId,
}

impl Skeleton {
    /// Creates a `Skeleton` with a stable caller-owned ID for its pulse animation.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            style: gpui_kit::StyleRefinement::default(),
            id: id.into(),
        }
    }
}

impl RenderOnce for Skeleton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx);
        let root = div()
            .w_full()
            .h(theme.space(4.))
            .flex_shrink_0()
            .rounded(theme.radius.md)
            .bg(theme.colors.muted);
        let root = super::theme::apply_style(root, &self.style);
        if theme.motion.reduced || cx.reduce_motion() {
            return root.into_any_element();
        }
        root.with_animation(
            self.id,
            Animation::new(PULSE).repeat().with_easing(ease_in_out),
            // Tailwind's pulse dips to half opacity halfway through each cycle.
            |root, delta| root.opacity(1. - 0.5 * (1. - (2. * delta - 1.).abs())),
        )
        .into_any_element()
    }
}

impl Styled for Skeleton {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}
