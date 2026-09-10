# Button parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/button/button.rs`, using GPUI Kit 0.6.1.

Kit Button supplies pointer, native keyboard and accessibility activation. gpuicn applies Nova variants and sizes. Disabled controls cannot activate. Enter and Space use native key down and key up behavior.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
