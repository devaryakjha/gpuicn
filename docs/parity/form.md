# Form parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/form/form.rs`, using GPUI Kit 0.6.1.

The form helper supplies a named semantic group and layout. The application owns retained input states, validation errors and submission. Subscribe to InputEvent::PressEnter and use the same submit handler as the submit button. There is no implicit serialization or validation context.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
