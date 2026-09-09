# Number Field parity

- Upstream visual source: shadcn Nova Input; behavior source: Base GPUI Number Field.
- gpuicn: `registry/number_field/number_field.rs`.
- The port provides numeric parsing, range limits, keyboard stepping, and steppers. Scrubbing is available in Base GPUI but is not exposed by this first visual wrapper.

`aria_label` names the native spin button; a surrounding Field label is the fallback. The input publishes its current text, numeric value, bounds and step on its existing focusable element. The shared native Input provides undo/redo, Unicode word movement/selection/deletion and IME editing. Base GPUI still owns numeric parsing, range/step rules and commit behavior. Read-only and disabled guards cover typing, history and stepping.
