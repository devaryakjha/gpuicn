# Markdown parity

shadcn/ui has no Markdown renderer. gpuicn's `markdown` returns GPUI Kit 0.8.1's
`TextView`, parsing CommonMark with GFM tables, task lists and strikethrough,
with Nova styling. Implementation: `crates/ui/registry/markdown/markdown.rs`.

Body text is 14px with a 1.45 line height. Code blocks sit in a bordered,
rounded muted frame in the theme's mono font and scroll sideways rather than
wrap; inline code gets a muted fill. Quotes have a left rule and muted text,
tables a rounded border and a muted header row. Links use the primary color.
`UiTheme::set` passes these colors and the light or dark highlight theme to GPUI
Kit, so a theme switch restyles rendered Markdown. Text is selectable.

Code blocks are syntax-highlighted when GPUI Kit's `syntect` feature is on: in
crate mode enable gpuicn's `syntect` feature, and with copied source add
`syntect` to the `gpui-kit` features in `Cargo.toml`. The browser preview has no
highlighter, so code there is plain.

`EntityLinks` turns links with a custom scheme into inline chips, so an app can
render mentions and channels: rewrite `@arya` to `[@arya](mention:arya)` before
rendering and add `EntityLinks::new("mention")`. `highlight` fills chips that
match, such as mentions of the reader, and `on_click` receives the target.
Which entities exist and how they resolve stay with the application.

`selection_root` mounts GPUI Kit's window text-selection layer and copies the
selection with Cmd+C or Ctrl+C, so a selection can span several Markdown views
and `selectable_text` runs, such as many messages. A focused text field keeps
its own copy shortcut.

Images never load by themselves. `MarkdownImages` resolves each image URL
through the application, usually from a decoded-image cache, and shows the alt
text in a muted frame until the image is available; `on_click` receives the
URL for opening a lightbox. `CustomEmoji` handles images with its own scheme,
such as `![:party:](emoji:party)`, drawing them at the height of the text with
the alt text kept for selection, copy and assistive technology. Add
`CustomEmoji` before `MarkdownImages`.
