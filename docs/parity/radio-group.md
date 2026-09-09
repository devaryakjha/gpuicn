# Radio Group parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/radio_group/radio_group.rs`, using GPUI Kit 0.6.1.

The application owns one selected value and handles on_change. The group has one Tab stop; arrows and Home/End move and select while skipping disabled items. Kit radio primitives supply the individual controls.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
