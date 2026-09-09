# Scroll Area parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/scroll_area/scroll_area.rs`, using GPUI Kit 0.6.1.

ScrollArea uses a native GPUI scroll viewport and a Kit scrollbar. It supports horizontal or vertical scrolling, wheel input, scrollbar dragging and keyboard arrows, Page Up/Down, Home and End. Content layout and viewport dimensions belong to the caller.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
