# Dropdown Menu parity

- Upstream: shadcn/ui `4.19.0` at `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`, `apps/v4/registry/bases/base/ui/dropdown-menu.tsx` and the Dropdown Menu section of `apps/v4/registry/styles/style-nova.css`.
- gpuicn: `registry/menu/menu.rs`.
- Difference types: platform, animation, and accessibility.

The port keeps Base GPUI's menu tree, pointer and keyboard activation, roving focus, typeahead, submenus, checkbox and radio state, positioning, and outside dismissal. The visible port applies Nova's popover, item, group-label, separator, check-indicator, and focus-highlight treatment.

Nova's 100ms fade and zoom animations are omitted because the pinned GPUI APIs do not expose equivalent presence transitions. Base GPUI also documents the unavailable `aria-haspopup`, `aria-controls`, and disabled-state builders; this port does not claim those browser attributes.
