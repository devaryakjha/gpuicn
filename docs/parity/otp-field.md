# Otp Field parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/otp_field/otp_field.rs`, using GPUI Kit 0.6.1.

OtpField takes a retained Kit OtpState configured with a slot count. Kit owns digit input, selection, deletion and paste distribution; gpuicn draws the Nova slots. The application reads the state and subscribes to its events.

The Rust source and Usage example define the supported API. Browser DOM/CSS behavior,
exact exit animations and cross-platform screen-reader parity are not implied by
the native component. See [migration qualification](../validation/gpui-kit-migration.md)
for the checks completed on this revision.
