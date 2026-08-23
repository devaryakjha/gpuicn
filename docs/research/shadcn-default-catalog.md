# Pin the shadcn default visual catalog

Research date: 2026-08-24

## Decision

Pin the first GPUI visual port to **shadcn/ui `shadcn@4.19.0` at commit `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`**. This was the latest stable, signed release when checked. The tag and commit are shown on the [official release page](https://github.com/shadcn-ui/ui/releases/tag/shadcn%404.19.0), published 2026-08-21.

Use this exact default design tuple:

```text
base: Base UI
style: Nova
base color: Neutral
theme: Neutral
chart color: Neutral
icons: Lucide
body font: Geist
heading font: inherit
radius: default
menu accent: subtle
menu color: default
direction: LTR
```

The CLI itself describes `--defaults` as `--template=next --preset=base-nova` and resolves that option through `DEFAULT_PRESETS.nova` ([pinned init source](https://github.com/shadcn-ui/ui/blob/1773ecfeeb4a04366978d353e69b5c7ded78dcb2/packages/shadcn/src/commands/init.ts#L140-L145), [resolution path](https://github.com/shadcn-ui/ui/blob/1773ecfeeb4a04366978d353e69b5c7ded78dcb2/packages/shadcn/src/commands/init.ts#L495-L505)). The pinned Nova preset supplies the rest of the tuple ([official preset source](https://github.com/shadcn-ui/ui/blob/1773ecfeeb4a04366978d353e69b5c7ded78dcb2/packages/shadcn/src/preset/defaults.ts#L3-L18)). Base UI became the official default for new projects in July 2026 ([official announcement](https://ui.shadcn.com/docs/changelog/2026-07-base-ui-default)).

This is a **visual baseline**, not an API baseline. Base UI and React implementation details do not transfer to GPUI. Nova's component geometry, type, colors, state treatment, and motion are the reference.

## What “default” means

“Default” is overloaded in shadcn:

- The old style named `default` is deprecated in favor of `new-york` ([components.json docs](https://ui.shadcn.com/docs/components-json#style)). It is not the current CLI default.
- Vega is the classic shadcn look, but it is a selectable style, not the current `--defaults` result.
- The current non-interactive CLI default is **Base UI + Nova**.
- Rhea, Vega, New York, and the other styles are out of this initial pin. Add them through later, explicit style milestones.

Pinning the commit matters because the website docs and `main` keep changing. A future upstream sync should be an intentional issue that changes this commit and records the catalog diff.

## Registry catalog

The pinned Base UI registry manifest contains **63 unique `registry:ui` entries** ([official manifest](https://github.com/shadcn-ui/ui/blob/1773ecfeeb4a04366978d353e69b5c7ded78dcb2/apps/v4/registry/bases/base/ui/_registry.ts)). Sixty have a GPUI visual or interaction surface and form the pinned visual catalog:

1. `accordion`
2. `alert`
3. `alert-dialog`
4. `aspect-ratio`
5. `attachment`
6. `avatar`
7. `badge`
8. `breadcrumb`
9. `bubble`
10. `button`
11. `button-group`
12. `calendar`
13. `card`
14. `carousel`
15. `chart`
16. `checkbox`
17. `collapsible`
18. `combobox`
19. `command`
20. `context-menu`
21. `dialog`
22. `drawer`
23. `dropdown-menu`
24. `empty`
25. `field`
26. `hover-card`
27. `input`
28. `input-group`
29. `input-otp`
30. `item`
31. `kbd`
32. `label`
33. `marker`
34. `menubar`
35. `message`
36. `message-scroller`
37. `native-select`
38. `navigation-menu`
39. `pagination`
40. `popover`
41. `progress`
42. `questionnaire`
43. `radio-group`
44. `resizable`
45. `scroll-area`
46. `select`
47. `separator`
48. `sheet`
49. `sidebar`
50. `skeleton`
51. `slider`
52. `spinner`
53. `switch`
54. `table`
55. `tabs`
56. `textarea`
57. `toast`
58. `toggle`
59. `toggle-group`
60. `tooltip`

### Registry entries excluded from the GPUI visual catalog

- `form`: the Base UI manifest declares an empty compatibility entry with no file or visual surface ([pinned manifest lines](https://github.com/shadcn-ui/ui/blob/1773ecfeeb4a04366978d353e69b5c7ded78dcb2/apps/v4/registry/bases/base/ui/_registry.ts#L423-L426)). Form layout and validation visuals remain covered by `field`, `label`, inputs, controls, and examples.
- `direction`: a re-export of Base UI's React direction provider, with no visual surface ([manifest entry](https://github.com/shadcn-ui/ui/blob/1773ecfeeb4a04366978d353e69b5c7ded78dcb2/apps/v4/registry/bases/base/ui/_registry.ts#L1028-L1043), [source](https://github.com/shadcn-ui/ui/blob/1773ecfeeb4a04366978d353e69b5c7ded78dcb2/apps/v4/registry/bases/base/ui/direction.tsx)). GPUI should handle direction through its own platform API when that work is scheduled.
- `sonner`: a React/Next wrapper around the third-party `sonner` package and `next-themes` ([manifest entry](https://github.com/shadcn-ui/ui/blob/1773ecfeeb4a04366978d353e69b5c7ded78dcb2/apps/v4/registry/bases/base/ui/_registry.ts#L797-L814)). The visual port uses the first-party `toast` catalog item instead.

These exclusions remove React-only adapters, not user-visible design. The resulting count is **60 visual catalog items**.

## Not catalog items

Exclude these from the pinned component catalog even though the documentation may show them alongside components:

- **Blocks, page templates, and app templates:** composed products, not `registry:ui` primitives. They can be a later effort.
- **Data Table, Date Picker, and Typography:** documentation patterns/examples composed from registry items; they are not entries in the pinned Base UI `registry:ui` manifest. They may become example pages later, but they must not inflate component parity counts.
- **Examples:** website preview fixtures, not separate installable components. Each ported component can still have an interactive website preview as its acceptance surface.
- **React-only helpers and hooks:** implementation aids do not belong in a visual GPUI port.

The official registry documentation defines `registry:ui` as a reusable UI component and keeps blocks, hooks, libraries, and other item types distinct ([registry examples](https://ui.shadcn.com/docs/registry/examples#registryui)).

## Initial tokens and style source

Treat the token names as the durable contract and their pinned Neutral values as the initial theme:

- Surface/text pairs: `background`/`foreground`, `card`/`card-foreground`, `popover`/`popover-foreground`.
- Intent pairs: `primary`/`primary-foreground`, `secondary`/`secondary-foreground`, `muted`/`muted-foreground`, `accent`/`accent-foreground`.
- Controls/status: `destructive`, `border`, `input`, `ring`.
- Data: `chart-1` through `chart-5`.
- Shape: `radius`, pinned to `0.625rem` before derived component radii.
- Sidebar: `sidebar`, `sidebar-foreground`, `sidebar-primary`, `sidebar-primary-foreground`, `sidebar-accent`, `sidebar-accent-foreground`, `sidebar-border`, `sidebar-ring`.

The exact light and dark OKLCH values live in the pinned [Neutral theme source](https://github.com/shadcn-ui/ui/blob/1773ecfeeb4a04366978d353e69b5c7ded78dcb2/apps/v4/registry/themes.ts#L3-L77). Nova's exact per-component spacing, size, radius, border, state, and motion rules live in the pinned [Nova style source](https://github.com/shadcn-ui/ui/blob/1773ecfeeb4a04366978d353e69b5c7ded78dcb2/apps/v4/registry/styles/style-nova.css). Those two files, plus each item's pinned component source and available preview/example, are the visual source of truth.

Do not port Tailwind class strings or React props as an API. Translate their rendered values and states into GPUI-native theme tokens and component behavior.

## Provenance and license

The shadcn/ui repository is MIT licensed. Its license permits use, modification, publication, and distribution, but requires the copyright and permission notice in copies or substantial portions ([pinned license](https://github.com/shadcn-ui/ui/blob/1773ecfeeb4a04366978d353e69b5c7ded78dcb2/LICENSE.md#L1-L20)). Record this pinned commit in project attribution and retain the MIT notice wherever copied source is substantial.

That MIT license does **not** automatically cover third-party dependencies or assets named by shadcn. Lucide icons, Geist, Base UI, and any component-specific dependency need their own license/provenance check. The separate `gpui-icons` effort should own Lucide licensing and attribution; this ticket only identifies Lucide as the default icon family.

## Acceptance rule for later implementation tickets

A component counts toward the pinned catalog only when:

1. Its default Nova/Neutral visual states match the pinned source closely.
2. Its GPUI API and behavior are native to GPUI rather than copied from React.
3. Its browser catalog page runs the real GPUI component interactively where the component is interactive.
4. Any deliberate visual or behavior difference is recorded on that component's issue.

This research fixes the source baseline and catalog boundary. It does not schedule all 60 components into `v0.1`; the agreed Button, Checkbox, and Dialog proof slice can test the model first.
