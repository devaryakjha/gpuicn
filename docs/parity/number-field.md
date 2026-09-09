# Number Field parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/number_field/number_field.rs`, using GPUI Kit 0.6.1.

NumberField takes a Kit InputState configured with min, max and step. Kit NumberInput and step actions own parsing and numeric changes; gpuicn composes the shared Input and Nova stepper buttons. Disabled and read-only state guard editing and stepping. Pointer scrubbing is outside this API.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
