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

## Catalog development

The TanStack/shadcn website lives in `web/`; the Rust crate in `site/` builds
the embedded GPUI/WASM previews.

```sh
cd site
CARGO_UNSTABLE_BUILD_STD=std,panic_abort trunk build --dist ../web/public/demo --public-url /demo/
cd ../web
bun install
bun run dev
```
