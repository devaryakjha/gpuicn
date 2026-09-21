# Source adoption readiness — 13 September 2026

[PR #59](https://github.com/devaryakjha/gpuicn/pull/59) contains the code changes.
[Issue #58](https://github.com/devaryakjha/gpuicn/issues/58) tracks qualification.
These fixes ship in v0.5.0-beta.4. The evidence below records the original
September 13 checks; see the beta.4 release notes for upgrade validation.

## Evidence

| Path | Observed result | Limit |
| --- | --- | --- |
| CI at `261de2a` | 47 workspace tests, Clippy, 19 installed-source tests, both registry installers, WASM build and web checks passed in [run 34769477162](https://github.com/devaryakjha/gpuicn/actions/runs/34769477162). | The consumer and all copied Usage snippets also passed at `8199229` in [run 34771604318](https://github.com/devaryakjha/gpuicn/actions/runs/34771604318). See PR checks for the final selector frame fix. |
| Installation | The rendered first command has real newlines and ran in a temporary app. The tagged public CLI installed and completed init/add-button. Font, license and starter URLs returned successfully. | Clipboard API readback was unavailable. Public beta.3 retains the earlier component defects. |
| Source updates | Starting with beta.3 Button source, adding Select and updating without overwrite preserved an app edit. Dry run wrote nothing. Explicit overwrite replaced the source; the saved customization was reapplied. | File handling against the local destination registry, not app behavior after a new public release. |
| Mixed-form WASM | The CI-built form displayed error borders and messages on text, choice and number controls. Entering valid values, selecting Pear by keyboard and incrementing quantity allowed submission and cleared errors. The closed selector showed a keyboard-focus border. Light/dark switching retained the values. | Verified against the CI artifact at `261de2a`; this does not establish native screen-reader parity. |
| Native consumer | A standalone copied-source binary built with one compiler job in 5.7 seconds. Native input and region selection retained edited values across navigation. Filtering Billing, editing its name, saving by keyboard and reopening showed the saved name. Escape dismissed the dialog; restored focus reopened it with Return. Sidebar selection changed projects and Right changed the split width from 280 to 288. | Session-owned data. The final toolbar grouping, accessible receipt and stepper-name follow-ups were not visually rechecked. |
| Usage installs | All 40 snippets use flat installed module imports. Generated commands include Usage-only dependencies. CI compiles the same snippets against copied sources, not library re-exports. | Generator checks import/module closure; Rust compilation remains the syntax/type check. |

The consumer is at `fixtures/registry-install/src/bin/readiness.rs`; its README
contains setup and run commands. The original installation starter remains the
default binary. Its controls expose custom palettes, independent compact spacing
and larger text, long content and disabled fields for the remaining review.

The final browser pass exposed a saved-text font panic that native CI did not
catch. Searchable editors now wait for their first rendered frame before syncing
saved text. In the CI WASM artifact at `79c1b86`, Combobox showed Pear and
Autocomplete showed the unlisted dragonfruit value on initial load. Clearing the
Combobox and pressing Enter selected Apple; Autocomplete accepted lychee with
Enter. The next commit only updates the test frame tick and a source comment.
The focused regression tests retain their value, record-change and reset checks.

## Remaining acceptance

- Complete native pointer/focus appearance and the full palette, density,
  typography, long-content and narrow-window matrix. The native control tool
  stalled about 17 minutes during launch despite a 30-second timeout, later
  returned stale screenshots, and could not target pointer coordinates. Keyboard
  and accessibility-tree observations above are narrower evidence.
- Have the independent tester complete [the trial](adoption-trial.md), including
  native VoiceOver, OS IME and runtime performance observations. The user will
  arrange the tester; no outside acceptance result is claimed.
- Windows and Linux native behavior are not qualified by the macOS or WASM checks.

Local Rust compilation uses one job at a time after the laptop overheated during
parallel builds. Broad verification runs in GitHub CI; do not repeat unchanged
local suites.
