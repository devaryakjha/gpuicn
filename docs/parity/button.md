# Button parity

- Upstream: shadcn/ui `4.19.0` at `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`, `apps/v4/registry/bases/base/ui/button.tsx` and the Button section of `apps/v4/registry/styles/style-nova.css`.
- gpuicn: `registry/button/button.rs`.
- Difference type: platform and interaction styling.

Nova shifts a pressed Button down by one pixel unless it opens a popup. The pinned GPUI revision cannot apply that descendant-style rule cleanly, and its `active` state is documented upstream as sticking every other click in `crates/gpui/examples/active_state_bug.rs`. gpuicn omits the pressed translation in v0.1 while retaining pointer, Enter, Space, disabled, hover, and focus-visible behavior through Base GPUI.

The visible impact is limited to the one-pixel pressed offset. GPUI also cannot restyle opaque child elements, so callers size icons explicitly and choose any icon-side padding. Re-review these differences when GPUI fixes the active state or adds descendant-aware styling. GitHub issue #18 and milestone `v0.1` own the decision. Native and GPUI/WASM preview evidence will be recorded before #18 closes.
