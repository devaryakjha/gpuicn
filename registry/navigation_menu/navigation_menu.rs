//! Navigation menus composed from Nova menus and link items.
use super::menubar::Menubar;
use gpui_kit::{ElementId, Role, SharedString};
/// Create a named navigation landmark; supply menus of `MenuItem::link` entries with caller-owned navigation handlers.
pub fn navigation_menu(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Menubar {
    let mut menu = Menubar::new(id, label);
    menu.role = Role::Navigation;
    menu
}
