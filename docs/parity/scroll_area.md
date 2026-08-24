# Scroll Area parity

- Upstream: shadcn/ui `4.19.0` at `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`, `apps/v4/registry/bases/base/ui/scroll-area.tsx` and the Scroll Area section of `apps/v4/registry/styles/style-nova.css`.
- gpuicn: `registry/scroll_area/scroll_area.rs`.
- Difference type: platform scrollbar implementation.

Base GPUI owns scroll offsets, wheel input, drag and track-click behavior, focusability, and accessibility scroll actions. The visual port keeps Nova's ten-pixel rounded thumb and transparent track. GPUI exposes scrollbar state directly instead of browser CSS transitions, so the thumb shifts to the muted-foreground color while scrolling rather than animating CSS opacity.
