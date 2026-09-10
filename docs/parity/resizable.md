# Resizable

Visual reference: [shadcn Resizable](https://github.com/shadcn-ui/ui/blob/1773ecfeeb4a04366978d353e69b5c7ded78dcb2/apps/v4/registry/bases/base/ui/resizable.tsx) and its pinned Nova grip treatment.

`Resizable` is a native GPUI interaction; it keeps caller-owned pane sizes and bounds. It draws a thin themed separator and grip, with visible keyboard focus. Use horizontal, vertical, or nested two-pane groups. Each group has a stable caller-owned ID and accessible label, first-pane preferred size, independent finite pane limits, two children, and a resize-intent callback. Applications own persistence and collapse state. GPUI `Styled` overrides apply to the outer group.

Pointer capture survives movement over pane children. Arrow keys resize by 8 logical pixels, Shift by 32; Home/End reach the allowed limits; Escape ends an active drag. Accessibility Increment, Decrement, and SetValue use the same bounds. Orientation describes the separator itself, not the pane stacking axis.

If the window cannot fit both minima, panes keep their minimum sizes and the group scrolls. If both maxima leave unused space, the group leaves that space empty. These policies are explicit; there is no implicit zero-sized pane, docking system, or layout persistence. Pass retained entities for expensive content. Keep pointer dragging direct; use the shared theme transition helper for discrete collapse/restore changes.

Native frame work, GPU completion, and input latency are different measurements. See the desktop plan and current validation report before treating a batch as qualified.
