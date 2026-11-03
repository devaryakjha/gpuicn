# Lightbox parity

shadcn/ui has no image viewer. Implementation:
`crates/ui/registry/lightbox/lightbox.rs`, using GPUI Kit 0.8.1's dialog host
for the modal layer and focus return.

`lightbox(id, &handle, &state, actions, window, cx)` mounts a full-window
viewer over a near-black backdrop, opened and closed with a `DialogHandle`
like `dialog`. Keep it mounted while closed. A top bar shows the file name, an
optional caption, "N of M", zoom out, the zoom percentage, zoom in, the
caller's `actions` (such as Download) and Close. With more than one image,
round previous and next buttons sit at the sides.

`LightboxState` holds `LightboxImage`s with an `ImageSource` the application
has already resolved, such as a cached file or an authenticated URL; the
viewer never fetches. Call `show(index)` for the clicked image before opening.
Each image starts fitted to the window. Zoom runs from 1× to 8×, and the image
pans only as far as its zoomed edges, so it never leaves the window.

Keyboard: Left and Right show the previous and next image, wrapping; `+` and
`-` zoom, `0` fits again, Shift with an arrow pans a zoomed image, and Escape
closes and returns focus to where it was. Mouse: the wheel zooms, dragging
pans a zoomed image, and a double-click toggles between fitted and 2×.
`LightboxEvent::Change` reports the index after paging.
