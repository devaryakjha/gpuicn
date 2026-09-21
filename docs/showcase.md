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
persist to `~/Library/Application Support/gpuicn Workspace/workspace.json` on
macOS. Linux uses `$XDG_DATA_HOME/gpuicn-workspace/workspace.json`, or
`~/.local/share/gpuicn-workspace/workspace.json` when XDG_DATA_HOME is unset or
relative. Existing Linux trial data in the older `~/Library` location remains in
use when no file exists at the new location. `GPUICN_WORKSPACE_DATA` overrides
the file path on both platforms.
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

### Experimental Linux build

Build and package the native x86_64 app on Ubuntu 24.04:

```sh
python3 scripts/package-showcase-linux.py
# Repackage an existing target/release/showcase:
python3 scripts/package-showcase-linux.py --skip-build
# Cross-build ARM64 after installing its Rust target, linker and system libraries:
python3 scripts/package-showcase-linux.py --target aarch64-unknown-linux-gnu
```

The script builds the release app with X11 and Wayland support, strips the
executable, and writes an architecture-specific `.tar.gz`, checksum and manifest
under `target/linux-dist/`. Linux needs separate x86_64 and ARM64 downloads;
these are not universal binaries. Both target Ubuntu 24.04 and glibc 2.39 and
need `libxcb1`, `libxkbcommon0`, `libxkbcommon-x11-0`, `libfontconfig1`,
`libwayland-client0`, `libwayland-cursor0`, `libvulkan1` and a working Vulkan
driver at runtime. Treat them as experimental until tested across real Linux desktops.
The remote software-rendered x86_64 check does not qualify graphics performance
or broad distribution compatibility, and the ARM64 runtime is untested.

On an x86_64 Linux build host, the included Docker recipe supplies ARM64 system
libraries without changing the host's package architectures:

```sh
rustup target add aarch64-unknown-linux-gnu
docker build -t gpuicn-linux-arm64-build - < scripts/linux-arm64.Dockerfile
docker run --rm --user "$(id -u):$(id -g)" \
  -v "$PWD:/source" -v "$HOME/.cargo:/cargo" -v "$HOME/.rustup:/rustup" \
  -e CARGO_TARGET_DIR=/source/target/arm64 gpuicn-linux-arm64-build \
  python3 scripts/package-showcase-linux.py --target aarch64-unknown-linux-gnu
```

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

The GitHub workflow requires `CLOUDFLARE_API_TOKEN` and
`CLOUDFLARE_ACCOUNT_ID` repository secrets. A signed-in local release checkout
can instead build the catalog with `cd web && bun run build`, then run
`bunx wrangler@4.129.0 deploy` from the repository root. Keep the verified
release download files in `web/public/downloads/` for either path.

[The audit record](validation/showcase.md) distinguishes automated checks,
completed native/browser journeys, and remaining limits. NumberField and toolbar
inputs now expose native editor roles, names and values.
Both now use the shared editor for undo/redo and word editing while retaining
numeric rules and toolbar focus behavior; see their parity notes.

To build macOS releases without compiling on your laptop, run **Build release
downloads** on the release commit. Download `showcase-macos-arm64`, put its
`showcase` executable at `target/release/showcase` in a clean checkout of that
same commit, restore its executable bit, and run `scripts/package-showcase.py
--skip-build` with the signing settings above. Confirm the workflow's commit
matches the checkout before signing.

The same workflow packages Linux x86-64 and ARM64 archives on Ubuntu 24.04
runners. Download their archive, checksum and manifest artifacts and attach
them to the matching release. These build jobs do not qualify desktop runtime
behavior; keep the experimental labels until native testing supports changing them.
