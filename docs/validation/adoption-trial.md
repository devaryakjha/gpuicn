# Independent adoption trial

Ask a developer who has not worked in this repository to follow only the public
installation and component documentation. Do not coach them through missing
steps; record each place where they need help.

Use a named release tag when testing public claims. For work that is still in a
pull request, record the PR and commit SHA, install from its checkout or CI
artifact, and label the result pre-release. A PR result does not qualify the
published release.

Record the operating system, Rust version, GPUI dependency versions, commands,
elapsed time, source diff, build or runtime errors, and screenshots or a short
recording. Note every documentation question and any intervention from a gpuicn
maintainer.

## Journeys

1. Create a new GPUI app, install one component from the registry, load the
   documented fonts and theme, and open a native window that uses it.
2. Build a small form with Input, Select and NumberField. Add labels, required
   and error feedback, a disabled control, keyboard traversal, and a light/dark
   theme switch. Save one record, load another, and reset the form.
3. Edit one installed component, add another component, run the documented dry
   run and update flow, and confirm the app edit remains unless overwrite is
   explicitly chosen. Inspect the resulting source diff and run the app again.

The trial passes when the developer completes all three journeys from the docs,
the app builds and behaves as described, and no maintainer intervention hides a
missing step. File concrete failures in issue #58 and attach the evidence; do
not turn a pre-release result into a released-product claim.
