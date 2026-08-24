# Input parity

- Upstream: shadcn/ui `4.19.0` Nova Input.
- gpuicn: `registry/input/input.rs`.
- Base GPUI owns editing, selection, focus, disabled, read-only, and value-change behavior. Its pinned accessibility label builder is currently a no-op; callers should keep a visible label.
