# Menubar parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/menubar/menubar.rs`, using GPUI Kit 0.6.1.

Menubar composes retained MenuState entities. It maintains one Tab stop across triggers, moves between them with Left/Right and switches open menus on hover. Each popup uses Menu keyboard navigation and focus restoration.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
