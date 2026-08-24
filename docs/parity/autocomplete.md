# Autocomplete parity

- Upstream behavior: Base UI Autocomplete, represented by the pinned Base GPUI `AutocompleteRoot` and reused Combobox parts.
- Visual source: shadcn/ui `4.19.0` Nova Combobox wrapper, because shadcn does not ship a separate Autocomplete wrapper in this pinned registry.
- gpuicn: `registry/autocomplete/autocomplete.rs`.
- Difference types: API, platform, animation, and accessibility.

The port keeps Base GPUI's `List`, `Both`, `Inline`, and `None` modes, typed-value axis, filtering, inline autocomplete behavior, selection handling, and popup lifecycle. It reuses the Nova Combobox skin intentionally because the same editable list and suggestion affordances apply.

No React API is copied. The pinned GPUI surface cannot expose Base UI's `aria-autocomplete`, active-descendant relationship, live regions, browser disabled or read-only attributes, or Nova motion, so the parity claim stops at the actual native behavior and visual treatment.
