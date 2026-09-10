# Fieldset parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/fieldset/fieldset.rs`, using GPUI Kit 0.6.1.

A named fieldset groups application-owned fields with a visible legend and description. gpuicn supplies layout and Nova typography; the application owns each field and its validation.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
