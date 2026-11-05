# Otp Field parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`crates/ui/registry/otp_field/otp_field.rs`, using GPUI Kit 0.6.4.

OtpField takes a retained Kit OtpState configured with a slot count. Kit owns digit input and deletion; gpuicn draws the Nova slots and handles paste: Cmd/Ctrl+V or a dispatched `Paste` action (the Edit menu) fills the cells from the clipboard's digits, ignoring spaces and dashes, and `otp_field::paste_code` does the same from code. The application reads the state and subscribes to its events.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../internal/validation/gpui-kit-migration.md)
for the checks completed on this revision.
