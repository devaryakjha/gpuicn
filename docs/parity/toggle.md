# Toggle parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/toggle/toggle.rs`, using GPUI Kit 0.6.1.

The application passes pressed state and handles on_change. Kit supplies the toggle primitive; gpuicn adds Nova default/outline variants and focus and disabled styles.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
