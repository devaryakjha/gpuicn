# Toast parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`crates/ui/registry/toast/toast.rs`, using GPUI Kit 0.6.4.

A retained ToastState owns Kit notification entries and expiry tasks. The application pushes or dismisses notifications by ID. gpuicn renders Nova titles, descriptions and dismissal controls; a timeout is optional.

`Notice::on_dismiss` reports why a notification left (`NoticeDismissal`:
Action, Cancel, Close, Timeout, Dismissed or Replaced; `by_reader` is true for
Cancel and Close), and `ToastState::contains(id)` says whether one is still
queued or on screen. `set_placement` moves the stack to a corner or edge
(bottom left by default) and `set_inset` sets its distance from the window
edges, such as to clear a composer.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../internal/validation/gpui-kit-migration.md)
for the checks completed on this revision.
