# Meter parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/meter/meter.rs`, using GPUI Kit 0.6.1.

Kit supplies the meter primitive and bounded value presentation. gpuicn applies the Nova track and fill. The application supplies the accessible name and value.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
