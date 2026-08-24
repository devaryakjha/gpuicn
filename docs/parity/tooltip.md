# Tooltip parity

- Upstream: shadcn/ui Nova at `ac60ef5c4db4265d71454dd9ecd3f93e255d7211`, `apps/v4/registry/bases/base/ui/tooltip.tsx` and `apps/v4/registry/styles/style-nova.css`.
- gpuicn: `registry/tooltip/tooltip.rs`.
- Difference types: platform, interaction, accessibility, and visual.

Base GPUI keeps the provider delay group, delayed hover/focus opening, collision handling, safe popup hover, controlled state, and disabled trigger guards. The wrapper leaves trigger appearance to its host control and applies Nova's inverse compact popup style.

The pinned Base GPUI Tooltip has no Arrow layer, so Nova's rotated tooltip arrow is not available. It also lacks live-region announcement and browser relationship attributes; the existing GPUI role remains the available accessibility signal.
