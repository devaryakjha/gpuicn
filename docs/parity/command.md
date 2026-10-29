# Command parity

Visual reference: shadcn/ui 4.19.0 `command`, Neutral Nova. Implementation:
`crates/ui/registry/command/command.rs`, using GPUI Kit 0.8.1.

gpuicn draws the Nova palette: a search row with a Lucide search icon and a
borderless Input, then grouped 32px items with muted headings, leading icons and
trailing shortcut text, on the popover surface. `command_dialog_popup` puts the
same palette in a dialog surface for a Cmd/Ctrl+K quick switcher.

`CommandState` owns the items, the query and the highlight. Filtering matches
the label and keywords case-insensitively, by substring or by characters in
order (`gnrl` finds "general"), and keeps the caller's order; ungrouped items
come first, as in cmdk. For server-side search, call `set_filter(false)`,
listen for `CommandEvent::Query` and pass results to `set_items`. The list is
virtualized with uniform rows, so thousands of channels stay fast.

Up and Down move the highlight past disabled items and wrap. Enter or a click
emits `CommandEvent::Confirm` with the item's value. Escape clears a non-empty
query first; with an empty query it emits `CommandEvent::Cancel` and lets the
same Escape close a hosting dialog. The application decides what a confirmed
value does and closes the dialog itself.

Unlike GPUI Kit's `Command`, which dispatches GPUI actions and draws GPUI Kit
icons, this palette reports stable string values and draws Lucide icons from
gpui-icons. Highlighted rows are exposed as the active descendant of a listbox.
