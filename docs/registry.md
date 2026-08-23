# Install from the registry

The registry copies editable Rust files into `src/ui`. It never edits `Cargo.toml` or creates a GPUI adapter.

## One-time setup

Pin the same dependencies used by the v0.1 registry:

```toml
[dependencies]
base-gpui = { git = "https://github.com/LukeTandjung/base-gpui", rev = "64b22337b6a790c636aab248e768e4875bb28ba8" }
gpui = { git = "https://github.com/zed-industries/zed", rev = "59b2ebf10351b5c0b5cd4403f01ed0460eeec06d" }
gpui-icons = { git = "https://github.com/devaryakjha/gpui-icons", rev = "53a54cfe5efc8eacb546de0c9742339b667381b2" }
```

Create `src/ui/mod.rs` and declare only the items you install:

```rust
pub mod theme;
pub mod button;
pub mod checkbox;
pub mod dialog;
```

Declare `mod ui;` from your crate root. At app startup, register the Base GPUI actions, install a `UiTheme`, load the Geist font faces, and compose `gpui_icons::LucideAssetSource` into the application asset source. The files in `site/src/main.rs` show the complete setup.

## Install

Use the stock shadcn CLI. Dialog pulls its immutable Button and theme dependencies first.

```sh
npx -y shadcn@4.19.0 add https://devaryakjha.github.io/ui/r/dialog.json --overwrite
```

Replace `dialog.json` with `button.json`, `checkbox.json`, or `theme.json` for a smaller install. Re-running the command with `--overwrite` restores the pinned source exactly.
