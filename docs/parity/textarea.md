# Textarea parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/textarea/textarea.rs`, backed by `registry/input/input.rs` and GPUI Kit
0.6.4.

Textarea uses caller-owned `TextareaState` and supports the same disabled,
read-only, invalid, paste interception, history, clipboard and IME behavior as
Input. It composes with `Field` for a visible label, description and validation
message. Rows set the starting height; content scrolls inside the native editor.

Image or file paste interception is synchronous on native platforms. Browser
clipboard access for those payloads needs an async permission flow and remains
outside this API. Native OS IME and screen-reader workflows require device
checks.
