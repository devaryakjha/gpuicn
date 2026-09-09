# Tabs parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/tabs/tabs.rs`, using GPUI Kit 0.6.1.

The application passes the selected value and renders its matching panel. The tab list keeps one Tab stop; arrows and Home/End select enabled tabs. Kit tab primitives supply roles and control interaction. Nova supports filled and line variants.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
