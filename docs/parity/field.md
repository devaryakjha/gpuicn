# Field parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`crates/ui/registry/field/field.rs`, using GPUI Kit 0.6.4.

`Field::new` keeps the existing retained InputState API. `Field::from_control`
composes the same label, required marker, description, disabled state and error
treatment around Input, Select, Combobox, Autocomplete and NumberField. Each
control supplies its real focus target and receives the same accessible name,
disabled state and invalid state. Validation and values remain application-owned;
there is no hidden form registry.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../internal/validation/gpui-kit-migration.md)
for the checks completed on this revision.

`Field::from_control` also takes a `Checkbox`, which sits before its label on
one row with the description or error under the label (a click on the label
toggles it), and a `RadioGroup`, which takes the label as the group's name.
`FieldControl::field_label_focus`, `field_label_click` and `field_inline` let
other controls choose the same behavior.
