# Desktop components plan — 8 September 2026

Arya approved the product expansion and local implementation on 8 September 2026. Build reviewable slices; user testing precedes any publishing. Arya authorized committing and pushing the current work on 8 September 2026. Tags, releases and production deployment remain unauthorized.

## Fixed scope and boundaries

Bonsai is a full native desktop Git client covering basic and advanced Git operations and in-app provider repositories, PRs, issues and releases. UI/UX and performance are release gates. Platform rollout and initial provider scope remain undecided.

Reuse or compose gpuicn components. Ordinary GPUI layout is allowed. Missing reusable UI interactions must be implemented, tested and released in gpuicn before Bonsai adopts them; do not create one-off Bonsai substitutes. Git operations, diff parsing, patch generation, graph layout algorithms, provider APIs, domain merge logic and application state stay in Bonsai.

Migration note (2026-09-09): evaluate GPUI Kit 0.6.1's tree, editor, text/Markdown,
and docking primitives before implementing these planned capabilities. Reuse them
where the required selection, keyboard, accessibility, and source-install contracts
hold. Keep Git operations and domain state in applications. See
[the migration qualification](../validation/gpui-kit-migration.md).

Original planning baseline: gpuicn v0.4.2, commit d138aad99b723c69b7b8c793f8ca615cebede656. Source-registry consumption; workspace crate is publish=false. Rust 1.95.0 / edition 2024; GPUI and gpui_platform 59b2ebf10351b5c0b5cd4403f01ed0460eeec06d; Base GPUI 64b22337b6a790c636aab248e768e4875bb28ba8; gpui-icons b25a5ebae2e1a5f4ddfca1389ab9d21d481d9ec8; shadcn CLI 4.19.0. Recheck pins before later implementation, without silently upgrading them.

## Reuse versus confirmed gaps

- Already available/composable: Toolbar, Tabs, Button, Separator and ordinary GPUI layout for a shell; Menu, ContextMenu and Menubar for in-window actions; Dialog, AlertDialog, Popover and Drawer; inputs, field validation, selection controls, labels, progress and toast. Empty/loading/error content, file/status adornments and branch/tag labels should compose these existing APIs wherever they fit.
- Confirmed absent as gpuicn components: interactive resizable panes, virtualized list with selection behavior, full virtualized tree, diff viewer, commit graph renderer and three-way conflict UI. Separator is not a splitter; ScrollArea does not virtualize; Collapsible is not a full tree.
- Reuse GPUI list/uniform_list where suitable. First inspect their actual pinned behavior, scroll handles and identity semantics. Do not add a second virtualization engine or parallel menus, dialogs, selection widgets, focus system or theme.
- Existing behavior requiring verification/repair: nested NavigationMenu link activation, accessibility relationships, nested modal safety and outside-content inertness. Render tests are not native interaction acceptance.

## Delivery order

T01 contract → T02 native baseline and T03 performance baseline → T04 splitter + T05 list rendering → T06 list interaction → T07 tree + T08 basic diff → T09 diff interaction → T10 advanced diff modes → T11 graph → T12 conflict UI (later).

T02 applies throughout. T11 can proceed independently after T06; it does not technically depend on diff delivery. T13 integrates the foundation and daily-use components; T14 qualifies each release batch after its relevant checks. Conflict UI is a later advanced-Git batch, not a reason to hold a tested foundation release.

## T01 — Reproducible consumption and API contract

Scope: document v0.4.2-based source installation, exact dependency pins, required module declarations, initialization, fonts/icons, stable IDs, controlled state conventions, supported APIs and composition patterns. Use release-checkout registry JSON for reproducibility; explain that live registry URLs are mutable and installation does not update Cargo.toml. Define upgrade/overwrite review steps and release-note requirements. Record current gaps as blockers.
Dependencies: none.
Acceptance: a fresh native consumer installs from the pinned source registry, compiles with its committed lockfile, and launches without undocumented setup. Examples use the same APIs shipped in registry items; source and registry agree. Upgrade instructions preserve caller changes and identify compatibility changes.
Evidence: exact installation commands, pins, installed-source comparison, compiler result and actual native launch, with host/toolchain recorded.
Gate: Bonsai foundation. Platform/provider choices remain open; document rather than decide them here.

