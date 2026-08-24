# Combobox parity

- Upstream: shadcn/ui `4.19.0` at `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`, `apps/v4/registry/bases/base/ui/combobox.tsx` and the Combobox section of `apps/v4/registry/styles/style-nova.css`.
- gpuicn: `registry/combobox/combobox.rs`.
- Difference types: platform, animation, accessibility, and composition.

The port keeps Base GPUI's editable input, filtering, keyboard navigation, controlled and uncontrolled value axes, single and multiple selection, chips, clear actions, group and collection wiring, field integration, and popup positioning. Nova styles cover the input, popup, list, item, indicator, group, empty state, separator, and chips.

The website component composes shadcn InputGroup parts that do not exist as one Base GPUI primitive. The port keeps the Base GPUI input, trigger, clear, and chip parts separate so callers retain the native behavior. Browser `aria-activedescendant`, relationship attributes, live regions, disabled or read-only attributes, and Nova motion remain unavailable in the pinned GPUI surface.
