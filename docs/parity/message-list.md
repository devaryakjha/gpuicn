# Message List parity

shadcn/ui has no virtualized transcript. Implementation:
`crates/ui/registry/message_list/message_list.rs`, wrapping GPUI Kit 0.8.1's
`MessageScroller`.

Kit supplies variable-height virtualization over GPUI's `list`, tail following
while the reader is at the bottom, anchor-preserving `prepend` for history
paging, `scroll_to_item`, and the `Log` accessibility role. `MessageListState`
is Kit's `MessageScrollerState`: the application keeps its messages and calls
`append`, `prepend`, `splice` and `remeasure_items` as they change.

gpuicn removes Kit's row padding so Message rows own their spacing, fades the
bottom edge into the background while scrolled up, and replaces Kit's icon jump
button with a Nova pill. The pill reads "Jump to latest", or "N new messages"
when the application passes an `unseen` count, which switches it to the primary
treatment.

GPUI's list measures rows as they are laid out. Give images and embeds their
final size up front so the list does not jump when they load. Width changes
remeasure rows; the anchor row stays in place.

Keyboard selection mode is opt-in. `track_focus` makes the transcript
focusable (a click focuses it), `selected` names the row the application
renders as `MessageHighlight::Selected`, and Up, Down and Escape report the
next row through `on_select`; the list scrolls it into view. Up with nothing
selected starts at the newest row. `selectable` skips rows such as dividers,
and `on_key` sees each key press first with the selected row, so the
application can claim keys like E to edit or R to reply.

Text selection across rows uses GPUI Kit's window selection. Wrap the window's
content in `markdown::selection_root` and render bodies with `markdown` or
`selectable_text`; a drag then selects across messages, and Cmd+C or Ctrl+C
copies the text in reading order.
