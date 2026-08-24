# Form parity

- Upstream: current shadcn/ui [Forms](https://ui.shadcn.com/docs/forms) and Field composition guidance.
- gpuicn: `registry/form/form.rs`.
- Difference types: platform and accessibility.

The `form` factory styles Base GPUI's `Form`, which registers fields, validates them on submit, focuses the first invalid control, and accepts an `on_form_submit` callback. `FormSubmitAction` remains the explicit native trigger; a styled gpuicn `Button` dispatches it from its click handler.

shadcn delegates state and submission to a React form library. Base GPUI owns those mechanics natively, so there is no React-style controller API or browser `<form>` submit event. The form itself has `Role::Form` and accepts a literal `aria_label`, while invalid and error relationships remain limited by the pinned GPUI AccessKit APIs described in the Field parity note.
