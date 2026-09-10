# Menu parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/menu/menu.rs`, using GPUI Kit 0.6.1.

A retained MenuState owns items, visibility, keyboard focus and submenu navigation. Each item has a caller-owned stable ID. Link activation and routing stay in caller handlers. Items support actions, links, disabled state, check/radio presentation, separators and submenus. MenuState retains checked state and emits MenuEvent for each committed action. Radio items at one menu level select one value. Kit buttons and popup positioning supply the primitives; gpuicn handles the menu keyboard loop and Nova styling.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
