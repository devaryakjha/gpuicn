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

For applications that ask before loading remote images, `image_placeholder`
on both cards (and `thumbnail_placeholder` on `MessageEmbed`) takes an
`ImagePlaceholder` instead of the image. It fills the box the image would
take, so nothing moves when the image arrives: the card's inner width at the
given pixel size's aspect, or 16:9 when the size is unknown, with the same
large or thumbnail layout as `image`. The surface is `muted` with
`radius.md` and no border, holding the Lucide `Image` icon at the large icon
size, "Load image" in the xs style at medium weight, and the host in xs
`muted_foreground`; hover fills it with `accent`. It is a GPUI Kit button
named "Load image from {host}" with the focus ring, and click, Enter or Space
runs `on_load` once. Fetching stays with the application, which passes the
image through `image` on the next render. Tests cover the box sizes, the
thumbnail layout, Enter, and the single call.

Both cards are groups named by their title. Titles with a click handler are
GPUI Kit buttons with the link role, so they take focus, show the focus ring
and activate on Enter. gpuicn never fetches images; deciding which embeds to
show, collapse preferences and opening links stay with the application.

`QuotedMessage` is a quoted post, for pasted permalinks and forwards: the
author's small avatar (image or initials), name and time on one line with a
trailing Jump link, a muted "Originally posted in {channel}" line (left out
for direct messages), the body slot clamped to four lines (`max_lines`) with
a "Show more" toggle that appears only when the body is taller, and an
optional strip for the quoted post's files. It uses the card frame and
density of the other embeds and is named "Quoted message from {author}".
Interactive attachment buttons and menus are not supported yet.
