# Alert parity

Visual reference: shadcn/ui 4.19.0 `alert.tsx`, Neutral Nova. Implementation:
`crates/ui/registry/alert/alert.rs`, using GPUI Kit 0.9.0.

GPUI Kit supplies nothing here: an alert is layout, not a control, and
nothing in it takes focus. gpuicn draws the card surface with a 1px border,
`radius.lg`, 16px by 12px padding, a 16px Lucide icon in the first column,
a medium-weight single-line title and a muted description, as shadcn does.
Children sit in a wrapping row under the description, for a Retry button.

shadcn has `default` and `destructive`. gpuicn adds `Warning`, `Info` and
`Success`, which color the title and icon with the theme's status tokens,
and gives each variant a default icon (`CircleAlert`, `TriangleAlert`,
`Info`, `CircleCheck`; none for `Default`). `icon` and `no_icon` override it.

Destructive and warning alerts use the Alert role so screen readers announce
them; the others use Status. The title is the accessible name and the
description its description. The application owns when the alert shows.