## T02 — Native interaction and accessibility baseline, then component gates

Scope: audit existing menus/context menus, toolbar, tabs, dialogs/popovers and inputs, extending the same checks to every new component. Fix known keyboard activation gaps at their shared source. Verify Tab/Shift-Tab, arrow navigation where applicable, Escape, focus return, disabled behavior, accessible names/roles/states/relationships. Verify nested modal behavior and outside-content inertness, including focus and pointer exclusion. Document the OS menu integration boundary: gpuicn Menubar is in-window; Bonsai owns application/OS command integration unless a separately justified reusable gap emerges.
Dependencies: T01; repeat focused acceptance as components arrive.
Acceptance: keyboard-only journeys work in a real native window; opening/closing nested overlays neither strands nor leaks focus; disabled actions cannot fire; screen-reader/OS accessibility inspection reports supported names, roles and states. Unsupported upstream semantics are explicit unresolved blockers, not visual substitutes. Distinguish component actions from Bonsai app shortcuts.
Evidence: native action traces or recordings, accessibility inspection, focused runnable regression checks, exact OS and input method. Report macOS, Windows and Linux separately. No Windows/Linux readiness inferred from macOS or WASM. Mark unresolved upstream dependencies and affected journeys.
Gate: foundation and every daily/advanced Git component used in a release.

## T03 — Performance measurement and budget baseline

Scope: establish a repeatable native harness before readiness claims. Record CPU/GPU, RAM, OS, resolution/scaling, display refresh rate, Rust/profile/commit, data shapes and warm/cold conditions. Measure frame-time distribution and missed frames, input-to-paint latency, mounted/rendered row counts, memory at rest and after repeated refresh/scroll, and resize behavior with expensive children.
Dependencies: T01; datasets feed T04–T13.
Acceptance: reproducible scripts and recorded baseline results exist. Propose budgets against the actual target hardware/datasets: compare frame work against 1000/refresh_hz milliseconds (16.67 ms at 60 Hz, 8.33 ms at 120 Hz), propose a measured p95/p99 and missed-frame allowance with headroom for Bonsai work, and propose memory limits from measured viewport overhead plus caller-owned data. Frame intervals are reference deadlines, not claims of achieved throughput. Agree measured budgets with Arya before calling a component ready; do not invent a universal row/memory/fps target.
Evidence: raw measurements, dataset generator/seed, profiler captures, chosen viewport/refresh rate, proposed and subsequently agreed budgets, pass/fail comparisons. At least 100k lightweight rows for list checks; include variable heights and refresh/selection/resize workloads. Record realistic large-tree/history/diff datasets separately.
Gate: foundation and performance sign-off across all batches.

## T04 — Resizable panes and splitter

Scope: horizontal/vertical layouts, nesting, min/max bounds, controlled sizes for caller persistence, pointer drag, keyboard resizing, visible focus and appropriate accessible separator semantics. Define behavior when the window cannot satisfy all minimum sizes; keep content reachable and focus intact. Support controlled collapse/restore where the intended layout needs it, retaining a usable restore control and prior size. Keep collapse in this API where it is just size state; a full docking/sidebar/window-management system is out of scope and requires its own later task only if a concrete need appears.
Dependencies: T01–T03.
Acceptance: drag and keyboard resize obey bounds; nested panes affect only the intended split; controlled updates round-trip without loops; persisted sizes restore predictably after window changes. Collapsing moves focus safely and restoring works by pointer and keyboard. Small windows do not trap content. Resizing virtualized/expensive children does not reconstruct every data row.
Evidence: native pointer and keyboard checks, resize/collapse regression checks, accessibility inspection, small-window recordings, frame and row-reconstruction profiles under T03 budgets. Integrate expensive list content after T05/T06.
Gate: foundation blocker.

