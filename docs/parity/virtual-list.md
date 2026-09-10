# Virtual List

A native desktop composition using GPUI's pinned `uniform_list` engine and GPUI Kit's shared scrollbar. It is not a direct shadcn component port. Its typography, spacing, selected/hover colors, focus border and radius use the same gpuicn theme as Sidebar and other controls.

Keep `VirtualListState` in your application view. It stores identity/label/disabled metadata, selection, anchor, active row and scroll state; application records and loading stay outside. `replace_items` runs only when that metadata changes. Notify your view after application-driven changes. Interactive gestures update the retained state before `on_event`; the callback can replace selection or data through the same state.

Rows need unique stable IDs, never current positions. Duplicate IDs return an error without changing state. Surviving selections persist after refresh; removed or disabled IDs drop out. Focus on a removed row moves to its next surviving old neighbor, then its previous neighbor. The top visible identity and pixel offset survive reorder; if removed, its next surviving old neighbor becomes the new top. A range uses the anchor's current position and excludes disabled rows. Right-clicking a selected row keeps the selection; right-clicking another enabled row selects it before an existing ContextMenu opens. Activation handlers should resolve their stable IDs with `index_of` before acting.

The list has one keyboard tab stop and uses active-descendant focus for visible options. Arrows, Home/End and Page Up/Down move through enabled rows. Shift extends a range; Cmd/Ctrl toggles pointer selection or moves keyboard focus without changing selection. Space toggles the active row; Cmd/Ctrl+A selects all enabled rows in Multiple mode. Enter and double-click emit activation. Keep row content noninteractive apart from existing ContextMenu composition; place other controls outside the list.

Rows have one fixed height per list, defaulting to eight spacing units. `row_height` changes the density of every row together while retaining the top row and its pixel offset. Only the visible range plus GPUI's measurement row is rendered. The ID index remains O(n); row replacement is O(n), while small selections and each rendered row use indexed identity lookups. Resizing does not rebuild an O(n) height tree. There is no second virtualization or scrollbar engine.

Variable-height rows remain an explicit T05 gap. The pinned GPUI `list` implementation clears offscreen height hints on width changes (`elements/list.rs` prepaint), so its scrollbar can lose the estimated extent; it also rebuilds the item-height tree while resizing. Do not use this fixed-height component for wrapped variable-length diff text. That dependency needs separate qualification before tree/diff adoption.

A bounded height is required, usually a `flex_1().min_h_0()` parent inside a sized column. Use `empty(...)` for caller-owned loading, empty or error content; the list performs no I/O. `scroll_area_scrollbar_for` exposes the same scrollbar for other GPUI scroll targets.

Qualification is local and starts on macOS. Native 100,000-row measurements and actual pointer/keyboard checks belong in `docs/validation/virtual-list.md`; automated rendering alone does not establish full screen-reader, Windows or Linux support. This component remains unreleased.
