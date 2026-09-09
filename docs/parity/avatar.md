# Avatar parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/avatar/avatar.rs`, using GPUI Kit 0.6.1.

Kit Avatar owns image loading and fallback presentation. The application supplies an image URL, fallback text, accessible label and size; gpuicn applies the Nova shape and colors.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
