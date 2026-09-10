# Dialog parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/dialog/dialog.rs`, using GPUI Kit 0.6.1.

A retained Kit DialogHandle owns visibility. Keep the host mounted while closed so it can restore focus. Tab and Shift+Tab wrap through popup controls, Escape and backdrop press request cancellation, and Enter activates the focused control. The application closes the handle after a successful save. Nested modal behavior and OS screen-reader operation still need platform validation.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
