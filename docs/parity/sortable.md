# Sortable parity

shadcn/ui has no sortable list. Implementation:
`crates/ui/registry/sortable/sortable.rs`, on GPUI's drag and drop with GPUI
Kit 0.9.0.

`Sortable` holds `SortableGroup`s of `SortableRow`s; each row's content comes
from a closure, so the drag preview can draw a copy of it at 70% opacity at
the row's width while the row itself dims. A 2px `ring` line marks where the
row will land; a group that is empty or collapsed fills with `accent` when a
row would drop into it, and takes the row at its end. With focus inside a
row, Alt+Shift+Up and Down move it one place, into the neighbouring group at
either end, and a status region announces "general moved to position 1 of 2
in Channels". `reorder_groups(true)` makes headers drag the same way and take
the same keys; the default header is then a focusable button.

Nothing animates, so reduced motion needs no case. The list never reorders
itself: `on_reorder` reports a `SortableMove` (`from` and `to` positions, with
`to` counted after the row has left) and `on_reorder_group` reports
`(from, to)`; `SortableMove::apply` and `move_group` apply them to the
application's own data. Row IDs are unique across the whole list, so a row
keeps its state and focus when it changes group. Group
headers are headings named by the group label.
