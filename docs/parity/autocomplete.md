# Autocomplete parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/autocomplete/autocomplete.rs`, using GPUI Kit 0.6.1.

A retained SelectState owns a Kit InputState, query, highlighted option and selection. Typing filters the choices; arrows skip disabled options, Enter commits and Escape closes. This API selects one value and permits free text. Multiple values and chips are outside this API.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
