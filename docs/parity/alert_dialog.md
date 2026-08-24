# Alert Dialog parity

- Upstream: shadcn/ui Nova at `ac60ef5c4db4265d71454dd9ecd3f93e255d7211`, `apps/v4/registry/bases/base/ui/alert-dialog.tsx` and `apps/v4/registry/styles/style-nova.css`.
- gpuicn: `registry/alert_dialog/alert_dialog.rs`.
- Difference types: platform, interaction, accessibility, and visual.

Alert Dialog reuses Base GPUI's Dialog implementation, including trigger activation, Escape, outside press, Close, focus return, and registered Popup/Close Tab cycling. It does not provide arbitrary-child focus trapping, relationship attributes, outside-content inertness, nested-dialog safety, non-modal parity, or a browser accessibility tree.

Nova's fade/zoom motion and backdrop blur are omitted because GPUI does not expose matching transition or backdrop-filter behavior. Revisit these gaps when GPUI and Base GPUI add those APIs.
