# Popover parity

- Upstream: shadcn/ui Nova at `ac60ef5c4db4265d71454dd9ecd3f93e255d7211`, `apps/v4/registry/bases/base/ui/popover.tsx` and `apps/v4/registry/styles/style-nova.css`.
- gpuicn: `registry/popover/popover.rs`.
- Difference types: platform, interaction, accessibility, and visual.

Base GPUI supplies controlled state, anchor collision handling, outside press dismissal, modal support, focus tracking, and the optional arrow. The wrapper uses its 4px Nova side offset and visual surface only.

The Nova open/close fade, zoom, and directional slide transitions are omitted. GPUI has no browser relationship attributes or matching motion system; callers should keep the popup `aria_label` explicit.
