# Personalize gpuicn

The installed source belongs to your app. Start with the shadcn Nova defaults and change the application theme; no app model, router, or persistence layer is required.

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

Colors, fonts, radius, spacing, shadows, overlay color, typography, breakpoints and motion are app-owned tokens. `theme.space(2.)` means two spacing units; `theme.text(14.)` scales a 14px default text size. One-pixel borders, focus outlines, circular shapes, pointer coordinates, and caller-supplied pane limits retain their physical or semantic meaning. Check contrast and legibility when choosing a palette or dense layout.

## Tokens

| Group | Fields | Notes |
| --- | --- | --- |
| `theme.colors` | the shadcn palette plus `destructive_foreground`, `success(_foreground)`, `warning(_foreground)`, `info(_foreground)`, `overlay` | `from_shadcn_css` reads the matching `--success`, `--warning` and `--info` variables when a theme defines them |
| `theme.control()` | `background`, `border`, `hover`, `pressed`, `track`, `destructive_*`, `ring`, `invalid_ring`, `surface_border` | derived per mode once, so no component branches on `ThemeMode` |
| `theme.typography` | `xs` 12/16, `sm` 14/20, `base` 16/24, `lg` 18/28, `xl` 20/28 | `theme.text_style(theme.typography.sm)` applies `text_scale` to size and line height |
| `theme.radius` | `base`, `sm`, `md`, `lg`, `xl`, `two_xl`, `three_xl` | one `--radius` drives the scale, as in shadcn |
| `theme.shadows` | `sm`, `md`, `lg` | shared slices; apply with `.shadow(theme.shadows.md.to_vec())` |
| `theme.breakpoints` | `mobile` 768px, `compact` 640px | sidebar and sheet behavior switch here |
| `theme.motion` | `fast`, `normal`, `easing`, `reduced` | see Motion |

Every Kit widget color follows these tokens: `UiTheme::set` maps all of Kit's
`ThemeColor` fields, so a Kit table, tab bar or switch inside a themed app never
shows Kit's default palette. A test fails when a Kit color is left unmapped.

## Primitives

`ui::theme::primitives` holds the styling every component is built from, so a
custom component looks like the catalog without copying numbers:

```rust
use ui::theme::{UiTheme, primitives::{focus, surface, text}};

let theme = UiTheme::read(cx);
surface::popover(div(), &theme)          // radius, border, background, shadow and text size of every floating panel
    .child(text::muted(div(), &theme).child("Description"));
button.focus_visible(|style| focus::ring(style, &theme)); // the 3px outer ring on every control
```

- `text::{xs, sm, base, lg, xl, heading, mono, muted}` set size, line height and font.
- `icon::{xs, sm, md, lg, xl}` are the icon sizes (12 to 24px before spacing scale).
- `focus::{ring, invalid_ring, ring_for}` draw the shared focus ring through `focus_visible`.
- `disabled::{style, interactive}` apply the shared disabled look.
- `surface::{popover, menu, modal, sheet, toast, card, floating, tooltip, divider}` are the surfaces.
- `control::{frame, checkbox_box, radio_dot, toggle}` are the form-control frames.
- `clipped_image` and `round_image` clip pictures to a radius without the overflow workaround.
- `dismiss::on_outside` closes a layer on outside press or Escape; `hover_reveal` shows actions on hover without a re-render.

## Import a shadcn theme

Any shadcn theme CSS works, including the output of the shadcn theme picker and
tweakcn. Paste it into your app and build both modes:

```rust
const THEME_CSS: &str = include_str!("theme.css");

let light = UiTheme::from_shadcn_css(THEME_CSS, ThemeMode::Light)?;
let dark = UiTheme::from_shadcn_css(THEME_CSS, ThemeMode::Dark)?;
UiTheme::set(cx, if prefers_dark { dark } else { light });
```

Light reads `:root`; dark reads `:root` then `.dark`. Colors may be
`oklch()`, `hsl()`, bare HSL triplets from shadcn v3, `rgb()` or hex, and
`--radius` sets the radius scale. Variables the CSS omits keep the Neutral
defaults, and unrelated declarations such as fonts are ignored, so load and
set any custom fonts yourself. `UiTheme::switch` always restores the Neutral
palette; keep both imported themes and call `UiTheme::set` to change mode.

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

`presence(id, open, window, cx)` is the one enter/exit lifecycle: its sample's
`progress` runs 0→1 on open and back on close, and `should_render()` keeps the
element mounted until the exit finishes, so overlays fade and drawers slide
with the same timing. `transition_with` and `presence_with` take an explicit
duration and `UiEasing`; drawers use `UiEasing::Emphasized`. All of these go
through one policy that honors `theme.motion.reduced` and `cx.reduce_motion()`.

Keep pointer dragging and keyboard navigation immediate. Width changes run native layout; they are not compositor-only animations. Both `theme.motion.reduced` and `cx.reduce_motion()` disable movement while preserving state feedback. The pinned GPUI does not automatically wire the operating system's reduced-motion preference into this flag; the application must set it.

Closed modal content and dismissed notifications are removed by the host. This
version does not claim animated exits for every component.

## Source update note

The shared animation clock uses `web-time = "1.1"`, the same cross-platform clock used by GPUI. Keep that dependency in the consuming app; native `std::time::Instant::now()` panics in a browser WASM build.

Input, numeric input, selectors, slider and OTP controls now take retained Kit state.
Keep those entities and their event subscriptions in the owning view. Dialogs and
drawers take a retained `DialogHandle`; keep the host mounted to restore focus after
closing. The catalog's compile-checked examples show each new signature.
