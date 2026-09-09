# Preview Card parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/preview_card/preview_card.rs`, using GPUI Kit 0.6.1.

Kit HoverCard owns hover/focus timing, trigger and positioning. gpuicn supplies the Nova surface. The application provides trigger content and the preview closure.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
