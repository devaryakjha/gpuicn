# Emoji Picker parity

shadcn/ui has no emoji picker. Implementation:
`crates/ui/registry/emoji_picker/emoji_picker.rs`, built on GPUI's
`uniform_list` and gpuicn's `Input`.

`EmojiPicker` draws a Nova popover surface: a search row, a tab per category,
a virtualized grid of nine 32px cells per row under each category heading, and
a footer that previews the highlighted emoji and its `:shortcode:`. Put it in a
popover anchored to a reaction or composer button and close the popover on
`EmojiPickerEvent::Pick` or `Cancel`; pass `bordered(false)` inside a popover
surface.

The application owns all of the data, and gpuicn ships none:

- `EmojiPickerState::new` takes `EmojiCategory`s of `Emoji` with a shortcode,
  a Unicode glyph and optional search keywords.
- `set_custom` adds a "Custom" category from `(name, ImageSource)` pairs,
  for server emoji the application has already resolved. The picker never
  fetches.
- `set_recents` lists shortcodes first as "Recently used"; names the catalog
  lacks are skipped. The application updates it after each pick.

Search filters once the query reaches two characters (`set_min_query`), as in
Mattermost; shorter queries show every category. Names that start with the
query rank before names and keywords that only contain it.

The search field keeps focus. Left and Right move through the grid in reading
order, Up and Down keep the column across category boundaries, Enter picks,
and Escape clears a query or, when it is empty, emits `Cancel` and lets the
same Escape close a hosting popover. A click on a tab jumps to that category.
The grid is a `ListBox` of `ListBoxOption` cells named `:shortcode:`, and the
tabs are a `TabList`.

Skin tones and a frequency-ranked "Frequently used" are left to a later
release; until then the application can order `set_recents` by frequency.
