# Tooltip parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/tooltip/tooltip.rs`, using GPUI Kit 0.6.1.

`tooltip` attaches a Nova surface to one Kit button and shows it on pointer hover or keyboard focus. Escape dismisses it. Kit supplies the delayed overlay and viewport-aware placement. `text_tooltip` remains available for GPUI's pointer-only `.tooltip(...)` attachment.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
