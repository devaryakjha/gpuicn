# Alert Dialog parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/alert_dialog/alert_dialog.rs`, using GPUI Kit 0.6.1.

A retained Kit DialogHandle owns visibility. Explicit confirm and cancel buttons route actions through the alert host, which can veto confirmation. Tab traversal stays within the popup and closing restores focus. Enter activates the focused button; the host does not convert every Enter press into confirmation.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
