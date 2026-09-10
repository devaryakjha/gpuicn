# Progress parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/progress/progress.rs`, using GPUI Kit 0.6.1.

Kit supplies determinate or indeterminate progress semantics. gpuicn applies the Nova track and fill. The application supplies progress and its accessible label.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
