# Field parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/field/field.rs`, using GPUI Kit 0.6.4.

`Field::new` keeps the existing retained InputState API. `Field::from_control`
composes the same label, required marker, description, disabled state and error
treatment around Input, Select, Combobox, Autocomplete and NumberField. Each
control supplies its real focus target and receives the same accessible name,
disabled state and invalid state. Validation and values remain application-owned;
there is no hidden form registry.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
