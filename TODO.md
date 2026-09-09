# gpuicn — active product pass

Updated 8 September 2026. This is the running checklist for Arya's current local pass. New requests go here before implementation. Checked means verified; started work stays unchecked. No commit, push, release, or deployment until Arya tests and authorizes it.

## Product contract

- [x] Expand beyond small controls to reusable desktop components; record ownership boundaries in `CONTEXT.md`.
- [x] Keep shadcn Nova at the pinned revision as the visual source of truth. Remove the bespoke sidebar active border.
- [x] Keep every API general-purpose. Bonsai is one consumer; no routing, Git model, provider, or persistence logic in gpuicn.
- [x] Ready-to-use defaults, small caller changes for personalization, editable source, no runtime service or installer lock-in.

## Theme and customization

- [x] Wire spacing/density across the existing component catalog, including coupled heights, padding, icons, and offsets.
- [x] Keep typography independent of density; derive the radius scale from one base value.
- [x] Remove hardcoded component colors/radii and expose overlay color through the theme.
- [x] Preserve custom non-color tokens when changing color mode; applying a theme must repaint components.
- [x] Support ordinary per-instance styling on wrapped controls without losing default interaction/accessibility behavior.
- [x] Document customization of Base GPUI composition helpers and any upstream limits honestly.
- [ ] Native review of default light/dark and a visibly personalized theme; check dense layouts, disabled, hover, focus, selected, and open states.

## Motion

- [x] Fix WASM animation clock panic with GPUI's existing `web-time` dependency; verify Accordion opening/closing, Switch toggling, and Navigation Menu opening in the browser.

- [x] Shared theme durations/easing and one interruptible native transition pattern.
- [x] Smooth sidebar collapse/expand; rapid reversals continue from the current width, while pointer dragging stays direct.
- [x] Reuse motion for appropriate state changes such as the switch thumb and disclosure chevrons.
- [x] Honor both the app and theme reduced-motion settings. Do not claim automatic OS preference wiring unless verified.
- [x] Audit open/close motion against Base GPUI presence support; preserve focus/dismissal behavior and document blocked exit animations.
- [x] Measure animated layouts with expensive retained children; no idle animation frames or per-row decorative animation.

## First native component batch — ready for local acceptance

- [x] Sidebar: header/footer, scrolling, groups, active/disabled items, truncation, tooltips, icon rail, stable focus, caller-owned state.
- [x] Resizable: horizontal/vertical/nested groups, controlled sizes, limits, small-window policy, pointer/keyboard/accessibility actions.
- [x] Native desktop review app with persistent input, dialog, theme controls, and a retained 100k-row list.
- [x] Regression check for resize limits, drag/release, and keyboard resizing.
- [x] Diagnose missing native text and enable the pinned platform's `font-kit` feature in consumer setup.
- [x] Complete the first-batch macOS interaction/accessibility smoke check, generated registry items, catalog previews, usage examples, and fresh installation. Full screen-reader coverage remains open.

## Native distribution without Node — local preview verified

- [x] Small Rust CLI with TOML config; install editable Rust source into a caller-selected directory.
- [x] List/add components from a pinned remote registry or local checkout; install required shared modules together.
- [x] Preserve local edits by default, make overwrite explicit, validate registry paths, and avoid partial writes after validation/download failures.
- [x] Keep module declarations correct without replacing caller code; show required Cargo dependencies and setup.
- [x] No Node, npm, `components.json`, or `tsconfig.json` required for the Rust installation path.
- [x] Keep the existing shadcn path compatible where practical; document that the React documentation site has separate authoring dependencies.
- [x] Reproducible native registry generation/packaging if needed, CLI checks, and a fresh Rust-only consumer install/compile/run.
- [x] Install/upgrade docs, local CLI usage, and clear release/publication boundary.

## Performance and validation

- [x] Initial native debug baseline: 100k fixed-height rows, maximum 14 rendered rows, p95 render-to-paint CPU 4.61ms on this Mac. This is not GPU completion or input latency.
- [x] Repeat after theme/motion changes; record hardware/profile/data, raw measurements, frame distributions, and memory.
- [x] Verify native mouse/keyboard interactions separately from headless tests and WASM checks.
- [x] Run focused regression, workspace checks, source/registry consistency, fresh consumer check, and docs-site build.
- [x] Review the local change set; record platform/upstream limits and user-test steps in [validation](docs/validation/desktop-pass-2026-09-08.md).
- [ ] Arya's local acceptance before any publishing.

## Sidebar depth pass — local implementation complete

Arya requested first-class shadcn-like Sidebar support and several advanced, working site examples. Keep this pass restricted to Sidebar and the preview plumbing it needs. Preserve the shared visual identity and theme tokens. No publishing without local acceptance.

