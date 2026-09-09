# Drawer parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/drawer/drawer.rs`, using GPUI Kit 0.6.1.

Drawer takes a retained DialogHandle and one of four edges. Its modal surface traps Tab traversal and restores focus after closing. The grip tracks a native drag and dismisses beyond its distance threshold. The application owns the content and save action.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
