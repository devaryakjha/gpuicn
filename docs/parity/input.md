# Input parity

- Upstream: shadcn/ui `4.19.0` Nova Input.
- gpuicn: `registry/input/input.rs`.
- Native editing and text layout adapt the pinned Base GPUI implementation, with its MIT notice retained in the installed source.
- The shared Input owns undo/redo, Unicode word movement and deletion, line selection, double-click word selection, IME composition, disabled/read-only guards, and theme-aware selection and placeholders.
- `aria_label` reaches the native accessibility tree with the current value and placeholder. Field controls inherit their registered visible label; an explicit `aria_label` overrides it.
- `on_change` and `on_submit` accept `cx.listener(...)`; the value-only callbacks remain available. Call `theme::init(cx)` at startup for keyboard traversal and Base GPUI bindings.
- Undo retains at most 100 snapshots or 8 MiB. Input is a single-line control; paste and native line breaks normalize to spaces.
- Field controls use this same Input and still register value, focus, required and disabled state with Base GPUI forms.
