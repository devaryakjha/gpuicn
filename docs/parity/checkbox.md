# Checkbox parity

- Upstream: shadcn/ui `4.19.0` at `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`, `apps/v4/registry/bases/base/ui/checkbox.tsx` and the Checkbox section of `apps/v4/registry/styles/style-nova.css`.
- gpuicn: `registry/checkbox/checkbox.rs`.
- Difference type: platform accessibility and hit target.

The pinned GPUI revision has no builder for `aria-disabled` or `aria-readonly`. Base GPUI keeps disabled controls out of tab order and blocks disabled and read-only changes, but assistive technology cannot inspect those two states. The 16px control also has no Nova mobile pseudo-element that expands its pointer hit target.

Mouse and Space behavior, checked and mixed states, focus-visible styling, and disabled and read-only input guards remain backed by Base GPUI. Revisit these gaps when GPUI exposes the missing accessibility states and pointer-only hit-target support.
