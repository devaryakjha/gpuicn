//! Discoverable Nova textarea built on the shared Kit-backed input source.

pub use super::input::{Textarea, TextareaState};
use gpui_kit::Entity;

/// Creates a textarea using the caller's retained editing state.
pub fn textarea(state: &Entity<TextareaState>) -> Textarea {
    Textarea::new(state)
}
