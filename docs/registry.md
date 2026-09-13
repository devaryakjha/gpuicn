# Install from the registry

The registry copies editable Rust files into `src/ui`. It never edits
`Cargo.toml`. The [installation guide](https://ui.imajha.com/installation)
includes a complete working starter.

## One-time setup

Use Rust 1.97.1 or newer. gpuicn v0.5.0-beta.3 supports GPUI Kit 0.6.1
on its pinned `gpui-pre` 0.3.4 runtime. Keep the dependency pins below
together. Apps using another GPUI revision must migrate to this runtime before
installing the components.

Create a Rust app with `cargo new my-app`, then add these dependencies:

```toml
[dependencies]
web-time = "1.1"
gpui_platform = { package = "gpui-pre-platform", version = "=0.3.4", features = ["font-kit"] }
gpui-icons = { git = "https://github.com/devaryakjha/gpui-icons", rev = "01ac07dd83e97f9d6a3526466413732fbdfc2975" }

gpui-kit = { version = "=0.6.1", default-features = false }

[dev-dependencies]
gpui-kit = { version = "=0.6.1", default-features = false, features = ["test-support"] }
```

Import GPUI types through `gpui_kit` so the application and installed components
share the same GPUI types. No compatibility crate or Cargo patch is required.

## Install the native CLI

The CLI is not published to crates.io yet. Install the current published beta
from its Git tag:

```sh
cargo install gpuicn-cli --git https://github.com/devaryakjha/gpuicn --tag v0.5.0-beta.3 --locked
```

From your app directory, initialize a config pointing to the registry from that
same release:

```sh
gpuicn --registry https://raw.githubusercontent.com/devaryakjha/gpuicn/v0.5.0-beta.3/site/pages/r init
gpuicn list
gpuicn add button
```

`gpuicn.toml` contains only the registry and destination. Paths resolve beside
this config; an HTTP(S) URL can replace the local registry directory. Pin a
registry snapshot if you need reproducible installs.

```toml
version = 1
registry = "https://raw.githubusercontent.com/devaryakjha/gpuicn/v0.5.0-beta.3/site/pages/r"
output = "src/ui"
```

This URL is a reproducible snapshot of the current published beta. It does not
include fixes made after the tag. Use a later release tag when those fixes are
published.

The installer copies Button and its shared theme and adds declarations to
`src/ui/mod.rs`, preserving caller code. Add `mod ui;` to your crate root.
No Node.js, npm, `components.json`, or `tsconfig.json` is needed. The copied Rust
files have no CLI runtime dependency; you can edit, move, or copy them yourself.

## Add and update

```sh
gpuicn add sidebar resizable
gpuicn add --all --dry-run
gpuicn add button --dry-run --overwrite
gpuicn add button --overwrite
```

Existing edited files are kept by default. A dry run with `--overwrite` lists
the files that would be replaced without changing them. Downloads and source
validation finish before writes begin. Each file is replaced atomically, with
`mod.rs` last; an I/O failure while committing files can still leave a partial
batch. The CLI does not edit Cargo dependencies, application setup, or fonts.

To adopt a newer gpuicn release, reinstall the CLI from its new tag with
`--force` and update the registry URL to that same tag. First commit your current
app-specific edits to a branch. A dry run with `--overwrite` reports which files
would be replaced; it does not show their content diff. Repeat without
`--dry-run`, then inspect `git diff -- src/ui` and reapply the app-specific edits
you still need.

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

## Run a window

Download the fonts and their license:

```sh
mkdir -p assets/fonts
for file in Geist-Regular.ttf Geist-Medium.ttf GeistMono-Regular.ttf; do
  curl -fL "https://raw.githubusercontent.com/devaryakjha/gpuicn/v0.5.0-beta.3/site/assets/fonts/$file" -o "assets/fonts/$file"
done
curl -fL "https://raw.githubusercontent.com/devaryakjha/gpuicn/v0.5.0-beta.3/LICENSES/Geist-OFL-1.1" -o assets/fonts/OFL.txt
curl -fL "https://raw.githubusercontent.com/devaryakjha/gpuicn/v0.5.0-beta.3/fixtures/registry-install/src/main.rs" -o src/main.rs
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
