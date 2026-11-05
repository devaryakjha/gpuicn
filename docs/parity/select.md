# Select parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`crates/ui/registry/select/select.rs`, using GPUI Kit 0.6.4.

Select takes retained SelectState with stable option values and labels. It opens a single-choice list, skips disabled options during keyboard navigation and commits on Enter. Escape closes and returns focus. Searchable and free-text presentations use the same state through Combobox and Autocomplete.
The `invalid` builder applies the destructive border directly or through
`Field::from_control`. The pinned Base Select has no accessibility-description
seam, so the control's accessible name also says “invalid value”; searchable
presentations expose the same state on their text editor.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../internal/validation/gpui-kit-migration.md)
for the checks completed on this revision.

`SelectState::set_items` replaces the items while open, for results fetched
as the person types, keeping the query, the value and the highlight when it
is still shown. `set_loading` shows a "Loading…" status row, and the trigger
keeps the picked item's label after the items change.
