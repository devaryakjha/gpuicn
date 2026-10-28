//! Nova text suggestions with optional free-text commits.
use super::select::{Mode, Select};
pub use super::select::{SelectEvent, SelectItem, SelectState};
use gpui_kit::Entity;

/// Search suggestions or press Enter to commit text when no suggestion matches.
pub fn autocomplete(state: &Entity<SelectState>) -> Select {
    let mut select = Select::new(state);
    select.mode = Mode::Autocomplete;
    select
}
