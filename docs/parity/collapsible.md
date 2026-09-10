# Collapsible parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/collapsible/collapsible.rs`, using GPUI Kit 0.6.1.

The application passes open state and handles the trigger click. Kit supplies the disclosure host and trigger; gpuicn adds Nova spacing and typography.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
