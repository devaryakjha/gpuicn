#![allow(missing_docs)]
//! Nova-styled Separator backed by Base GPUI accessibility wiring.

use base_gpui::separator::{Separator as BaseSeparator, SeparatorOrientation};
use gpui::{App, ElementId, IntoElement, RenderOnce, Styled, Window, px};

use super::theme::UiTheme;

pub use base_gpui::separator::SeparatorOrientation as Orientation;

#[derive(IntoElement)]
pub struct Separator {
    id: ElementId,
    orientation: SeparatorOrientation,
}

impl Separator {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            orientation: SeparatorOrientation::Horizontal,
        }
    }
    pub fn horizontal(mut self) -> Self {
        self.orientation = SeparatorOrientation::Horizontal;
        self
    }
    pub fn vertical(mut self) -> Self {
        self.orientation = SeparatorOrientation::Vertical;
        self
    }
}

impl RenderOnce for Separator {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        BaseSeparator::new()
            .id(self.id)
            .orientation(self.orientation)
            .style_with_state(move |state, base| match state.orientation {
                SeparatorOrientation::Horizontal => base.w_full().h(px(1.)).bg(theme.colors.border),
                SeparatorOrientation::Vertical => base.h_full().w(px(1.)).bg(theme.colors.border),
            })
    }
}
