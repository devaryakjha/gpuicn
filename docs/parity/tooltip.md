# Tooltip parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/tooltip/tooltip.rs`, using GPUI Kit 0.6.1.

text_tooltip creates a themed view for the native GPUI tooltip attachment point. Attach it to one interactive trigger. Native pointer/focus behavior owns its visibility; gpuicn supplies the Nova tooltip surface.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
