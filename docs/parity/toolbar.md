# Toolbar parity

- Upstream: no shadcn/ui Toolbar component exists in the pinned `4.19.0` base registry.
- gpuicn: `registry/toolbar/toolbar.rs`.
- Difference type: visual adaptation.

Base GPUI provides the toolbar runtime, keyboard actions, buttons, links and separators. The local composition retains its typed child wiring and disabled cascade while using the shared native Input. The port applies the same Neutral Nova controls as Button, Tabs, and Navigation Menu. This is intentionally a visual adaptation rather than a claimed shadcn API or source port.

`toolbar_input_with_label("Find text", cx)` gives the shared toolbar editor a native TextInput role, name and current value. `toolbar_input(cx)` remains available with a generic accessible name. Undo/redo and Unicode word editing use the same editor as standalone Input. Plain arrows leave only at the matching caret edge with no selection; modified word arrows stay in the editor. Roving entry selects the text, and the toolbar keeps one Tab stop.
