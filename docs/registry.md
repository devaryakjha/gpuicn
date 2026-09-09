# Install from the registry

The registry copies editable Rust files into `src/ui`. It never edits
`Cargo.toml`. The [installation guide](https://ui.imajha.com/installation)
includes a complete working starter.

## One-time setup

Use Rust 1.97.1 or newer. Keep the dependency pins below together.

Create a Rust app with `cargo new my-app`, then add these dependencies:

```toml
[dependencies]
unicode-segmentation = "1.13"
web-time = "1.1"
base-gpui = { git = "https://github.com/LukeTandjung/base-gpui", rev = "64b22337b6a790c636aab248e768e4875bb28ba8" }
gpui = { package = "gpui-pre", version = "=0.3.4" }
gpui_platform = { package = "gpui-pre-platform", version = "=0.3.4", features = ["font-kit"] }
gpui-icons = { git = "https://github.com/devaryakjha/gpui-icons", rev = "01ac07dd83e97f9d6a3526466413732fbdfc2975" }

gpui-base = "=0.6.1"
gpui-kit = { version = "=0.6.1", default-features = false }

[patch."https://github.com/zed-industries/zed"]
gpui = { git = "https://github.com/devaryakjha/gpuicn", rev = "64d1bd9678d365c2f1367431c700c36184fb5142" }
```

The `gpui` patch is a small re-export crate. It directs retained `base-gpui`
code to the same `gpui-pre` runtime used by Kit. Cargo patches must appear in
the consuming application's root manifest; they do not propagate through dependencies.
Do not omit it or mix the old Zed Git runtime with Kit types.

## Install the native CLI

The new CLI is a local preview and is not published to crates.io yet. From this
checkout, build and install it with Rust alone:

```sh
cargo install --path crates/cli --locked
```

From your app directory, initialize a config pointing to this checkout's built
registry (replace `/path/to/gpuicn` with its location):

```sh
gpuicn --registry /path/to/gpuicn/site/pages/r init
gpuicn list
gpuicn add button
```

`gpuicn.toml` contains only the registry and destination. Paths resolve beside
this config; an HTTP(S) URL can replace the local registry directory. Pin a
registry snapshot if you need reproducible installs.

```toml
version = 1
registry = "/path/to/gpuicn/site/pages/r"
output = "src/ui"
```

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

Existing edited files are kept by default. Use version control to review a dry
run and diff before opting into replacement. Downloads and source validation
finish before writes begin. Each file is replaced atomically, with `mod.rs`
last; an I/O failure while committing files can still leave a partial batch.
The CLI does not edit Cargo dependencies, application setup, or fonts.

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
for file in Geist-Regular.ttf Geist-Medium.ttf GeistMono-Regular.ttf OFL.txt; do
  curl -fL "https://ui.imajha.com/fonts/$file" -o "assets/fonts/$file"
done
curl -fL https://ui.imajha.com/examples/hello.rs -o src/main.rs
cargo run
```

The starter calls `ui::theme::init(cx)` to register component actions, keyboard
traversal and the default theme, then loads Geist,
and composes the icon asset source. You can replace the fonts and theme with
your own. Native builds require the platform's GPUI build tools; macOS needs
Xcode and its Metal toolchain.

Copied sources include focused behavior checks. To run them with `cargo test`,
add the same pinned `gpui` dependency under `[dev-dependencies]` with
`features = ["test-support"]`. The starter dependency block includes this.
