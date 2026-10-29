# Attachment parity

shadcn/ui has no attachment card. gpuicn follows GPUI Kit 0.8.1's `Attachment`
states (pending, uploading, failed, complete), drawn with Nova tokens and Lucide
icons. Implementation: `crates/ui/registry/attachment/attachment.rs`.

A file attachment is a 240px card with a file-type icon tile, an ellipsized
name and a muted secondary line. While uploading, the secondary line shows the
percentage and a 2px primary bar runs along the bottom edge. A failed upload
gets a destructive border and text and, with `on_retry`, a retry button. An
image attachment is a 64px thumbnail with the same overlays and a remove button
in the corner.

`on_remove` and `on_retry` add focusable icon buttons announced with the file
name. `on_click` makes the card clickable for previews; keyboard users reach the
same action through the message's actions or context menu. `attachment_group`
lays cards out in a horizontally scrolling row for a composer's upload tray.
Uploading, file sizes and previews stay with the application.
