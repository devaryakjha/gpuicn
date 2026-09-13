# Source adoption readiness — 13 September 2026

Track remaining work in [issue #58](https://github.com/devaryakjha/gpuicn/issues/58).
These results describe local work based on v0.5.0-beta.3. They do not establish
that the public release includes the fixes or that the full adoption goal is met.

## Completed checks

| Path | Evidence | Limit |
| --- | --- | --- |
| Installed sources | A fresh consumer compiled and passed 19 copied-source tests after fixing sibling theme initialization. CI now runs these tests. | This consumer check preceded the final selector cleanup. |
| Selector values | Two regression checks cover initial listed Combobox values, unlisted Autocomplete text, later values and reset. | Runtime tests do not prove native or WASM appearance. |
| Integrated workspace | 46 tests passed; Clippy with warnings denied passed. | Results precede the mixed-field work. Reuse them for unchanged code. |
| Installation guide | The rendered first command contains real newlines and ran successfully in a temporary directory. The tagged public CLI installed and completed init/add-button. Font, license and starter URLs returned successfully. | Clipboard API readback was unavailable. Public beta.3 still contains the earlier component defects. |
| Source updates | Using the published CLI, a temporary beta.3 Button installation retained an app edit when adding Select and when updating without overwrite. Dry run changed no files. Explicit overwrite replaced the source; the saved app edit could then be reapplied. | The destination was the local registry, not a new public release. This verifies file handling, not application behavior after a version change. |
| Native probe | An already-built consumer displayed the saved Combobox label and unlisted Autocomplete text. Next-record and Reset actions changed the editor values together. | This binary precedes the final selector cleanup. Native automation could invoke accessible actions but did not reliably target the window for keyboard/mouse input; focus remains unverified. |
| Mixed fields | Field now applies a common required name, disabled state and validation state to text, choice and number controls. The form preview validates all three. A serial incremental compile check passed for the library, tests, Usage examples and showcase in 3.5 seconds. | The new field regression compiled but was not run. The changed native/WASM interface still needs review. |
| Mixed-field installation | Installing Select and NumberField from the regenerated registry copied byte-identical source and only their Field, Input and Theme dependencies. | This file-level check did not compile a fresh consumer again. |

## Next evidence

- Verify initial values, reset, selection and visible keyboard focus in native
  and WASM interfaces built from the final source.
- Exercise the new mixed form's labels, required/error feedback and disabled
  behavior in the real interface, then cover saving and switching records.
- Complete the consumer journeys and theme/content matrix in issue #58.
- Record native keyboard, VoiceOver, IME and performance evidence separately
  from Windows/Linux support and a docs-only trial with another developer.

Further local Rust compilation is limited to one job at a time after the
laptop overheated during parallel builds. Do not rerun broad suites for changes
that the completed checks already cover.
