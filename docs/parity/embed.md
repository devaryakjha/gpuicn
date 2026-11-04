# Embed parity

shadcn/ui has no embed card. gpuicn draws Mattermost's two post embeds with
Nova card tokens and the `Message` density steps. Implementation:
`crates/ui/registry/embed/embed.rs`, using GPUI Kit 0.9.0.

`LinkPreview` is an Open Graph preview: a muted site name, a semibold title
clamped to two lines, a muted description clamped to three, and an optional
image. The image is large below the text, or an 80px thumbnail on the right
when it is narrower than 150px or roughly square (`LinkPreviewImageLayout`
overrides the choice). The caller passes the image's pixel size, so the card
reserves the box before the bytes arrive, as `Attachment::dimensions` does.
`on_toggle` adds a chevron named "Collapse preview" or "Expand preview" that
hides the image; `on_remove` adds a "Remove preview" button. The card is at
most 480px wide.

`MessageEmbed` is the integration attachment: a 4px colored bar (any color,
a hex string, or `good`, `warning` and `danger` mapped to the status tokens),
pretext above the card, an author line with a small round icon, a title, a
body slot for the application's Markdown, a fields grid with short fields two
to a row, a large image, a thumbnail and a footer with icon and time.

Both cards are groups named by their title. Titles with a click handler are
GPUI Kit buttons with the link role, so they take focus, show the focus ring
and activate on Enter. gpuicn never fetches images; deciding which embeds to
show, collapse preferences and opening links stay with the application.
Interactive attachment buttons and menus are not supported yet.
