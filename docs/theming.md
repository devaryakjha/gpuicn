# Personalize gpuicn

The installed source belongs to your app. Start with the shadcn Nova defaults and change the application theme; no Bonsai model, router, or persistence layer is required.

```rust
use gpui::{px, rgb};
use ui::theme::{UiRadius, UiTheme};

let mut theme = UiTheme::neutral_light();
theme.colors.primary = rgb(0x2563eb);
theme.colors.primary_foreground = rgb(0xffffff);
theme.colors.ring = theme.colors.primary;
theme.radius = UiRadius::new(px(4.));
theme.spacing.unit = px(4.5); // default 4px; controls, icons, gaps and insets scale together
theme.text_scale = 1.0;      // typography stays independent of density
theme.motion.normal = std::time::Duration::from_millis(180);
UiTheme::set(cx, theme);     // repaints open windows, including retained child views
```

Colors, fonts, radius, spacing, shadows, overlay color, and motion are app-owned tokens. `theme.space(2.)` means two spacing units; `theme.text(14.)` scales a 14px default text size. One-pixel borders, focus outlines, circular shapes, pointer coordinates, and caller-supplied pane limits retain their physical or semantic meaning. Check contrast and legibility when choosing a palette or dense layout.

`UiTheme::switch` selects the pinned Neutral light/dark palette and preserves non-color customization. For a custom light/dark color pair, select your own palette and call `UiTheme::set`; the library cannot infer a dark palette from custom light colors.

## Per-instance changes

Wrapped controls implement GPUI `Styled`. Overrides refine the actual control after its default styling, without adding a layout or focus wrapper:

```rust
use gpui::{Styled, px};
use ui::button::Button;

Button::new("save").w(px(160.)).rounded(px(2.))
```

Base GPUI composition helpers expose Base's native builders. Structural properties the helper does not set remain directly configurable. **Base's `style_with_state` replaces its previous callback**; it is a complete custom skin, not an additive override. To make a small change while keeping the default skin, change the theme or edit that one style line in the installed helper. This is an upstream API limit, not a claim that every Base builder override wins over gpuicn defaults.

## Motion

`UiMotion` provides fast/normal durations, an easing choice, and a reduced-motion preference. The shared `transition_value(id, target, duration, window, cx)` starts at its target, retargets from its current value, and requests frames only while moving. Use stable IDs and retain expensive child entities. Switch thumbs and disclosure chevrons use this pattern; the desktop example uses it for sidebar width.

Keep pointer dragging and keyboard navigation immediate. Width changes run native layout; they are not compositor-only animations. Both `theme.motion.reduced` and `cx.reduce_motion()` disable movement while preserving state feedback. The pinned GPUI does not automatically wire the operating system's reduced-motion preference into this flag; the application must set it.

The pinned Base GPUI removes closed dialog/toast/disclosure content immediately. Full exit animations cannot preserve that content through a style callback, and gpuicn does not replace Base's focus, presence, or dismissal machinery to fake them.

## Source update note

The shared animation clock uses `web-time = "1.1"`, the same cross-platform clock used by GPUI. Keep that dependency in the consuming app; native `std::time::Instant::now()` panics in a browser WASM build.

Theme-aware layout helpers that previously had no arguments now take `cx`: backdrop/viewport/header, list/group, and positioner helpers are affected. Keep source files and their shared theme from the same revision, review existing edits before overwriting, and use the compile-checked usage shown in the catalog.
