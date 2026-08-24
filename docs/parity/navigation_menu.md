# Navigation Menu parity

- Upstream: shadcn/ui `4.19.0` at `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`, `apps/v4/registry/bases/base/ui/navigation-menu.tsx` and the Navigation Menu section of `apps/v4/registry/styles/style-nova.css`.
- gpuicn: `registry/navigation_menu/navigation_menu.rs`.
- Difference type: platform animation and icon composition.

Base GPUI owns hover delays, patient-click behavior, safe-polygon handling, keyboard navigation, portal positioning, and dismissals. The port keeps the Nova trigger, link, popover, viewport, arrow, and focus treatment. Callers provide icon drawing within `navigation_menu_icon()`; CSS chevron rotation and CSS popup motion remain absent until GPUI exposes equivalent transform and transition APIs.
