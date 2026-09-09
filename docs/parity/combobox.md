# Combobox parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/combobox/combobox.rs`, using GPUI Kit 0.6.1.

A retained SelectState owns a Kit text editor and one selected option. The query filters choices. Arrow keys skip disabled options, Enter commits and Escape closes and restores focus. Multiple selection and chips are outside this API.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
