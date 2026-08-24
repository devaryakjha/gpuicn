# Select parity

- Upstream: shadcn/ui `4.19.0` at `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`, `apps/v4/registry/bases/base/ui/select.tsx` and the Select section of `apps/v4/registry/styles/style-nova.css`.
- gpuicn: `registry/select/select.rs`.
- Difference types: platform, animation, and accessibility.

The port keeps Base GPUI's controlled and uncontrolled single or multiple selection, typeahead, keyboard navigation, item alignment, scrolling, and field integration. Nova styling covers the 32px trigger, value state, popup, item, group, indicator, separator, and scroll affordances.

The pinned GPUI build lacks browser relationship attributes, live value announcements, disabled and read-only attributes, and Nova's 100ms animation system. The port preserves the actual Base GPUI behavior instead of faking those browser features.
