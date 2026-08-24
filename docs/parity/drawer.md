# Drawer parity

- Upstream: shadcn/ui Nova at `ac60ef5c4db4265d71454dd9ecd3f93e255d7211`, `apps/v4/registry/bases/base/ui/drawer.tsx` and `apps/v4/registry/styles/style-nova.css`.
- gpuicn: `registry/drawer/drawer.rs`.
- Difference types: platform, interaction, accessibility, and visual.

The wrapper keeps Base GPUI's drawer gesture, snap-point, nested-drawer, Escape, outside-press, focus-return, and modal handling. Its public root remains configurable for modal mode, focus trapping, snap callbacks, and nesting.

Nova's CSS transform physics, responsive width rules, bleed pseudo-element, and transition curves have no direct GPUI equivalent. The wrapper keeps Base GPUI's native gesture state and surfaces the correct edge radius and border for each direction.
