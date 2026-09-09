# Accordion parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/accordion/accordion.rs`, using GPUI Kit 0.6.1.

Kit supplies the accordion parts; the application passes each expanded state and handles trigger clicks. gpuicn adds Nova borders, spacing, typography and chevrons. Single or multiple expansion is an application choice.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
