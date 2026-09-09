# Showcase and shared-component audit — 2026-09-09

Scope: working tree on `codex/v0.3-component-fidelity`, including the native
Workspace app, component gallery, shared registry source and fresh installation.
This record describes the local beta checks. Release signing and deployment
are tracked separately in the GitHub release and deployment run.

## Corrections

| Observed issue | Shared or application correction | Evidence |
| --- | --- | --- |
| Workspace used its own sidebar shell and viewport styling | Shared SidebarLayout and default transparent ScrollArea viewport | GPUI collapse/restore and Cmd-B checks, including from focused input |
| Title pencil did not focus the editor | Auto-focus on opening the shared Input | Native title typing, undo, redo and Enter save |
| Chat lost focus after sending | Stable controlled Input; contextual submit callback | Two consecutive native messages without refocusing; persisted-content test |
| Failed detail save discarded the draft and could appear saved | Roll back data on failure; retain drafts; track unsaved writes; retry and quit guard | Real filesystem failure/retry regression |
| Undo deletion could change project before navigation confirmation | Change project only after navigation is accepted | Controller-path review; guarded navigation regression |
| Installed source lacked shared keyboard initialization | Bind traversal and Base GPUI initialization in theme::init | Fresh source install and input/dialog keyboard tests |
| Text input lacked undo/word shortcuts and native text semantics | Shared native editor and text layout with retained upstream MIT notice | Unicode/IME, reconversion, undo/redo, disabled/read-only tests; native title editing |
| Form Enter could miss the last keystroke or skip required validation | Register changed value before callback; use enclosing Form submit | Enter immediately after typing with no intervening draw; empty required rejection |
| Controlled dialogs could leave focus behind the popup | Shared root transition handling captures/restores focus and retains modal traversal | Controlled open, Tab/Shift-Tab, Escape and direct-close checks |
| Cmd-Q from a dirty focused input could bypass the confirmation | Defer the app-menu quit action until after key dispatch | Native Cmd-Q opens confirmation; Escape/Keep editing preserve the draft |
| Tab from a dialog button could select the global traversal binding | Capture both dialog and shared fallback actions inside the modal | Native four-Tab wrap, reverse traversal, Enter activation and restored typing focus |
| Field labels did not name controls | Inherit FieldContext label unless explicitly overridden | Rendered accessibility-node assertion |
| Number/toolbar editors were absent from the native accessibility tree | Shared Input publishes the name, value and role; NumberField adds numeric metadata | Metadata regression; final native names/values, stepping and guarded states |
| NumberField and Toolbar lacked undo/redo and word editing | Both now use shared Input; Base numeric and toolbar runtimes retain their own behavior | Keyboard regressions cover history, Unicode word selection, numeric steps/bounds, disabled/read-only guards, grouped roving focus and one Tab stop |
| Text buttons/triggers were unnamed to accessibility clients | Button.label and explicit trigger labels in previews and copied examples | Native accessibility tree exposes each button and trigger name |
| Menu demo actions did nothing and showed unbound shortcuts | Visible action feedback and no fake shortcut badges | Native Profile action closes menu and shows Last action: Profile |
| Gallery required a launch flag | Sidebar and app-menu entry opens a gallery window | Native gallery opened from Workspace |

## Completed native journeys

An isolated audit workspace used `GPUICN_WORKSPACE_DATA`; the user's saved
workspace was not used for the final title/chat checks.

- Pencil → type title → Cmd-Z → Cmd-Shift-Z → Enter saved the expected title.
- Two messages sent consecutively without another click in the composer.
- Cmd-B hid/restored project navigation while the editor held focus.
- Light and dark Workspace screens rendered; orange remained confined to the logo.
- Component gallery opened as a separate native window from the sidebar.

- Empty required Form rejects Enter; typing `ada@example.com` then Enter submits.
- Number Field increments 4 → 5; read-only 6 and disabled 8 remain unchanged.
- Field labels and toolbar editor names/values appear in the native accessibility tree.
- Dropdown Profile closes the menu and shows action feedback.
- Cmd-Q from a dirty input opens the save guard. Four Tabs wrap to Keep editing;
  three Shift-Tabs from the popup also reach it. Enter closes the guard and
  typing resumes in the draft. The original sample title was restored afterward.

