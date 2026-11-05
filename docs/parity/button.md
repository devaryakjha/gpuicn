# Button parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`crates/ui/registry/button/button.rs`, using GPUI Kit 0.6.4.

Kit Button supplies pointer, native keyboard and accessibility activation. gpuicn applies Nova variants and sizes. Disabled controls cannot activate. Enter and Space use native key down and key up behavior.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../internal/validation/gpui-kit-migration.md)
for the checks completed on this revision.

`tint` fills the button with one application color, such as an integration's
`#3f51b5`, composited over the background when translucent. The text is the
theme's `foreground` or `background`, whichever has the higher WCAG contrast
with the fill, so it stays readable in light and dark mode; hover mixes 14%
of the text color into the fill.
