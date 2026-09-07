# Dialog parity

- Upstream: shadcn/ui `4.19.0` at `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`, `apps/v4/registry/bases/base/ui/dialog.tsx` and the Dialog section of `apps/v4/registry/styles/style-nova.css`.
- gpuicn: `registry/dialog/dialog.rs`.
- Difference types: platform, interaction, accessibility, and visual.

Dialog keeps Base GPUI's trigger, Escape, outside press, close, and focus return. Modal Tab and Shift-Tab traversal includes app-owned inputs and buttons through GPUI's tab stops. It does not claim relationship attributes, outside-content inertness, nested-dialog safety, non-modal parity, or a browser accessibility-tree bridge. Base GPUI scopes the registered parts but the pinned Backdrop still uses a fixed element ID, which is another reason nested dialogs remain out of scope.

Nova's 100ms fade/zoom motion and backdrop blur are omitted because the pinned GPUI parts do not expose equivalent transition or backdrop-filter behavior. The visible result keeps the pinned layout, color, radius, border, typography, and focus states without pretending the missing effects exist.

Re-review these differences when GPUI exposes relationship/inert APIs, Base GPUI expands its focus registry, nested Backdrops become scoped, or GPUI gains matching transition and backdrop-filter support. The regression check covers forward and reverse wrapping; the WASM preview also verifies input editing, Save, and focus return.
