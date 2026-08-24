# Menubar parity

- Upstream: shadcn/ui `4.19.0` at `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`, `apps/v4/registry/bases/base/ui/menubar.tsx` and the Menubar section of `apps/v4/registry/styles/style-nova.css`.
- gpuicn: `registry/menubar/menubar.rs`.
- Difference types: platform, animation, and accessibility.

The port keeps Base GPUI's menubar roving focus, horizontal and vertical navigation, menu coordination, modal behavior, and hosted menu semantics. It applies Nova's compact 32px bar, trigger, popup, item, label, separator, and check styles.

Nova's animation classes and browser disabled-state attributes are not available in the pinned GPUI surface. Base GPUI does retain the interactive behavior and the `role=menubar` / `role=menuitem` access semantics it supports.
