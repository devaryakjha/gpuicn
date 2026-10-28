//! Nova-styled semantic separator.

use gpui_kit::{
    App, Axis, ElementId, InteractiveElement as _, IntoElement, RenderOnce, Role,
    StatefulInteractiveElement as _, Styled, Window, div, px,
};

use super::theme::UiTheme;

#[derive(IntoElement)]
/// A Nova horizontal or vertical separator.
pub struct Separator {
    style: gpui_kit::StyleRefinement,
    id: ElementId,
    orientation: Axis,
}

impl Separator {
    /// Creates a `Separator` with a stable caller-owned ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            style: gpui_kit::StyleRefinement::default(),
            id: id.into(),
            orientation: Axis::Horizontal,
        }
    }
    /// Draws a horizontal separator.
    pub fn horizontal(mut self) -> Self {
        self.orientation = Axis::Horizontal;
        self
    }
    /// Draws a vertical separator.
    pub fn vertical(mut self) -> Self {
        self.orientation = Axis::Vertical;
        self
    }
}

impl RenderOnce for Separator {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let base =
            div()
                .id(self.id)
                .role(Role::Splitter)
                .aria_orientation(match self.orientation {
                    Axis::Horizontal => gpui_kit::accesskit::Orientation::Horizontal,
                    Axis::Vertical => gpui_kit::accesskit::Orientation::Vertical,
                });
        let base = match self.orientation {
            Axis::Horizontal => base.w_full().h(px(1.)),
            Axis::Vertical => base.h_full().w(px(1.)),
        };
        super::theme::apply_style(base.bg(theme.colors.border), &self.style)
    }
}

impl gpui_kit::Styled for Separator {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}
