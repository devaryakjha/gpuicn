# Toolbar parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/toolbar/toolbar.rs`, using GPUI Kit 0.6.1.

Toolbar composes Kit buttons and retained Kit InputState editors. One enabled item participates in Tab order; arrows and Home/End move between controls. Editing keys remain in a focused input. The application owns actions and editor subscriptions.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
