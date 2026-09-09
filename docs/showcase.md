# Workspace example app

The default native app is gpuicn Workspace: projects, editable tasks, completion,
priority, search, deletion with undo, and a local message log for each project.
It composes the shared SidebarLayout, Resizable, Tabs, Input, Button, Toggle,
Checkbox, Avatar, Progress, Menu, Dialog and ScrollArea components. Both app
themes use black, white and gray; the panels logo keeps orange.

Open **Component gallery** from the sidebar or app menu to browse every component
in a separate window. Search, reset and theme switching are available there.
The website and native gallery use the same component examples. The app applies
its monochrome theme through shared tokens; the website previews use the stock
Neutral theme. Menu examples show which action ran and omit unbound shortcuts.

## Editing and persistence

Task creation, saved edits, completion, priority, deletion/undo and sent messages
persist to `~/Library/Application Support/gpuicn Workspace/workspace.json`.
The first run supplies sample tasks; messages start empty. Chat is an on-device
project log with no server or remote participants. Message drafts stay separate
per project during the session.

New task/project editors focus when opened. The title pencil focuses the title
editor. Enter creates or saves a task and sends a message; sending keeps focus
in the composer. Cmd-B hides/restores the shared sidebar, including from an input.
Navigation and quitting prompt before discarding task edits. Failed writes stay
unsaved, show Retry save, and prevent a silent quit. Failed detail saves preserve
the editable draft.

The store validates IDs and references, rejects malformed or oversized data,
writes through a temporary file, and refuses to overwrite a file changed since
loading. Corrupt files remain untouched. The example supports up to 5 MB of
saved data and one app process; simultaneous writers do not share a file lock.
`GPUICN_WORKSPACE_DATA` selects an isolated data file for testing.

The shared Input supplies native text semantics, undo/redo, word movement and
deletion, selection, IME composition, and contextual callbacks usable with
`cx.listener`. `field_control` uses it too. Enter submits an enclosing form and
validates required fields, unless the input has its own submit handler. Call
`ui::theme::init(cx)` when using installed source so keyboard bindings are active.
See [Input parity](parity/input.md) for limits.

## Build and package

```sh
cargo run --release -p gpuicn-showcase --features gpui_platform/runtime_shaders
python3 scripts/package-showcase.py
open 'target/showcase/gpuicn Workspace.app'
# Dark workspace or standalone component gallery:
cargo run --release -p gpuicn-showcase --features gpui_platform/runtime_shaders -- --workspace dark
cargo run --release -p gpuicn-showcase --features gpui_platform/runtime_shaders -- --catalog
```

The packaging script embeds fonts and the app icon, verifies the code signature,
and writes the ZIP, checksum and manifest under `web/public/downloads/`.
The `/showcase` page downloads that archive. The build architecture matches its
host; local build validation covers Apple silicon.

## Distribution and verification

Without signing settings, the script creates an ad-hoc signed local preview.
For public releases, set `SHOWCASE_SIGN_IDENTITY` to a Developer ID Application
identity and `SHOWCASE_NOTARY_PROFILE` to credentials stored by `notarytool` in
the macOS Keychain. The script signs with the hardened runtime, submits to Apple,
staples and validates the ticket, and checks Gatekeeper before writing download
metadata. Credentials and private keys stay outside the repository.

Upload the ZIP, its `.sha256` file, and `showcase.json` to the matching GitHub
release. Run **Deploy ui.imajha.com** with that release tag. Deployment checks
the version, source revision, notarization flag, file size, and checksum before
building the catalog with the same download. The release tag and packaged source
must refer to the same clean commit.

[The audit record](validation/showcase.md) distinguishes automated checks,
completed native/browser journeys, and remaining limits. NumberField and toolbar
inputs now expose native editor roles, names and values.
Both now use the shared editor for undo/redo and word editing while retaining
numeric rules and toolbar focus behavior; see their parity notes.
