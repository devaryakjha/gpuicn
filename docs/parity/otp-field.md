# OTP Field parity

- Upstream: shadcn/ui `4.19.0` Nova Input OTP.
- gpuicn: `registry/otp_field/otp_field.rs`.
- Base GPUI owns slot movement, typing, backspace, paste, masking, and completion. The Nova caret blink is not ported because GPUI has no matching declarative animation primitive.
