# Fieldset parity

- Upstream: current shadcn/ui [Field](https://ui.shadcn.com/docs/components/field) `FieldSet` and `FieldLegend`.
- gpuicn: `registry/fieldset/fieldset.rs`.
- Difference types: platform and accessibility.

shadcn's `FieldSet` and `FieldLegend` map to `fieldset_root` and `fieldset_legend`. The Base GPUI root exposes `Role::Group` and a literal `aria_label`; use the same text for `FieldsetRoot::aria_label(...)` and render the visible legend with `Text::new_inaccessible(...)` to avoid a duplicate announcement.

This is not a browser `<fieldset>`/`<legend>` pair, so browser-specific form semantics do not apply. Base GPUI still cascades disabled state to registered descendant controls. Revisit the literal-label workaround when GPUI supports `aria-labelledby` references.
