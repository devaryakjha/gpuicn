# Install from the registry

The registry copies editable Rust files into `src/ui`. The `gpuicn` CLI also
prepares a new app: it adds the pinned Cargo dependencies, downloads the Geist
fonts, and writes a starter window. The [installation guide](https://ui.imajha.com/installation)
shows the same starter.

## Quick start

Use Rust 1.98 or newer. Install the CLI, create an app, and run it:

```sh
cargo install gpuicn-cli --version 0.9.9 --locked
gpuicn new my-app
cd my-app && cargo run
gpuicn add sidebar dialog
```

`gpuicn new` runs `cargo new` and then `gpuicn init` in the new directory. In an
existing app, run `gpuicn init` from its root. `init` works from the release the
CLI was built from, so the registry, Cargo pins and fonts always match
`gpuicn --version`. It:

- writes `gpuicn.toml` pointing at that release's registry snapshot, with the
  theme you chose (`--base-color`, `--radius`, `--appearance`, or `--theme` for
  a shadcn CSS file or URL);
- adds the dependencies below to `Cargo.toml`, keeping any you already declare
  and warning when yours differ;
- downloads Geist and its license into `assets/fonts/`, checking each font's SHA-256;
- writes `assets/theme.css` from the base color and generates
  `src/ui/app_theme.rs`, whose `light()`, `dark()` and `APPEARANCE` the starter
  passes to `ui::app::run`;
- installs the theme and `app`, then Button and a starter `src/main.rs`, but
  only when `src/main.rs` is missing or still the `cargo new` default. Pass
  `--all` to install every component.

Running `init` again is safe: it keeps existing files and fills in what is
missing. Use `gpuicn init --config-only` to create only `gpuicn.toml`.

## Theme

```sh
gpuicn theme set --base-color stone            # neutral, stone, zinc, mauve, olive, mist or taupe
gpuicn theme set --radius 0.5rem --appearance dark
gpuicn theme import theme.css                  # or a URL; the shadcn theme editor's CSS works
gpuicn theme preview                           # every token, light and dark
```

The base colors are shadcn 4.19.0's own. `theme set` rewrites
`assets/theme.css` and `src/ui/app_theme.rs`; `theme import` copies your CSS
there instead and clears the base color in `gpuicn.toml`, so later `--radius`
changes apply to your CSS. Edit `assets/theme.css` by hand when you prefer:
`init` keeps an edited file, and `theme set` replaces it. See
[theming](theming.md) for what the CSS may contain.

## Update and doctor

```sh
gpuicn update                 # to the latest release
gpuicn update --to 0.9.0      # to a release; --dry-run shows the changes
gpuicn update --overwrite     # also replace installed files with the release's
gpuicn doctor                 # exit 1 with a fix line per finding
```

`update` points `gpuicn.toml` at the release's registry, replaces the Cargo
pins with that release's, fetches fonts you lack, regenerates the theme files,
and then runs `status` so you can `diff` before `--overwrite`. It ends by
printing the `cargo install` line for the matching CLI. `doctor` checks the
config version, the registry release against the CLI, the Cargo pins, the
fonts and their license, `assets/theme.css`, `app_theme.rs`, how `main.rs`
starts, and whether installed components match the registry.

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

gpuicn v0.9.9 supports GPUI Kit 0.9.0 on its pinned `gpui-pre` 0.3.11
runtime. Keep these dependency pins together. Apps using another GPUI revision
must migrate to this runtime before installing the components.

```toml
[dependencies]
web-time = "1.1"
gpui_platform = { package = "gpui-pre-platform", version = "=0.3.11", features = ["font-kit"] }
gpui-icons = "=0.6.0"
gpui-kit = { version = "=0.9.0", default-features = false, features = ["component"] }

[dev-dependencies]
gpui-kit = { version = "=0.9.0", default-features = false, features = ["component", "test-support"] }
```

Import GPUI types through `gpui_kit` so the application and installed components
share the same GPUI types. No compatibility crate or Cargo patch is required.

Initialize a config pointing to the registry from the same release as your CLI:

```sh
gpuicn --registry https://raw.githubusercontent.com/karyalabs/gpuicn/v0.9.9/site/pages/r init --config-only
gpuicn list
gpuicn add button
```

`gpuicn.toml` holds the registry, the destination and the theme. Paths resolve
beside this config; an HTTP(S) URL or a local directory can serve as the
registry. Version 1 configs (without `[theme]`) still load; `theme set` and
`update` rewrite them as version 2.

```toml
version = 2
registry = "https://raw.githubusercontent.com/karyalabs/gpuicn/v0.9.9/site/pages/r"
output = "src/ui"

[theme]
base_color = "neutral"
appearance = "system"
```

The installer copies Button and its shared theme and adds declarations to
`src/ui/mod.rs`, preserving caller code. Add `mod ui;` to your crate root and
start through `ui::app::run`, or call `ui::theme::init(cx)` once at startup;
components panic with that instruction if the theme is missing. No Node.js, npm, `components.json`, or
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
`--dry-run` to preview. `remove --unused` finds the installed components no
Rust file outside `src/ui` names (as `ui::dialog::…` or in a `use` list) and
removes them; `theme` and `app` always stay.

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
bunx shadcn@4.19.0 add https://ui.imajha.com/r/button.json
```

This legacy route needs Node.js and the `components.json` / `tsconfig.json`
examples in `fixtures/registry-install/`. Declare installed modules yourself
when using it. The native CLI maintains module declarations for you.

## Run a window manually

`gpuicn init` does this for you. To set it up by hand, download the fonts,
their license and the starter:

```sh
mkdir -p assets/fonts
for file in Geist-Regular.ttf Geist-Medium.ttf Geist-SemiBold.ttf GeistMono-Regular.ttf; do
  curl -fL "https://raw.githubusercontent.com/karyalabs/gpuicn/v0.9.9/site/assets/fonts/$file" -o "assets/fonts/$file"
done
curl -fL "https://raw.githubusercontent.com/karyalabs/gpuicn/v0.9.9/LICENSES/Geist-OFL-1.1" -o assets/fonts/OFL.txt
curl -fL "https://raw.githubusercontent.com/karyalabs/gpuicn/v0.9.9/fixtures/registry-install/src/main.rs" -o src/main.rs
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
