# Slider parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/slider/slider.rs`, using GPUI Kit 0.6.1.

Slider takes a retained Kit SliderState configured with range, step and initial value. Kit owns pointer and keyboard changes, including range thumbs; gpuicn supplies the Nova track, range and thumb styles.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
