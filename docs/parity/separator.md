# Separator parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`crates/ui/registry/separator/separator.rs`, using GPUI Kit 0.6.4.

Kit supplies separator semantics. gpuicn applies a one-pixel Nova border color in horizontal or vertical orientation.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../internal/validation/gpui-kit-migration.md)
for the checks completed on this revision.
