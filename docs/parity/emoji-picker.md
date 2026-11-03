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
- `set_frequent` and `set_recents` list shortcodes first, as "Frequently
  used" and "Recently used"; names the catalog lacks are skipped, and a
  skin-tone variant's name finds its base emoji. The application updates them
  after each pick.
- `Emoji::skin_tones` adds the five tone variants, each with its own
  shortcode. The footer's six swatches choose the tone, emitting
  `EmojiPickerEvent::SkinTone` so the application can save it, and
  `set_skin_tone` restores it. Emoji with variants show and report the chosen
  tone; others are unchanged.

Search filters once the query reaches two characters (`set_min_query`), as in
Mattermost; shorter queries show every category. Names that start with the
query rank before names and keywords that only contain it.

The search field keeps focus. Left and Right move through the grid in reading
order, Up and Down keep the column across category boundaries, Enter picks,
and Escape clears a query or, when it is empty, emits `Cancel` and lets the
same Escape close a hosting popover. A click on a tab jumps to that category.
The grid is a `ListBox` of `ListBoxOption` cells named `:shortcode:`, and the
tabs are a `TabList`.

The skin-tone swatches are a `RadioGroup` of named `RadioButton`s.
