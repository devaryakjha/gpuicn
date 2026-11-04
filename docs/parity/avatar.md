# Avatar parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`crates/ui/registry/avatar/avatar.rs`, using GPUI Kit 0.6.4.

Kit Avatar owns image loading and fallback presentation. The application supplies an image URL, fallback text, accessible label and size; gpuicn applies the Nova shape and colors.

`status(AvatarStatus)` draws a presence dot over the bottom-right corner: 8,
10 or 12px for the small, default and large sizes, inside a ring of the
surface color (`background` by default; `ring_color` sets it, such as
`colors.sidebar` on a sidebar). Online, Away and Dnd fill it with the
`success`, `warning` and `destructive` tokens; Offline is a hollow
`muted_foreground` ring, so it reads without color; `Custom` takes any color
and its own words. The status joins the accessible name: "Ana Lopez, away".

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../internal/validation/gpui-kit-migration.md)
for the checks completed on this revision.
