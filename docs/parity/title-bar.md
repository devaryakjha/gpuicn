# Title Bar parity

shadcn/ui has no window title bar. gpuicn ports the behavior of GPUI Kit
0.8.1's `TitleBar` with Nova styling. Implementation:
`crates/ui/registry/title_bar/title_bar.rs`.

Open the window with `title_bar_window_options()` so the title bar owns
dragging. The bar is 34px high on the sidebar surface with a bottom border.
Dragging empty space moves the window, and a double-click zooms it the
platform's way. On macOS the bar leaves room for the traffic lights, which the
system draws. On Windows and on client-decorated Linux windows it draws
minimize, maximize or restore, and close controls with Lucide icons; Windows
hit-tests them natively, and Linux handles the clicks, with `on_close` to
override closing. Under server-side Linux decorations and in the browser no
controls are drawn.

Application content such as a workspace name or a search button goes in the
bar as children. Interactive children take their own clicks; the space around
them stays draggable.