- [x] Audit the pinned shadcn Sidebar source, current documentation, and distinct block patterns; record a capability matrix.
- [x] Support left/right placement, standard/floating/inset surfaces, icon/off-canvas/fixed collapse modes, shared controlled state, trigger/rail, and adjustable token-based widths.
- [x] Add composable groups, actions, nested navigation, menu sizes/variants, badges, separators, search/input, and skeleton/loading states.
- [x] Handle narrow layouts, keyboard/focus behavior, tooltips, state retention, and reduced motion; document any actual platform differences.
- [x] Build several substantial interactive sidebar examples with their own source on the site; make capabilities discoverable rather than packing everything into one demo.
- [x] Verify native/browser interactions and narrow layouts; render-check custom tokens and open states. Full arbitrary-font and long-label visual qualification remains application work.
- [x] Check source distribution, regression tests, performance, and the full Sidebar change set; leave a concrete local review.

Evidence and platform limits: `docs/validation/sidebar.md`. Browser iframe Shift-Tab containment and full platform accessibility qualification remain open.

## Remaining qualification

- [ ] Full native catalog state review and VoiceOver walkthrough; AX smoke checks do not prove complete screen-reader usability.
- [ ] Windows/Linux native checks and the Rust 1.95 CI run (local toolchain is 1.97.1).
- [ ] Upstream presence support before complete dialog/toast/disclosure exit motion.
- [ ] OS reduced-motion preference bridge in the application/platform.

## Later desktop batches

The detailed dependencies and acceptance gates remain in [desktop-components.md](docs/plans/desktop-components.md):

- [ ] Virtualized list rendering and selection (T05/T06), including stable IDs and refresh behavior.
- [ ] Virtualized tree (T07).
- [ ] Unified diff and interaction, then advanced/side-by-side modes (T08–T10).
- [ ] Commit graph renderer (T11); the application owns graph layout/domain data.
- [ ] Conflict UI after the editor dependency assessment (T12).
- [ ] Extend native integration and qualify each dependency-complete batch (T13/T14).

No check here claims universal shadcn parity, Windows/Linux readiness, or performance readiness without the relevant evidence.

## Virtual list foundation — active

Current Sidebar/Resizable/theme/CLI work was committed and pushed as `a64266f`; releases and deployments remain deferred. The next authorized slice covers T05/T06, before the tree and diff components.

- [x] Reuse pinned GPUI virtualization and Base GPUI scrollbar behavior.
- [x] Add stable identities, atomic metadata replacement, viewport anchors and surviving selection/focus.
- [x] Add range/toggle selection, keyboard navigation, disabled guards and activation callbacks.
- [x] Verify 100k fixed-height rows, native interactions, and refresh/reorder behavior in a real preview.
- [ ] Record native frame/memory measurements with their limits; performance budgets remain unapproved.
- [x] Finish source installation, catalog examples, docs and static/regression checks.

- [ ] T05 variable-height support: qualify the pinned engine's resize/height-estimate behavior before exposing this API.

## Native showcase — current priority

Further component work is paused while the native gallery is reviewed. macOS first; Windows and Linux follow.

- [x] One native app with all catalog components, search, themes, reset and Sidebar examples.
- [x] Package the Apple-silicon app and serve a verified ZIP/checksum from the local website.
- [x] Fix shared scrollbar corner geometry and long-sidebar clipping; inspect native light/dark results.
- [x] Check all gallery entries, search/navigation, native list reveal, Rust/WASM/web builds and lint.
- [ ] User review of the native app.
- [ ] Developer ID signing/notarization and public deployment; current package is a local developer preview.

## Workspace example — replaces the gallery as the default app

- [x] Projects, tasks, editable notes, completion/priority, search, deletion/undo and local project messages.
- [x] Validated local persistence; protect unsaved task edits on navigation and quit.
- [x] Correct panels icon; package and download page describe the working app.
- [x] Rapid input and saved-content regression checks; native task/edit workflow review.
- [ ] User review, then signing/public deployment. Windows/Linux follow macOS.

### Workspace design review

- Reworked the native workspace with monochrome light/dark themes, shared navigation, readable task details, and local chat.
- Added shared Input autofocus and Enter submission; covered task/message persistence through keyboard submission.
- Fixed missing app icon colors and card/scroll-surface seams in composition.
- Keep visual acceptance with the user open. Shared wrappers now supply native input accessibility metadata.

## Shared-control audit — 2026-09-09

- [x] Shared SidebarLayout, focus-preserving chat/title editing, failed-save retry and quit guard.
- [x] Installed theme initialization; native Input/Field editing, form Enter and inherited labels.
- [x] Named gallery controls, working demo menu feedback and reachable native gallery.
- [x] Native/browser interaction replay, final modal focus verification, and refreshed website screenshots; see the audit record for coverage.
- [x] NumberField and Toolbar share native Input undo/redo and word editing; preserve numeric rules, roving focus and disabled guards.

See `docs/validation/showcase.md` for observed behavior and verification limits.
