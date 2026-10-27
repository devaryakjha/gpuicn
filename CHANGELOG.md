# Changelog

Notable changes to gpuicn. Detailed notes for each beta are in
[docs/releases](docs/releases/). Installed source never updates itself; run
`gpuicn diff` after upgrading to see what a release changes in your copy.

## Unreleased

### Added

- `gpuicn new <path>` creates an app ready for `cargo run`.
- `gpuicn init` now sets up an existing app. It adds the release's pinned Cargo
  dependencies, downloads Geist with checksum verification, and writes a starter
  window over the `cargo new` default. `--config-only` keeps the old behavior.
- `gpuicn status`, `gpuicn diff` and `gpuicn remove` for managing installed source.
- A `cargo gpuicn` alias is installed with the CLI.
- `UiTheme::from_shadcn_css` imports shadcn and tweakcn theme CSS.
- Component names accept `alert_dialog` or `alert-dialog`. Unknown names
  suggest close matches, and `add` warns about missing Cargo dependencies.

### Changed

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
