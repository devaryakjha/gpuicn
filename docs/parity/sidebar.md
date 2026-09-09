# Sidebar

Reference: [pinned shadcn Nova source](https://github.com/shadcn-ui/ui/blob/1773ecfeeb4a04366978d353e69b5c7ded78dcb2/apps/v4/registry/bases/base/ui/sidebar.tsx), its `style-nova.css`, and the [workspace](https://ui.shadcn.com/view/new-york-v4/sidebar-07) and [mail](https://ui.shadcn.com/view/new-york-v4/sidebar-09) blocks.

| Capability | gpuicn |
| --- | --- |
| Shared open state | Caller-owned `SidebarState`; independent desktop and mobile state |
| Placement and surface | Left/right; Sidebar, Floating, Inset |
| Desktop collapse | Offcanvas, Icon, None; optional pointer rail and visible trigger |
| Narrow viewport | Modal sheet, backdrop, Escape, Tab containment, opener focus restoration |
| Shortcut | Cmd/Ctrl+B inside the layout |
| Structure | Fixed header/footer, scrolling content, groups, nested menus, separators |
| Navigation rows | Three sizes, selected/disabled/outline, icons, badges, secondary actions, tooltips |
| Composition | Existing Input and Menu; workspace/account dropdowns; stable skeleton rows |
| Styling | Semantic sidebar colors, spacing, fonts, radius and motion tokens; `Styled` overrides |
| Persistence/routing | Application responsibility; no browser cookie or provider API |

Start with `SidebarLayout::new(id, state, mobile, navigation, content, on_change)`. Store the returned state in your view and notify it. Use the same `mobile` decision for the layout, trigger, and `state.icon_collapsed(mobile, mode)`. `sidebar_is_mobile(window)` defaults to 768px; embedded panes can choose their own breakpoint. The site uses 540px to keep its 600px preview useful.

Navigation uses Geist at 14px with a 20px line height, secondary text at 12px/16px, themed 32px rows and 8px insets. Text sizes and line heights scale together through `text_scale`. Width defaults are 256px expanded, 48px for icons and 288px for the sheet; setters accept application widths. Hidden navigation leaves the focus tree. Keep a visible trigger in content. Omit group labels and nested rows when rendering icons, while retaining full item labels and stable IDs. The shared transition helper honors reduced motion and reverses from the current width.

The six site examples cover workspace navigation, searchable documentation, searchable mail with unread filtering, a floating right sidebar, a mobile sheet, and loading content. Selection and filters are local demonstration data; they do not call a service.

Sidebar rows use Kit buttons; selected and expanded state also appear in the accessible name. Native tooltips do not add tab stops. macOS checks and browser checks do not establish Windows/Linux or full screen-reader coverage. See `docs/validation/sidebar.md` for evidence and remaining limits.

In the embedded web preview, Shift-Tab can cross the iframe boundary even while GPUI navigation is modal. Native focus traversal is separately verified; browser modal containment remains a platform limitation.
