# Contributing to gpuicn

Thanks for helping. Bug reports, parity gaps and component requests go in
[GitHub issues](https://github.com/devaryakjha/gpuicn/issues). Focused pull
requests are welcome; say which checks you ran.

## Layout

| Path | What lives there |
| --- | --- |
| `crates/ui/registry/<module>/` | Component source. `gpuicn add` copies it into apps, and the `gpuicn` crate compiles it for crate mode. Mark cross-module helpers `#[doc(hidden)] pub`, never `pub(crate)`: an ejected component reaches them from outside the crate. |
| `registry.toml` | Component titles, descriptions and source dependencies. |
| `crates/ui` | The `gpuicn` crate: compiles the registry via `#[path]` modules, plus tests and examples. It is published for crate mode. |
| `crates/cli` | The `gpuicn` CLI (`init`, `new`, `add`, `status`, `diff`, `remove`, `build`). |
| `site/` | The GPUI/WASM previews and the gpuicn Workspace example app. |
| `web/` | The catalog website (TanStack, Bun). |
| `fixtures/registry-install` | A consumer app CI installs every component into. |
| `docs/` | User docs; `docs/internal/` holds plans and validation records. |
| `release-pins.toml` | Every pinned upstream version and hash. |

## Setup

Use Rust 1.99.1 for development (CI's toolchain); the minimum supported Rust is
1.98, the first release that builds gpui-pre 0.3.10 on macOS. On macOS, install Xcode's Metal toolchain. On Linux, install GPUI's system
libraries, for example on Ubuntu:

```sh
sudo apt-get install libfontconfig1-dev libxcb1-dev libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev
```

The website additionally needs the pinned WASM nightly, Trunk 0.21.14 and Bun:

```sh
rustup toolchain install nightly-2026-08-17 --component rust-src --target wasm32-unknown-unknown
cd site
RUSTUP_TOOLCHAIN=nightly-2026-08-17 CARGO_UNSTABLE_BUILD_STD=std,panic_abort trunk build --release --dist ../web/public/demo --public-url /demo/
cd ../web
bun install
bun run dev
```

## Checks

Before opening a pull request:

```sh
cargo fmt --all -- --check
cargo test --workspace --locked --features gpui_platform/runtime_shaders
cargo clippy --workspace --all-targets --locked --features gpui_platform/runtime_shaders -- -D warnings
cargo run -p gpuicn-cli --locked -- build   # regenerate registry.json and site/pages/r
node scripts/sync-catalog.mjs               # checks previews, usage, API docs and platform notes
git diff --exit-code -- registry.json site/pages/r
```

Any change under `crates/ui/registry/` changes the generated registry; commit the
rebuilt `registry.json` and `site/pages/r`. CI also installs every component
into `fixtures/registry-install` with the native CLI and the stock shadcn CLI,
compares the copies byte for byte, and builds the WASM catalog.

## Add a component

`scripts/sync-catalog.mjs` fails until every piece below exists.

1. Write `crates/ui/registry/<module>/<module>.rs`. Document every `pub fn`; the catalog
   builds its API reference from these comments. Read colors, spacing, radius
   and motion from `UiTheme`, never hard-coded values, and keep application
   data, routing and persistence caller-owned (see [CONTEXT.md](CONTEXT.md)).
2. Add `[components.<module>]` to `registry.toml` with any `dependencies`.
3. Add the `#[path]` module to `crates/ui/src/lib.rs` and tests under
   `crates/ui/tests/` or in the source file.
4. Add `mod <module> { … fn example(…) }` to `crates/ui/examples/usage.rs`,
   importing only from `crate::ui::<module>::…`. This is the usage shown on the
   site and compiled in CI.
5. Add `fn <module>_preview(` to `site/src/main.rs`.
6. Add platform notes in `docs/parity/<name>.md`: how it differs from shadcn
   and what was verified.
7. Run the checks above and commit the regenerated registry.

## Releases

1. Bump `workspace.package.version` in `Cargo.toml`. The CLI's default
   registry, fonts and dependency pins all come from tag `v<version>`.
2. Update `release-pins.toml`, `crates/cli/templates/dependencies.toml` and the
   fixture together when an upstream pin moves; the CLI tests fail if they drift.
3. Update the `cargo install gpuicn-cli --version` lines in `README.md`,
   `crates/ui/README.md`, `docs/registry.md` and the `release` constant in
   `web/src/routes/installation.tsx`.
4. Add the entry to `CHANGELOG.md` and notes in `docs/releases/`.
5. Merge, then tag `v<version>` on that commit.
6. From that commit, publish the library first and then the CLI:
   `cargo publish -p gpuicn --locked`, then `cargo publish -p gpuicn-cli --locked`.