## T05 — Virtualized list rendering and external-data contract

Scope: bounded rendering, stable row identity, fixed-height and variable-height use cases where pinned GPUI APIs support them, scroll-to-item and preservation of viewport anchors across refresh/pagination. Models and loading remain external. Inspect list/uniform_list first; explicitly record any variable-height limitation rather than advertising unsupported behavior.
Dependencies: T01–T03.
Acceptance: a 100k-row data set renders only the visible range and documented overscan, not all rows; variable-height measurement converges without jumps or unreachable rows; reveal/scroll-to-item reaches the correct identity. Inserts/removals/reorders preserve a surviving viewport anchor according to a documented policy. Caller empty/loading/error content composes existing controls.
Evidence: native scroll traces, rendered-row counters, fixed/variable-height cases, 100k-row benchmark with frame/memory evidence and agreed T03 budgets. Keep meaningful runnable checks for identity/anchor behavior.
Gate: foundation blocker; prerequisite for tree, diff and graph.

## T06 — Virtualized list selection and interaction

Scope: controlled single/multi selection, range/toggle selection, keyboard movement, anchor and focus identity, scroll-to-focused row, existing ContextMenu integration and external intent callbacks. Preserve surviving selections on refresh. Define selection for removed IDs and ranges after reorder; callbacks must use current data/state, not stale captured row positions.
Dependencies: T05 and T02.
Acceptance: pointer/keyboard range and toggle selection agree; focus remains visible; context actions target the intended current selection. Refresh/reorder/pagination preserve surviving identities and cannot emit stale selection callbacks. Empty/loading/error states remain usable and model/loading ownership stays external.
Evidence: native keyboard/pointer checks, race-like refresh/reorder callback regression, accessibility inspection and interaction performance at 100k rows under T03 budgets.
Gate: foundation blocker.

## T07 — Virtualized tree for file navigation

Scope: build on the list where sensible; expand/collapse, caller-owned lazy children, stable identities, selectable rows, arrow-key navigation, typeahead, reveal-selected-descendant, existing context menus and composed file/status adornments. Preserve expansion and selection through refresh. Define loading/error/retry composition for lazy branches without embedding data fetching.
Dependencies: T06, T02; T04 for pane integration.
Acceptance: revealing a descendant expands required ancestors and scrolls it into view; keyboard traversal matches visible hierarchy; pending lazy results cannot corrupt a refreshed tree. Surviving IDs retain expansion/selection. Deep nesting and long filenames remain readable/reachable without losing actions or focus.
Evidence: native keyboard/accessibility checks, lazy-load/refresh regression, deep/long-name fixtures and measured large-tree rendering with recorded total/visible node counts under T03 budgets.
Gate: foundation/file-navigation blocker.

## T08 — Basic virtualized unified diff rendering

Scope: caller-supplied parsed diff rows; unified view, line numbers, added/deleted/context rows, hunk headers and syntax-highlight spans or a documented reusable highlighting interface. Compose caller-supplied binary/too-large states. Handle no-final-newline markers, empty/deleted files, Unicode and tabs. Define horizontal scrolling and wrapping policy. No Git execution, parsing or patch generation in gpuicn.
Dependencies: T05, T01–T03.
Acceptance: fixtures preserve exact source text and correct old/new line numbering; markers are not mistaken for file text; Unicode/tabs stay aligned according to documented tab width and wrapping. Large diffs stay virtualized. Highlight input has stable ranges and clear invalid-range handling. Empty/deleted/binary/too-large states render correctly.
Evidence: rendering/text-mapping checks plus actual native viewing and scrolling of large diffs, width/wrapping cases, frame/memory profiling under T03 budgets.
Gate: daily Git blocker. This task alone is not a complete daily-use diff viewer.

