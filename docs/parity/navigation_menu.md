# Navigation Menu parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/navigation_menu/navigation_menu.rs`, using GPUI Kit 0.6.1.

Navigation Menu composes the Menubar trigger and popup behavior with link items. The application handles link activation and routing through `on_click`. Arbitrary rich navigation panels are outside this API.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
