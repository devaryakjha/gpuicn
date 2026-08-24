# Context Menu parity

- Upstream: shadcn/ui `4.19.0` at `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`, `apps/v4/registry/bases/base/ui/context-menu.tsx` and the Context Menu section of `apps/v4/registry/styles/style-nova.css`.
- gpuicn: `registry/context_menu/context_menu.rs`.
- Difference types: platform, gesture, animation, and accessibility.

The port keeps Base GPUI's cursor anchoring, right-click open, right-button release activation, modal backdrop, keyboard navigation, typeahead, submenus, and checkbox and radio state. Nova popover, item, group-label, separator, and check-indicator styles come from the shared menu treatment.

Touch long-press, browser ARIA relationship attributes, and Nova's 100ms fade and zoom motion are unavailable in the pinned GPUI/Base GPUI surface and are intentionally not claimed.
