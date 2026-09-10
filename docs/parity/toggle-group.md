# Toggle Group parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/toggle_group/toggle_group.rs`, using GPUI Kit 0.6.1.

The application owns selected values and handles on_change. The group supports single or multiple selection and roving keyboard focus that skips disabled items. Kit toggle primitives supply each control.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
