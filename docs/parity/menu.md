# Menu parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`crates/ui/registry/menu/menu.rs`, using GPUI Kit 0.6.4.

A retained MenuState owns items, visibility, keyboard focus and submenu navigation. Each item has a caller-owned stable ID. Link activation and routing stay in caller handlers. Items support actions, links, disabled state, check/radio presentation, separators and submenus. MenuState retains checked state and emits MenuEvent for each committed action. Radio items at one menu level select one value. Item handlers run after the menu closes and its update returns, so a handler may call `set_items` to rebuild the menu. Kit buttons and popup positioning supply the primitives; gpuicn handles the menu keyboard loop and Nova styling.

A panel is as tall as the window allows before it scrolls: a menu below its
trigger uses the larger of the room below and above (the positioner flips it
to the roomier side), and a context menu or submenu uses the window height,
keeping 8px from the edges. Resting the pointer on a submenu row for 200ms
opens the submenu; moving to a sibling row closes it.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../internal/validation/gpui-kit-migration.md)
for the checks completed on this revision.
