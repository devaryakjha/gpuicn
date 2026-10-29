# Kbd parity

Visual reference: shadcn/ui 4.19.0 `kbd`, Neutral Nova. Implementation:
`crates/ui/registry/kbd/kbd.rs`, using GPUI Kit 0.8.1.

gpuicn draws the Nova key cap: a 20px muted surface with small radius and medium
12px text. `kbd_group` matches shadcn's `KbdGroup` spacing.

Unlike the web component, `Kbd::shortcut`, `Kbd::keystroke` and `Kbd::for_action`
take GPUI keystrokes and format them for the operating system GPUI reports,
including in the browser: modifier glyphs in menu order such as `⇧⌘P` on macOS,
and `Ctrl+Shift+P` on Windows and Linux. `secondary-` resolves to Command on
macOS and Control elsewhere, as in key bindings. `Kbd::for_action` reads the window's keymap, so a hint follows
the user's rebinding. Use `Kbd::new` for literal text.

Kbd is presentational. It takes no focus and has no role; put the action's name
in nearby text or in the control the shortcut belongs to.
