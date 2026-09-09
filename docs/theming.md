# Personalize gpuicn

The installed source belongs to your app. Start with the shadcn Nova defaults and change the application theme; no Bonsai model, router, or persistence layer is required.

```rust
use gpui_kit::{px, rgb};
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
use gpui_kit::{Styled, px};
use ui::button::Button;

Button::new("save").w(px(160.)).rounded(px(2.))
```

Composition helpers expose GPUI Kit builders. Use the theme for shared changes,
`Styled` for a control's geometry, or edit the installed source for a custom skin.
Input editing colors come from `UiTheme` and update with the surrounding controls.

## Motion

`UiMotion` provides fast/normal durations, an easing choice, and a reduced-motion preference. The shared `transition_value(id, target, duration, window, cx)` starts at its target, retargets from its current value, and requests frames only while moving. Use stable IDs and retain expensive child entities. Switch thumbs and disclosure chevrons use this pattern; the desktop example uses it for sidebar width.

Keep pointer dragging and keyboard navigation immediate. Width changes run native layout; they are not compositor-only animations. Both `theme.motion.reduced` and `cx.reduce_motion()` disable movement while preserving state feedback. The pinned GPUI does not automatically wire the operating system's reduced-motion preference into this flag; the application must set it.

Closed modal content and dismissed notifications are removed by the host. This
version does not claim animated exits for every component.

## Source update note

The shared animation clock uses `web-time = "1.1"`, the same cross-platform clock used by GPUI. Keep that dependency in the consuming app; native `std::time::Instant::now()` panics in a browser WASM build.

Input, numeric input, selectors, slider and OTP controls now take retained Kit state.
Keep those entities and their event subscriptions in the owning view. Dialogs and
drawers take a retained `DialogHandle`; keep the host mounted to restore focus after
closing. The catalog's compile-checked examples show each new signature.
