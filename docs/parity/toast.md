# Toast parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/toast/toast.rs`, using GPUI Kit 0.6.1.

A retained ToastState owns Kit notification entries and expiry tasks. The application pushes or dismisses notifications by ID. gpuicn renders Nova titles, descriptions and dismissal controls; a timeout is optional.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
