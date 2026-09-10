//! Editable Nova selection using the shared Kit-backed collection state.
use super::select::{Mode, Select};
pub use super::select::{SelectEvent, SelectItem, SelectState};
use gpui_kit::Entity;

/// Search the options and commit a listed value; subscribe to the state's change events.
pub fn combobox(state: &Entity<SelectState>) -> Select {
    let mut select = Select::new(state);
    select.mode = Mode::Combobox;
    select
}
