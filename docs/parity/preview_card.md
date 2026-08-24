# Preview Card parity

- Upstream: shadcn/ui Nova Hover Card at `ac60ef5c4db4265d71454dd9ecd3f93e255d7211`, `apps/v4/registry/bases/base/ui/hover-card.tsx` and `apps/v4/registry/styles/style-nova.css`.
- gpuicn: `registry/preview_card/preview_card.rs`.
- Difference types: platform, interaction, accessibility, and visual.

Base GPUI names the primitive `PreviewCard`; it keeps the delayed hover/focus lifecycle, safe polygon, outside dismissal, anchor collision handling, controlled state, and arrow. The wrapper maps only the public name and Nova surface.

Base GPUI anchors the whole trigger bounds, not individual inline client rects. Nova motion is also omitted because GPUI lacks equivalent transition styling.
