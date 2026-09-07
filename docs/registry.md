# Install from the registry

The registry copies editable Rust files into `src/ui`. It never edits
`Cargo.toml`. The [installation guide](https://ui.imajha.com/installation)
includes a complete working starter.

## One-time setup

Create a Rust app with `cargo new my-app`, then add these pinned dependencies:

```toml
[dependencies]
base-gpui = { git = "https://github.com/LukeTandjung/base-gpui", rev = "64b22337b6a790c636aab248e768e4875bb28ba8" }
gpui = { git = "https://github.com/zed-industries/zed", rev = "59b2ebf10351b5c0b5cd4403f01ed0460eeec06d" }
gpui_platform = { git = "https://github.com/zed-industries/zed", rev = "59b2ebf10351b5c0b5cd4403f01ed0460eeec06d" }
gpui-icons = { git = "https://github.com/devaryakjha/gpui-icons", rev = "b25a5ebae2e1a5f4ddfca1389ab9d21d481d9ec8" }
```

Save `components.json` in your app root:

```json
{
  "$schema": "https://ui.shadcn.com/schema.json",
  "style": "new-york",
  "rsc": false,
  "tsx": false,
  "tailwind": {
    "config": "",
    "css": "",
    "baseColor": "neutral",
    "cssVariables": false
  },
  "aliases": {
    "components": "~/src",
    "utils": "~/src",
    "ui": "~/src/ui",
    "lib": "~/src",
    "hooks": "~/src"
  }
}
```

Save `tsconfig.json` beside it. The stock installer uses this file to resolve
aliases; your app remains Rust:

```json
{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "~/*": ["./*"]
    }
  }
}
```

## Install

Run the stock shadcn CLI from your app root. Node.js is required.

```sh
npx -y shadcn@4.19.0 add https://ui.imajha.com/r/button.json
```

Create `src/ui/mod.rs` for the installed modules:

```rust
pub mod theme;
pub mod button;
```

Declare `mod ui;` from your crate root. Each component page lists all required
modules, including shared dependencies. Each item installs the source it needs.
Review your local changes before re-running an install with `--overwrite`.

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

The starter registers Base GPUI actions, installs the theme, loads Geist,
and composes the icon asset source. You can replace the fonts and theme with
your own. Native builds require the platform's GPUI build tools; macOS needs
Xcode and its Metal toolchain.
