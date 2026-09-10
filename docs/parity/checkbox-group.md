# Checkbox Group parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/checkbox_group/checkbox_group.rs`, using GPUI Kit 0.6.1.

The application owns the selected values and handles on_change. Each item has a stable ID, value, visible label and disabled flag. The group composes Kit checkbox controls under a named group.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