## T09 — Diff selection, copy, navigation and intent callbacks

Scope: selectable/copyable text; file/hunk/line action selection distinct from text selection; keyboard navigation/focus; search/reveal-line. Emit intents for Bonsai's stage/unstage/discard/review actions. Distinguish actionable rows from context. Read-only commit/PR modes allow inspection/copy/search while suppressing mutation intents.
Dependencies: T08, T06 where selection behavior can be reused, T02.
Acceptance: selected text copies faithfully with documented line-ending behavior; action selections map to current stable file/hunk/line IDs after refresh. Context rows cannot accidentally become mutation targets. Read-only mode never emits mutation intents. Search and reveal reach virtualized lines and keep keyboard focus visible; stale callbacks cannot target replaced diffs.
Evidence: native pointer/keyboard/copy checks, read-only and refresh regression, search/selection performance on large diffs under T03 budgets, accessibility inspection.
Gate: daily Git blocker.

## T10 — Side-by-side diff and advanced viewing modes

Scope: side-by-side rendering with synchronized scrolling across unequal added/deleted rows, shared line/hunk identity, horizontal scrolling and explicit wrap/no-wrap policy. Reuse splitter and diff/list APIs. Preserve search, text selection, action selection and focus while switching modes or resizing.
Dependencies: T04, T08–T09.
Acceptance: aligned comparison rows/hunks stay synchronized through scrolling, gaps, wrapping, resize and reveal. Switching unified/side-by-side retains the logical location and selection. Both sides remain usable in narrow layouts; document fallback behavior. Large diffs do not instantiate all rows or oscillate between scroll handlers.
Evidence: native mode-switch/scroll/resize recordings, asymmetric diff fixtures, synchronization regression, selection/copy checks and frame/memory measurements under T03 budgets.
Gate: daily Git blocker for the requested dual-mode experience.

## T11 — Commit graph view aligned with virtualized history

Scope: consume caller-supplied stable commit rows, lane/node/edge geometry and labels; never own Git traversal or layout algorithms. Align graph rendering with virtualized history rows. Reuse list selection/keyboard behavior, existing context menus and composed branch/tag decorations. Preserve identity, scroll and selection on refresh/pagination.
Dependencies: T06, T02–T03; use T04 for integrated layout. No dependency on diff algorithms.
Acceptance: graph and text rows remain aligned during scroll/resize/refresh; selection and context actions reference current commit IDs. Merges, crossing/many lanes and long labels remain understandable without obscuring text. Do not rely on color alone: topology, shapes and/or labels carry meaning. Reject or handle malformed caller geometry without crashes, stale targets or runaway work.
Evidence: native large-history benchmark with recorded rows/lanes/edges, pagination/refresh and many-lane fixtures, keyboard/accessibility inspection, frame/memory profiling under T03 budgets.
Gate: history blocker for daily Git and advanced history use.

## T12 — Three-way conflict-resolution UI (later batch)

Scope: base/ours/theirs views plus editable result, conflict navigation, clearly identified unresolved/resolved regions and resolve-intent callbacks. Reuse diff, panes, lists and native text behavior where available. Domain merge logic and persistence remain Bonsai. First assess existing editing APIs; if a necessary reusable editor behavior is missing, split that dependency into a focused gpuicn task before this task proceeds.
Dependencies: T04, T08–T10, T02; any identified editing prerequisite. Not required for the initial foundation batch.
Acceptance: navigation/reveal reaches each conflict, editing/copy/undo behavior follows the documented editor contract, resolve callbacks target stable current conflict IDs, and external refresh cannot silently discard edits or mark conflicts resolved. All four views remain reachable with keyboard and in small windows. The UI does not compute merges or execute Git.
Evidence: native conflict journeys with Unicode/large hunks and refresh while editing, editing/resolve regression, accessibility inspection and large-conflict performance against agreed T03 budgets.
Gate: advanced Git blocker.