Saved, inspected evidence is under `target/validation/showcase-audit/`:
`10-native-form-submit.jpg`, `12-native-menu-action.jpg`,
`13-native-quit-guard.jpg`, `14-workspace-light.jpg`, `15-workspace-dark.jpg`,
`16-workspace-chat.jpg`, and `17-final-native-gallery.jpg`.
The refreshed website uses the three Workspace captures. Blank or stale
captures are not used as evidence.

## Completed browser journeys

- Shared WASM Input accepts real key events; Option-Backspace deletes the last
  word and Cmd-Z restores `alpha beta`.
- Empty required Form rejects Enter; the completed email submits successfully.
- Saved evidence: `09-browser-input-undo.jpg` and `11-browser-form-submit.jpg`.
- All 40 previews also have automated light/dark rendering checks. This does
  not claim every interaction in all 40 components received a manual replay.

## Final shared-editor replay

After unlocking the Mac, the isolated review app was restarted with the final
binary. NumberField and Toolbar passed the native checks below; screenshots
were captured and inspected after those checks.

- NumberField: replace 4 with 7, undo to 4, redo to 7, Option-Backspace clears,
  undo restores 7, and Up steps to 8. Read-only 6 and disabled 8 remain unchanged
  after attempted edits and stepper activation. Native names and SpinButton
  roles remain present.
- Toolbar: Option-Backspace removes the last word of `alpha beta`; undo restores
  it and redo removes it again. Word selection replaces only `beta` with `gamma`.
  A plain Left at the start moves focus to Underline; Right enters the editor,
  selects its text, and typing replaces it with `project notes`. The TextInput
  name/value remain present. Light and dark appearances were inspected.
- Browser WASM: Toolbar word deletion and undo pass through real key events.
  NumberField passes undo/redo, word deletion, restored stepping to 8, and
  read-only/disabled guards against the final rebuilt WASM preview.

Evidence in `target/validation/showcase-audit/`:
`18-native-number-history.jpg`, `19-native-toolbar-history.jpg`,
`20-native-toolbar-dark.jpg`, `21-browser-toolbar-history.jpg`, and
`22-browser-number-history.jpg`.

## Final automated and package checks

- `cargo test --workspace --locked --features gpui_platform/runtime_shaders`: **44 passed** (29 component, 2 integration, 2 CLI, 11 showcase checks).
- Fresh native CLI install: **29 component tests passed** with the fixture's locked dependencies. Installed Rust files exactly match registry source, including nested Input helpers.
- Stock `shadcn@4.19.0 add --overwrite` and native CLI installation both compare all **50** installed Rust files byte for byte with the final registry, including the shared editor and toolbar composition.
- Workspace Clippy with all targets and `-D warnings`, rustfmt, and `git diff --check`: passed.
- Native release, WASM release, web production build, ESLint, highlighting and preview-message checks: passed.
- Packaged arm64 ZIP: **7,048,813 bytes**, ad-hoc signature verified, ZIP integrity and SHA-256 checked. The signed executable in the archive matches the local app. The built website contains the same verified archive; a live download from localhost:8790 also matches its SHA-256.
- App: `target/showcase/gpuicn Workspace.app`; download metadata: `web/public/downloads/showcase.json`. Build label `a64266f-local` identifies an uncommitted working-tree build, not an immutable release commit.

## Task-entry and introduction follow-up

- Search now aligns with the task list, and the 32px task composer replaces the footer action while open. Closing and reopening it preserves the draft.
- The introduction has a labeled close button. Dismissal survives restart; older workspace files default to showing it.
- All 11 showcase tests, showcase Clippy with warnings denied, native release build, and `git diff --check` passed.
- Native checks covered search, task submission with Enter, composer close/reopen, light and dark appearance, and dismissal across restart using isolated data. The updated app was then reopened with the user's workspace.
- Refreshed local app and ZIP: **7,050,332 bytes**. ZIP integrity, SHA-256, and executable equality with the packaged app passed. Earlier website download checks above apply to the earlier package.

## New task focus follow-up

- The header action now focuses the composer through a stable Input focus handle, including when it is already open. Existing drafts remain intact.
- All 44 workspace tests passed, including a regression that moves focus from search back to the open composer and types into it. Showcase Clippy and the release build passed; registry source and generated API pages were refreshed.
- Native checks covered opening the composer, refocusing it from search, returning from Local chat, and submitting with Enter. The updated app was reopened with the user's data.
- Latest local ZIP: **7,050,186 bytes**. ZIP integrity, SHA-256, and executable equality with the packaged app passed.

## Remaining limits

Notarization, public deployment, other operating systems, and user visual
acceptance remain outside the completed checks.
