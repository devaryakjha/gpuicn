# Install from the registry

The registry copies editable Rust files into `src/ui`. The `gpuicn` CLI also
prepares a new app: it adds the pinned Cargo dependencies, downloads the Geist
fonts, and writes a starter window. The [installation guide](https://ui.imajha.com/installation)
shows the same starter.

## Quick start

Use Rust 1.97 or newer. Install the CLI, create an app, and run it:

```sh
cargo install gpuicn-cli --git https://github.com/devaryakjha/gpuicn --tag v0.5.0-beta.4 --locked
gpuicn new my-app
cd my-app && cargo run
gpuicn add sidebar dialog
```

`gpuicn new` runs `cargo new` and then `gpuicn init` in the new directory. In an
existing app, run `gpuicn init` from its root. `init` works from the release the
CLI was built from, so the registry, Cargo pins and fonts always match
`gpuicn --version`. It:

- writes `gpuicn.toml` pointing at that release's registry snapshot;
- adds the dependencies below to `Cargo.toml`, keeping any you already declare
  and warning when yours differ;
- downloads Geist and its license into `assets/fonts/`, checking each font's SHA-256;
- installs Button and the theme, then writes a starter `src/main.rs`, but only
  when `src/main.rs` is missing or still the `cargo new` default.

Running `init` again is safe: it keeps existing files and fills in what is
missing. Use `gpuicn init --config-only` to create only `gpuicn.toml`.

> `gpuicn new` and the automatic setup ship in the CLI release after v0.5.0-beta.4. With the
> v0.5.0-beta.4 CLI, `init` only writes `gpuicn.toml`; follow
> [Manual setup](#manual-setup) for the rest.

## Crate mode

`gpuicn new --crate` (or `gpuicn init --crate`) sets up the same app, but takes
the components from the `gpuicn` crate instead of copying them. It also adds
`gpuicn` to `Cargo.toml` at the CLI's version, and writes this `src/ui/mod.rs`:

```rust
pub use gpuicn::*;
```

Your code uses `ui::button::Button` and the rest, the same paths as in source
mode. When you want to change a component, eject it:

```sh
gpuicn eject select          # copies select.rs into src/ui and declares it in mod.rs
gpuicn status                # ejected components and whether they differ
gpuicn diff select           # your copy against the release
gpuicn remove select         # delete your copy; ui::select is the crate's again
```

An ejected module shadows the crate's module of the same name, so call sites
stay the same. Its shared code (the theme and any components it builds on)
still comes from the crate. That is why `theme` can't be ejected: there is one
theme for every component, and you customize it with `UiTheme::set` or
`UiTheme::from_shadcn_css`. Other crate components keep using the crate's copy
of a component you eject. `eject` tells you which ones, so you can eject them
too when you pass them its types.

`gpuicn add` is for source mode. In crate mode, every component is already there.

## Manual setup

gpuicn v0.5.0-beta.4 supports GPUI Kit 0.6.4 on its pinned `gpui-pre` 0.3.5
runtime. Keep these dependency pins together. Apps using another GPUI revision
must migrate to this runtime before installing the components.

```toml
[dependencies]
web-time = "1.1"
gpui_platform = { package = "gpui-pre-platform", version = "=0.3.5", features = ["font-kit"] }
gpui-icons = { version = "0.4.2", git = "https://github.com/devaryakjha/gpui-icons", rev = "ff42e4ed0e29ee0e2f4a00ab977490e7e46fe230" }
gpui-kit = { version = "=0.6.4", default-features = false, features = ["component"] }

[dev-dependencies]
gpui-kit = { version = "=0.6.4", default-features = false, features = ["component", "test-support"] }
```

Import GPUI types through `gpui_kit` so the application and installed components
share the same GPUI types. No compatibility crate or Cargo patch is required.

Initialize a config pointing to the registry from the same release as your CLI:

```sh
gpuicn --registry https://raw.githubusercontent.com/devaryakjha/gpuicn/v0.5.0-beta.4/site/pages/r init --config-only
gpuicn list
gpuicn add button
```

`gpuicn.toml` contains only the registry and destination. Paths resolve beside
this config; an HTTP(S) URL or a local directory can serve as the registry.

```toml
version = 1
registry = "https://raw.githubusercontent.com/devaryakjha/gpuicn/v0.5.0-beta.4/site/pages/r"
output = "src/ui"
```

The installer copies Button and its shared theme and adds declarations to
`src/ui/mod.rs`, preserving caller code. Add `mod ui;` to your crate root and
call `ui::theme::init(cx)` once at startup; components panic with that
instruction if the theme is missing. No Node.js, npm, `components.json`, or
`tsconfig.json` is needed. The copied Rust files have no CLI runtime
dependency; you can edit, move, or copy them yourself.

## Add and update

```sh
gpuicn add sidebar resizable
gpuicn add --all --dry-run
gpuicn add button --dry-run --overwrite
gpuicn add button --overwrite
```

Component names accept `alert-dialog` or `alert_dialog`; an unknown name lists
the closest matches. After adding, the CLI warns when `Cargo.toml` lacks a
dependency the new source uses.

Existing edited files are kept by default. A dry run with `--overwrite` lists
the files that would be replaced without changing them. Downloads and source
validation finish before writes begin. Each file is replaced atomically, with
`mod.rs` last; an I/O failure while committing files can still leave a partial
batch. `add` never edits Cargo.toml or your application code.

## Track, update and remove

```sh
gpuicn status            # which installed components differ from the registry
gpuicn diff dialog       # unified diff of what `add dialog --overwrite` would change
gpuicn remove carousel   # delete its own files and mod.rs lines
```

`status` and `diff` cover every installed component when you name none; a
component counts as installed when its own module, such as `src/ui/dialog.rs`,
exists. `remove` keeps files that another installed component still uses and
refuses to delete files you have edited unless you pass `--force`; use
`--dry-run` to preview.

To adopt a newer gpuicn release, reinstall the CLI from its new tag with
`--force` and update the registry URL in `gpuicn.toml` to that same tag. First
commit your current app-specific edits. `gpuicn diff` then shows exactly what
the new release changes in your source; replace files with
`gpuicn add <component> --overwrite` and reapply the edits you still need.

## Build a registry

From the gpuicn checkout:

```sh
cargo run -p gpuicn-cli --locked -- build
```

`registry.toml` describes component metadata and source dependencies. The build
writes `registry.json` and inline-source items under `site/pages/r`. This JSON
transport remains compatible with the stock shadcn installer and can be served
as static files, mirrored, or used from a local directory. The React website's
Bun/Node dependencies are separate from Rust component installation.

## Optional stock shadcn installer

The existing published registry also supports:

```sh
npx -y shadcn@4.19.0 add https://ui.imajha.com/r/button.json
```

This legacy route needs Node.js and the `components.json` / `tsconfig.json`
examples in `fixtures/registry-install/`. Declare installed modules yourself
when using it. The native CLI maintains module declarations for you.

## Run a window manually

`gpuicn init` does this for you. To set it up by hand, download the fonts,
their license and the starter:

```sh
mkdir -p assets/fonts
for file in Geist-Regular.ttf Geist-Medium.ttf GeistMono-Regular.ttf; do
  curl -fL "https://raw.githubusercontent.com/devaryakjha/gpuicn/v0.5.0-beta.4/site/assets/fonts/$file" -o "assets/fonts/$file"
done
curl -fL "https://raw.githubusercontent.com/devaryakjha/gpuicn/v0.5.0-beta.4/LICENSES/Geist-OFL-1.1" -o assets/fonts/OFL.txt
curl -fL "https://raw.githubusercontent.com/devaryakjha/gpuicn/v0.5.0-beta.4/fixtures/registry-install/src/main.rs" -o src/main.rs
cargo run
```

The starter calls `ui::theme::init(cx)` to register component actions, keyboard
traversal and the default theme, then loads Geist,
and composes the icon asset source. You can replace the fonts and theme with
your own. Native builds require the platform's GPUI build tools; macOS needs
Xcode and its Metal toolchain.

Copied sources include focused behavior checks. To run them with `cargo test`,
add the same pinned `gpui-kit` dependency under `[dev-dependencies]` with
`features = ["test-support"]`. The starter dependency block includes this.
