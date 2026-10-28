//! Right-click and Shift-F10 menus using the shared Nova menu state.
use super::menu::{Menu, MenuState};
use gpui_kit::{Entity, IntoElement, SharedString};
/// Attach a context menu to a focusable area. The caller owns its menu state.
pub fn context_menu(
    state: &Entity<MenuState>,
    label: impl Into<SharedString>,
    area: impl IntoElement,
) -> Menu {
    Menu::new(state, label).context_area(area)
}
