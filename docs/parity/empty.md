# Empty parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`crates/ui/registry/empty/empty.rs`, using GPUI Kit 0.6.4.

`Empty` provides the Nova empty-state layout. `EmptyHeader` keeps media, title,
and description in a stable order; `EmptyContent` holds application-owned
actions or other supporting content. `EmptyMediaVariant::Icon` adds the compact
muted icon frame while the default variant leaves custom media unframed.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact responsive breakpoints and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../internal/validation/gpui-kit-migration.md)
for the checks completed on this revision.
