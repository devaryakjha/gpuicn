//! Nova hover-card surfaces using Kit's delayed hover lifecycle.
use super::theme::{UiTheme, primitives::surface};
pub use gpui_kit::base::{HoverCard, HoverCardState};
use gpui_kit::{App, Div, ElementId, InteractiveElement as _, Stateful, Styled, div};

/// Supply a trigger and a content builder to the Kit hover card.
pub fn preview_card(id: impl Into<ElementId>) -> HoverCard {
    HoverCard::new(id)
}
/// The 256px Nova preview surface.
pub fn preview_card_popup(id: impl Into<ElementId>, cx: &App) -> Stateful<Div> {
    let t = UiTheme::read(cx);
    surface::popover(div().id(id).w(t.space(64.)).p(t.space(2.5)), t)
}
