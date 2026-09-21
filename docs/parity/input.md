# Input parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/input/input.rs`, using GPUI Kit 0.6.4.

Input and Textarea take retained Kit editing states. Kit owns editing, selection,
history, clipboard and IME handling; gpuicn applies editor colors and the Nova
surface. Subscribe to `InputEvent::Change` and `PressEnter` in the owning view.
`on_paste` can consume image or file clipboard payloads before normal text
insertion on native platforms; it is skipped for disabled and read-only
controls. Browser image/file paste needs async clipboard permission and remains
out of scope. Native OS IME and screen-reader workflows require device checks.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
