# Popover parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/popover/popover.rs`, using GPUI Kit 0.6.1.

Kit Popover owns the retained popup state, trigger and positioning. The caller supplies content with a closure; gpuicn provides Nova trigger, surface and text helpers. Use native buttons as triggers without nesting another button inside them.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
