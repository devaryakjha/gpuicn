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
| `site/` | The GPUI/WASM previews (one file per component under `site/src/previews/`) and the gpuicn Workspace example app. |
| `xtask/` | `cargo xtask new-component <name>` scaffolds a component in every place above. |
| `web/` | The catalog website: static Astro pages with plain CSS, built with Bun. `bun run build` regenerates the catalog data first. |
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
bun scripts/sync-catalog.mjs               # checks previews, usage, API docs and platform notes
git diff --exit-code -- registry.json site/pages/r
```

Any change under `crates/ui/registry/` changes the generated registry; commit the
rebuilt `registry.json` and `site/pages/r`. CI also installs every component
into `fixtures/registry-install` with the native CLI and the stock shadcn CLI,
compares the copies byte for byte, and builds the WASM catalog.

## Design rules are checked

`cargo test -p gpuicn --test registry_lint` reads every file under
`crates/ui/registry` and fails on patterns the shared primitives replace:
raw colors, raw text sizes, `opacity(0.5)` for disabled, `ThemeMode::Dark`
branches outside the theme, hand-written popover surfaces, `img` inside a
rounded clip, hover state kept in `use_keyed_state`, key glyph literals, and
components without any test. `crates/ui/tests/registry_lint_baseline.txt`
lists what was allowed when a rule was added; counts may only go down, and the
test tells you which baseline lines to lower. After adding a rule, run the test
once with `REGISTRY_LINT_WRITE_BASELINE=1` to record the current state.

`scripts/screenshots.mjs` shoots every preview in light and dark mode from the
built site and writes a Markdown sheet, comparing against the newest folder in
`docs/screenshots/`. The release verification runs it and uploads the sheet as
the `screenshots` artifact; run it locally for a pull request that changes
how anything looks:

```sh
(cd site && trunk build --release --public-url /demo/ --dist ../web/public/demo)
bun scripts/screenshots.mjs --dist web/public/demo --out docs/screenshots/v<next> --compare docs/screenshots/v<previous>
```

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

Merging a version bump to `main` is the release; nothing runs on a laptop.

1. Run `python3 scripts/release-version.py bump <version>`. It rewrites every
   place the version is written (Cargo.toml, the install lines, the registry
   URLs in the docs and the site's `release` constant) and updates Cargo.lock.
   CI fails while any of them disagree.
2. Write `docs/releases/v<version>.md` (the GitHub release notes) and the
   `## <version>` entry in `CHANGELOG.md`, then run
   `cargo run -p gpuicn-cli --locked -- build`.
3. Update `release-pins.toml`, `crates/cli/templates/dependencies.toml` and the
   fixture together when an upstream pin moves; the CLI tests fail if they drift.
   A new gpui-icons version must be released from its own repository first.
4. Merge. When CI passes on `main`, the **Release** workflow:
   - tags the commit `v<version>`;
   - publishes `gpuicn` and then `gpuicn-cli` to crates.io;
   - builds the macOS app (signed and notarized when the Apple secrets exist)
     and the Linux x86-64 and ARM64 archives;
   - publishes the GitHub release with every download and checksum;
   - deploys ui.imajha.com from that release.

A merge that doesn't change the version releases nothing. If a job fails,
fix the cause and use **Re-run failed jobs**; every step skips what is already
done. **Release** can also be started by hand on `main`.

The `release` environment holds the secrets:

| Secret | Used for |
|---|---|
| none (crates.io Trusted Publishing) or `CARGO_REGISTRY_TOKEN` | Publishing the crates |
| `MACOS_CERT_P12`, `MACOS_CERT_PASSWORD` | The Developer ID Application certificate, as base64 .p12 |
| `APPLE_API_KEY_P8`, `APPLE_API_KEY_ID`, `APPLE_API_ISSUER_ID` | Notarization with an App Store Connect API key |
| `CLOUDFLARE_API_TOKEN`, `CLOUDFLARE_ACCOUNT_ID` | Deploying ui.imajha.com |