## T13 — Small native Bonsai-style integration example

Scope: a small example exercising panes, virtualized file navigation, diff/history content, menus, keyboard navigation, inputs and dialogs using released/intended gpuicn APIs and synthetic caller-owned data. Demonstrate compositions and ownership boundaries; no provider or Git backend scaffolding.
Dependencies: T04, T06–T11 and relevant T02 fixes; add conflict checks when T12 ships.
Acceptance: pointer and keyboard complete representative navigation → selection → diff/history → context action → dialog → focus-return journeys. Resizing, refresh and pagination preserve location/selection. All required content remains reachable at the agreed small-window sizes. Missing APIs or upstream behavior are explicit blockers rather than bypasses in the example.
Evidence: actual native interaction recordings/checklist and runnable focused checks, integrated performance captures under T03 budgets, separate results for each selected target OS. A render snapshot or successful compile alone is insufficient.
Gate: foundation/daily Git integration release gate; extend for advanced Git.

## T14 — Component qualification and release handoff (authorization deferred)

Scope: for each dependency-complete batch, review supported APIs, registry consistency, composition examples, pins, compatibility/upgrade steps, release notes and remaining blockers. Only after Arya separately directs implementation and release should tested components be published; Bonsai adopts them afterward.
Dependencies: relevant tasks, native interaction acceptance and agreed performance budgets. T12 can ship in a later batch.
Acceptance: clean fresh source-registry install, installed-source comparison and native compile/run pass; release pins and examples match artifacts; native/platform/performance evidence is attached with explicit limits. Each Bonsai adoption references a tested component release, not an unreleased local workaround. Do not advertise full platform or accessibility readiness with unresolved gating checks.
Evidence: release checklist, native integration results, benchmark records, source/artifact consistency and upgrade trial. Publishing itself remains unauthorized by this handoff.
Gate: every Bonsai adoption batch.

## Current delivery and review gates

1. **Locally implemented: sidebar + resizable panes + native desktop example.** Implement T01 scope/contract updates and T04 alongside the sidebar task below. Verify real native pointer/keyboard behavior and make the example available for Arya's testing. Keep performance readiness open until measured.
2. List rendering/selection, then tree. Add the 100k-row benchmark and agree hardware-specific budgets before claiming performance readiness.
3. Unified diff and interactions, side-by-side diff, then graph. Validate each through the native example before starting the next acceptance batch.
4. Conflict editor after the editor dependency assessment.
5. User acceptance is a hard publishing gate for every batch. Passing automated checks does not authorize a release.

## T04a — Sidebar composition (first review batch)

Scope: a styled navigation container with header, scrollable content, labelled groups, navigation items and footer; active and disabled item states; optional leading icons/trailing badges or actions. Reuse existing buttons, focus behavior, tooltips and theme sidebar tokens. Caller owns selected destination and collapsed state. Sidebar and splitter are separate: the sidebar owns navigation presentation; the splitter controls adjacent pane sizes.
Dependencies: T01/T02; compose with T04 for resize. No dependency on a tree for ordinary navigation links.
Acceptance: selected destination is clear without color alone; disabled items cannot activate; long names truncate or wrap predictably with a discoverable full label; content scrolls while header/footer remain reachable. Expanded/collapsed navigation works by pointer and keyboard with accessible names. Icon-rail mode retains useful labels/tooltips; full hiding has a reachable restore control outside the hidden subtree. Focus moves safely when a focused subtree is removed. Controlled state persists in the caller. No duplicate navigation router, toolbar, menu or list engine.
Evidence: actual native light/dark screenshots, keyboard and pointer checks, small-window checks, active/disabled and collapse/restore regression checks. Platform evidence starts with macOS only.
Gate: Bonsai foundation; this is a new reusable composition, not a claimed Base GPUI primitive or exact shadcn port.
