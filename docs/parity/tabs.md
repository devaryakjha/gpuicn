# Tabs parity

- Upstream: shadcn/ui `4.19.0` at `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`, `apps/v4/registry/bases/base/ui/tabs.tsx` and the Tabs section of `apps/v4/registry/styles/style-nova.css`.
- gpuicn: `registry/tabs/tabs.rs`.
- Difference type: platform layout and animation.

Base GPUI owns selection, roving focus, and arrow-key navigation. The port exposes the pinned `Default` and `Line` list treatments through `TabsVariant`; callers pass the same variant to their triggers. The selected line uses a bottom border rather than CSS pseudo-elements, and cross-panel motion is omitted pending GPUI transition support.
