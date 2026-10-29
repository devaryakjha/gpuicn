# Badge parity

Visual reference: shadcn/ui 4.19.0 `badge`, Neutral Nova. Implementation:
`crates/ui/registry/badge/badge.rs`, using GPUI Kit 0.8.1.

gpuicn ports the 20px pill with medium 12px text and the Default, Secondary,
Destructive, Outline and Ghost treatments. Destructive uses Nova's tinted
destructive fill rather than a solid one. `Badge::count` formats unread and
mention counts and caps them as `99+`; `Badge::icon` adds a leading Lucide icon.

shadcn's `asChild` link badge has no equivalent: a badge is noninteractive. Wrap
it in a Button when it must act. The label is read as plain text, so give counts
context in surrounding text, such as a sidebar item's accessible name.
