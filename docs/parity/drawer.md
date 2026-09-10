# Drawer parity

Reference: [shadcn/ui Drawer](https://ui.shadcn.com/docs/components/base/drawer),
checked 2026-09-10. Nova presentation, implemented with GPUI Kit 0.6.1.

`Drawer` is the root; `drawer_trigger` and `drawer_close` act on its handle.
Pass `DrawerContent` through `.content(...)`. Compose its children with
`drawer_header`, `drawer_title`, `drawer_description`, `drawer_body` and
`drawer_footer`. These correspond to shadcn's root, trigger, close, content,
header, title, description and footer parts.

Keep the root mounted and retain its `DrawerHandle`. Use `.direction(...)` for
any edge and `.show_swipe_handle(true)` for an optional grip. Explicit close
buttons work even with `.dismissible(false)`; that setting disables backdrop,
Escape and swipe dismissal. Application state and save actions stay caller-owned.

Vertical drawers size to their content, with a viewport-relative ceiling that
leaves 24 spacing units visible behind the panel. Side drawers use 75% of a narrow
viewport and 96 spacing units from the 160-unit breakpoint. Header and footer do
not shrink; the body scrolls when space runs out. Standard sizing methods on
`DrawerContent` override the defaults. Vertical headings center, while side
headings align left. Grips follow the axis and exposed edge.

Kit presence retains the panel through its exit. Translation and backdrop fading
use the theme's normal duration and an ease-out curve; reversing an active
transition starts at its current position. OS and theme reduced-motion preferences
remove the travel. Focus stays trapped until the exit completes, then returns to
the trigger. Window-level pointer capture keeps grip drags alive over content;
short drags settle back and drags past a quarter of the panel dismiss it.

The native checks cover initial closed state, interrupted entry/exit, focus
return, Tab trapping, four-edge sizing, a long scroll body with a visible footer,
reduced motion and swipe dismissal. Browser checks exercise all four layouts,
short-drag settling, vertical/horizontal swipes, Escape and backdrop dismissal.

Swipe handling is grip-only, leaving body text selection and scrolling intact.
Snap points, nested stacking, non-modal drawers and velocity-based flings are not
part of this API. Mobile OS gesture behavior and screen-reader parity remain
unqualified. See [migration qualification](../validation/gpui-kit-migration.md).
