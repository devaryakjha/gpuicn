# gpuicn

Open-code components for [GPUI](https://www.gpui.rs/), built on
[Base GPUI](https://github.com/LukeTandjung/base-gpui).

The project follows the shadcn/ui model: install component source into your
app, then own and adapt it. It ports visual identity and themes to idiomatic
GPUI APIs; it does not copy React APIs.

The registry covers all 37 component families exposed by the pinned Base GPUI
release, styled with shadcn's Neutral Nova defaults. The catalog uses real
GPUI/WASM previews; component source stays editable after installation.

- [Catalog](https://ui.imajha.com/)
- [Registry setup](docs/registry.md)
- [Source pins](release-pins.toml)
- [Full Lucide icon library](https://github.com/devaryakjha/gpui-icons)

## Catalog development

The TanStack/shadcn website lives in `web/`; the Rust crate in `site/` builds
the embedded GPUI/WASM previews.

Use Rust 1.95 for native development, the pinned WASM nightly, Trunk 0.21.14,
and Bun. On macOS, native builds require Xcode's Metal toolchain.

```sh
rustup toolchain install nightly-2026-08-17 --component rust-src --target wasm32-unknown-unknown
cd site
RUSTUP_TOOLCHAIN=nightly-2026-08-17 CARGO_UNSTABLE_BUILD_STD=std,panic_abort trunk build --release --dist ../web/public/demo --public-url /demo/
cd ../web
bun install
bun run dev
```

The site generates its code examples and API reference from the same Rust source
used by the previews. It also includes a searchable gallery of all 1,776 Lucide
icons, backed by the standalone `gpui-icons` library.

## Verify changes

```sh
cargo test -p gpuicn --lib --locked
cargo clippy -p gpuicn --lib --tests --locked -- -D warnings
cargo check -p gpuicn-showcase --locked
cargo check -p gpuicn --example usage --locked
cd web
bun run build
bun run lint
```

CI also installs every registry item with the stock shadcn CLI, compares the
installed source byte for byte, tests overwrite behavior, and compiles the app.

## Platform scope

The components use Base GPUI's native state and input behavior. The browser
catalog renders the same Rust components through WebGPU and WASM. It needs a
WebGPU-capable browser; source and installation instructions remain available
when a preview cannot start.

The WASM canvas does not expose a browser accessibility tree. Relationship
attributes, nested overlays, motion, and other differences are documented per
component under `docs/parity/` and in the catalog. These are platform limits,
not claims of complete browser or shadcn behavioral parity.
