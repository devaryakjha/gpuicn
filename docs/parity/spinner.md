# Spinner parity

Visual reference: shadcn/ui 4.19.0 `spinner`, Neutral Nova. Implementation:
`crates/ui/registry/spinner/spinner.rs`, using GPUI Kit 0.8.1.

The spinner is Lucide `loader-circle` at 16px, rotating once a second like
Tailwind's `animate-spin`. It is announced with the status role and the label
"Loading" unless `Spinner::label` replaces it. Size it with `size`; GPUI SVGs do
not inherit the text color, so the icon uses the muted foreground unless
`Spinner::color` sets one, for example the primary foreground inside a Button.

When `UiTheme.motion.reduced` or the platform's reduced-motion setting is on, the
icon does not rotate and no animation frames are requested.
