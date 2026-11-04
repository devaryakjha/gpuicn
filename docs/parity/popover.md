# Popover parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`crates/ui/registry/popover/popover.rs`, using GPUI Kit 0.6.4.

Kit Popover owns the retained popup state, trigger and positioning. The caller supplies content with a closure; gpuicn provides Nova trigger, surface and text helpers. Use native buttons as triggers without nesting another button inside them.

`hover_popover(id, label)` is a popover that also opens on hover, for profile
cards on avatars and names. The trigger is one focusable button named by
`label` ("Priya Nair, online") with the focus ring; `trigger_style_with`
restyles it. Resting the pointer on it opens the popup after 500ms without
moving focus, and the pointer can move into the popup to use it; it closes
300ms after the pointer leaves both (`open_on_hover` sets both delays).
Click, Enter or Space opens it and moves focus into it, a non-modal dialog,
and it then stays open until Escape, a click outside or the trigger again;
Escape returns focus to the trigger. `on_open_change` reports every change.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../internal/validation/gpui-kit-migration.md)
for the checks completed on this revision.
