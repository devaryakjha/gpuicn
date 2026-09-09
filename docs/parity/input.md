# Input parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/input/input.rs`, using GPUI Kit 0.6.1.

Input takes a retained Kit InputState. Kit owns editing, selection, history, clipboard and IME handling; gpuicn applies editor colors and the Nova input surface. Subscribe to InputEvent::Change and PressEnter in the owning view. Disabled and read-only presentation guards reach the editor. Native OS IME and screen-reader workflows require device checks.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
