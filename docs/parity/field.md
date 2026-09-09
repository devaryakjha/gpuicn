# Field parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/field/field.rs`, using GPUI Kit 0.6.1.

Field takes the same retained InputState used by the editor. It supplies a clickable visible label, description, required marker, error text, and vertical or horizontal layout. Validation and values belong to the application; there is no hidden form registry.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
