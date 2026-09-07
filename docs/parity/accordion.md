# Accordion parity

- Upstream: shadcn/ui `4.19.0` at `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`, `apps/v4/registry/bases/base/ui/accordion.tsx` and the Accordion section of `apps/v4/registry/styles/style-nova.css`.
- gpuicn: `registry/accordion/accordion.rs`.
- Difference type: platform animation and icon composition.

Base GPUI supplies disclosure state, roving focus, pointer activation, and keyboard activation. The port keeps Nova spacing, dividers, typography, hover, disabled, and focus-visible styling. The styled trigger includes the state-dependent chevron. CSS height animations are not reproduced; the pinned GPUI Accordion mounts or unmounts its panel from typed presence state.
