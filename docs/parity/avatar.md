# Avatar parity

- Upstream: shadcn/ui `4.19.0` Nova Avatar.
- gpuicn: `registry/avatar/avatar.rs`.
- Base GPUI owns image loading and fallback timing. Avatar groups and badges remain composition-level elements for now.

The wrapper keeps Base GPUI's image node for loading and fallback state, but paints a rounded GPUI `Img` over it. GPUI does not inherit an image's corner radius from its container. Images use a cover crop and remain circular at each supported size. URL sources require the application's HTTP client; the web showcase provides one.
