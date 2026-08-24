# Toast parity

- Upstream: shadcn/ui Nova at `ac60ef5c4db4265d71454dd9ecd3f93e255d7211`, `apps/v4/registry/bases/base/ui/toast.tsx` and `apps/v4/registry/styles/style-nova.css`.
- gpuicn: `registry/toast/toast.rs`.
- Difference types: platform, interaction, accessibility, and visual.

Base GPUI owns the typed manager, queue, upsert, timeout, pause/resume, stack limit, Escape, close action, and swipe-to-dismiss behavior. The wrapper applies Nova's bottom-right viewport, rounded toast surface, and compact controls while leaving provider timeout and limit configuration intact.

Nova's CSS stack transforms, transition timing, and icon-by-type helper are not reproduced. The pinned GPUI revision has no live-region API, so new toasts are not announced automatically to screen readers.
