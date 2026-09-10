# Context Menu parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/context_menu/context_menu.rs`, using GPUI Kit 0.6.1.

MenuState owns the popup, highlighted item and submenu path. Secondary click or Shift+F10 opens the menu; arrows, Home, End, typeahead, Enter and Escape navigate it. Primary clicks remain available to the wrapped content, including Virtual List selection.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
