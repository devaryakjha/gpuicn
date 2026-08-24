# Field parity

- Upstream: current shadcn/ui [Field](https://ui.shadcn.com/docs/components/field), based on `apps/v4/registry/new-york-v4/ui/field.tsx`.
- gpuicn: `registry/field/field.rs`.
- Difference types: platform, accessibility, and responsive layout.

`field_root`, `field_label`, `field_description`, `field_error`, `field_control`, and `field_item` style Base GPUI's Field parts. `field_group`, `field_content`, `field_title`, and `field_separator` are visual GPUI `Div` helpers because Base GPUI has no corresponding behavior primitive.

shadcn names the root `Field`; gpuicn calls its factory `field_root` to make the Base GPUI part explicit. The `Responsive` orientation uses the vertical layout: GPUI has no container-query API for shadcn's breakpoint switch. `FieldControl` renders only a single-line text input; textarea, select, checkbox, radio, and switch controls retain their own Base GPUI ports.

The pinned GPUI AccessKit surface has no `aria-invalid`, `aria-required`, or `aria-describedby` builders. Base GPUI keeps label-to-control text plumbing, validation, disabled state, and form registration, but does not expose those browser relationship attributes or a live error announcement. Revisit when GPUI adds those APIs.
