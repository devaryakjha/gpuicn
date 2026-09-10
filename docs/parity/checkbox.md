# Checkbox parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/checkbox/checkbox.rs`, using GPUI Kit 0.6.1.

The application passes checked state and handles on_change. Kit supplies the checkbox primitive; gpuicn adds the Nova indicator, focus border and disabled styling.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
