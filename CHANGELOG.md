# Changelog

Notable changes to gpuicn. Detailed notes for each release are in
[docs/releases](docs/releases/). Installed source never updates itself; run
`gpuicn diff` after upgrading to see what a release changes in your copy.

## 0.5.0

[Notes](docs/releases/v0.5.0.md)

The first release without a beta suffix. gpuicn is still experimental, and
APIs can change between minor versions.

### Changed

- GPUI Kit 0.8.1 on its pinned `gpui-pre` 0.3.10 runtime (was 0.6.4 on 0.3.5).
  Update `gpui-kit`, `gpui_platform` and `gpui-icons` together; `gpuicn init`
  writes the new pins.
- gpui-icons 0.5.0 with Lucide 1.63.0: 1,924 icons, up from 1,818. Icons
  Lucide renamed, such as `LucideIcon::Music`, still compile.
- The WASM build uses wasm-bindgen 0.2.127.
- The minimum Rust is now 1.98, which gpui-pre 0.3.10 needs on macOS. CI
  builds with 1.99.1.

## 0.5.0-beta.5

[Notes](docs/releases/v0.5.0-beta.5.md)

### Added

- `gpuicn new <path>` creates an app ready for `cargo run`.
- `gpuicn init` now sets up an existing app. It adds the release's pinned Cargo
  dependencies, downloads Geist with checksum verification, and writes a starter
  window over the `cargo new` default. `--config-only` keeps the old behavior.
- `gpuicn status`, `gpuicn diff` and `gpuicn remove` for managing installed source.
- A `cargo gpuicn` alias is installed with the CLI.
- Crate mode: `gpuicn new --crate` and `gpuicn init --crate` depend on the
  `gpuicn` crate instead of copying source. `gpuicn eject <component>` copies
  one component into `src/ui` to edit, and `gpuicn remove` hands it back to the
  crate. `ui::<component>` paths work the same in both modes.
- The `gpuicn` crate is ready for crates.io, and gpui-icons is pinned to 0.4.2.
- `UiTheme::from_shadcn_css` imports shadcn and tweakcn theme CSS.
- Component names accept `alert_dialog` or `alert-dialog`. Unknown names
  suggest close matches, and `add` warns about missing Cargo dependencies.

### Changed (breaking)

Component APIs now share one shape: required inputs go to `new`, everything
optional is a builder method, and `cx` is taken only when a function reads it.
Run `gpuicn diff` to see the new source, then update call sites:

| Before | After |
| --- | --- |
| `toasts.push(id, title, description, Some(timeout), cx)` | `toasts.push(id, Notice::new(title).description(description).timeout(timeout), cx)`; use `.persistent()` for no timeout |
| `Resizable::new(id, label, size, first, second, on_resize)` | `Resizable::new(id, label, size).first(first).second(second).on_resize(on_resize)` |
| `SidebarLayout::new(id, state, mobile, sidebar, content, on_change)` | `SidebarLayout::new(id, state).mobile(mobile).sidebar(sidebar).content(content).on_change(on_change)` |
| `dialog_trigger(id, &handle, cx)` | `dialog_trigger(id, &handle)` |
| `dialog_action(id, &handle, cx)` | `dialog_action(id, &handle)` |
| `fieldset_root(id, cx)` | `fieldset(id, cx)` |

### Changed

- Component source moved from `registry/` to `crates/ui/registry/` so the
  `gpuicn` crate can be packaged. Crate-internal helpers are now public but
  hidden from the docs, because ejected components reach them from outside.
- The CLI shares the gpuicn version, and its default registry is the snapshot
  for that release tag rather than the live site.
- Minimum supported Rust is now 1.97 (was 1.97.1), with gpui-icons pinned to a
  revision that supports it.
- `UiTheme::read` panics with instructions to call `ui::theme::init(cx)` when
  the theme is missing.
- CI runs the CLI and UI tests on Linux on the minimum Rust, and sets up a new
  app with the CLI.

## 0.5.0-beta.4

GPUI Kit 0.6.4, GPUI 0.3.5 and gpui-icons 0.4.1. Adds Carousel, Empty and Input
Group, and Textarea as its own install target. [Notes](docs/releases/v0.5.0-beta.4.md)

## 0.5.0-beta.3

Focus is now shown with a border color only. `UiTheme::focus_ring()` and
`UiTheme::destructive_focus_ring()` were removed. [Notes](docs/releases/v0.5.0-beta.3.md)

## 0.5.0-beta.2

Moves every component and gpuicn Workspace to GPUI Kit 0.6.1, with Kit-based
input, menu, selector, form, dialog and drawer APIs. [Notes](docs/releases/v0.5.0-beta.2.md)

## 0.5.0-beta.1

Adds Sidebar, Resizable, Virtual List, the native source-installation CLI and
the gpuicn Workspace example app. [Notes](docs/releases/v0.5.0-beta.1.md)
