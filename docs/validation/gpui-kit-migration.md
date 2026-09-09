# Direct GPUI Kit migration qualification — 2026-09-10

[PR #55](https://github.com/devaryakjha/gpuicn/pull/55) replaces the previous
compatibility approach. The published gpuicn beta remains unchanged.

## Implementation

- GPUI Kit 0.6.1 supplies the headless controls, editor, motion, notifications
  and popup hosts. All application and registry imports use `gpui_kit`.
- LukeTandjung `base-gpui`, the copied editing implementation, `gpui-compat`
  and the Zed Cargo patch are removed. Fresh installs need no compatibility package.
- Inputs, numeric inputs, OTP and sliders take retained Kit state. Selectors and
  menus take caller-owned entities that compose Kit editors, buttons and positioning.
- Forms use explicit values, validation and submission. Dialogs and drawers use
  retained handles; the mounted host restores focus after dismissal. Group controls
  receive selected values and report changes to the application.
- The workspace app, gallery, all 40 Usage examples, registry metadata and current
  setup/platform notes use the new APIs. Public API compatibility is intentionally
  dropped. Selectors currently select one value; navigation menus use link popups.
- The runtime and platform stay pinned to `gpui-pre` 0.3.4. The direct platform
  dependency enables native shader compilation flags. The released
  [gpui-icons 0.4.0](https://github.com/devaryakjha/gpui-icons/releases/tag/v0.4.0)
  pin supplies all 1,818 Lucide icons and 259 aliases on the same runtime.

Native builds use Rust 1.97.1. WASM uses nightly-2026-08-17, wasm-bindgen 0.2.121
and Trunk 0.21.14. Nova styling remains in the installed source.

Independent review found and resolved menu alignment, position-derived menu IDs,
component-owned URL routing and fixed toolbar dimensions. Menu entries now require
stable caller IDs, link handlers belong to the app, and toolbar sizing uses theme tokens.

## Behavior checks

The migration exposed and fixed three integration issues: context-menu focus
blocked list selection; dialog-wide Enter bindings intercepted focused buttons;
and delayed editor events could restore an old draft after submission. Repeated
chat sends now create a fresh editor state. These paths have runnable regression checks.

The browser check also found that GPUI 0.3.4 binds keyboard events to its IME
textarea but blurs it when focus moves to a button. The preview shell forwards
body/canvas key events to that textarea without refocusing it, duplicating editor
events or opening a mobile keyboard. The JavaScript check covers forwarding and
the existing parent-message validation. This is preview integration code; consumers
embedding this runtime on the web need the same handling until upstream fixes it.

| Check | Result |
| --- | --- |
| Workspace tests | 37 passed |
| Workspace Clippy, all targets, warnings denied | Passed |
| Independent registry install with its own lockfile | Passed; unused public re-export warnings only |
| Optimized native showcase | Built and launched with isolated temporary data |
| Actual native task entry and Enter submission | Created and persisted a task |
| Actual native repeated chat entry | Persisted two separate messages; composer cleared each time |
| Optimized shared WASM preview | Passed |
| Actual WASM input | Typing, undo, redo, Tab traversal and read-only guard passed |
| Actual WASM form | Invalid Enter submission showed an error; valid Enter submission succeeded |
| Actual WASM dialog | Input editing, Tab to Save, Enter activation, closing and focus return passed |
| Web build, TypeScript, lint and highlighting | Passed |
| Preview event boundary check | Passed |

The native run used `/tmp/gpuicn-direct-kit-review/workspace.json`, not the user's
saved workspace. Native pointer automation lost its window mapping; accessibility
actions and keyboard input verified the app, and the saved JSON confirmed results.
OS IME composition, full screen-reader workflows, Windows/Linux runtime and
performance budgets remain unqualified. Historical validation documents describe
their original revisions and are not evidence for this migration.

## Release status

The icon release is already published. This change updates PR #55; it does not
publish gpuicn, merge the PR or deploy the website. Remote CI must validate the
new PR head; the earlier compatibility implementation's green run does not apply.
