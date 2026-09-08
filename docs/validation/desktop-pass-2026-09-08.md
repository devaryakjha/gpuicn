# Local desktop and distribution pass — 8 September 2026

This is a local review build on `codex/v0.3-component-fidelity`. No release, commit, push, or deployment belongs to this pass. The running scope and remaining work are in [TODO.md](../../TODO.md).

## Changes ready to review

- Added caller-controlled Sidebar and Resizable components, registry items, compiled usage examples, catalog previews, and a native review app. The app owns navigation, selected state, pane sizes, and collapse state.
- Applied shared spacing and type tokens throughout the existing catalog; added a radius constructor, overlay color, and theme motion settings. Wrapped controls accept GPUI `Styled` overrides after defaults. Theme switching preserves non-color tokens and initializes correctly on first use.
- Added interruptible sidebar width, switch-thumb, and disclosure-icon transitions with reduced-motion handling. Retained children keep input and list state while the layout changes.
- Added a Rust `gpuicn` CLI with `gpuicn.toml`, local/HTTP(S) registries, list/add/dry-run/explicit overwrite, shared source installation, path validation, and module declarations. Native registry generation replaces the Node generator. The JSON transport remains compatible with shadcn.

## Verification

Local native toolchain: Rust 1.97.1, Apple M4 Max, macOS 27.0 (26A5425a). CI remains configured for Rust 1.95; that exact toolchain was not run locally.

| Evidence | Result and scope |
| --- | --- |
| Workspace tests | 16 library, 1 theme integration, 2 CLI, and 3 showcase tests pass. Includes all component previews in light, dark, and custom tokens; resize bounds/input; transition reversal; first-theme initialization; installer preservation/path validation. |
| Static checks | Workspace Clippy with warnings denied, usage example check, Rust formatting, and diff whitespace checks pass. |
| Fresh native consumer | Installed all 40 registry items (39 components plus theme) into a clean temporary Rust app. No Node or JSON app config involved. All copied modules compile; the release starter opens and its button responds to a click. |
| HTTP source path | Loopback HTTP registry installs Menu and Sidebar with their transitive Button, Theme, Scroll Area, and Tooltip sources. |
| Legacy installer | Stock shadcn 4.19.0 installs every item; source matches byte for byte. Explicit Dialog overwrite restores the source and the fixture compiles with its lockfile. |
| macOS interface | Long sidebar labels truncate; active and disabled actions behave correctly. Mouse drag, arrow-key resize, accessible SetValue, collapse/restore, reduced-motion toggle, theme/personalization, text retention, dialog opening, and Escape dismissal were exercised in the app. |
| Browser/docs | Installation guide exposes native CLI/TOML setup. Production web build, TypeScript, lint, highlighting, and preview message checks pass. Release WASM builds with the pinned nightly/Trunk. Browser startup caught and drove the first-theme initialization fix; Sidebar and Resizable then rendered in the live browser. |

The native accessibility tree exposes navigation names, current/disabled descriptions, switch state, and settable splitters. This is a smoke check, not a complete VoiceOver review. The pinned input and browser canvas still have documented accessibility limits.

## Release performance sample

The native `desktop --benchmark` run changes both pane dimensions, scrolls a retained 100,000-row fixed-height list, and toggles sidebar collapse every 120 frames. It discards 60 warmup frames and records 600 CPU samples plus 599 paint intervals. No network or domain processing is part of this workload.

| Metric | Result |
| --- | --- |
| Render-start to canvas-paint CPU p50 / p95 / p99 | 2.18 / 2.91 / 3.22 ms |
| Paint interval p50 / p95 / p99 | 8.28 / 9.65 / 10.47 ms |
| Maximum paint interval | 12.85 ms |
| Maximum rows requested in one render | 14 of 100,000 |
| Peak sampled process RSS | 100.44 MiB (20 samples at roughly 250 ms) |

Raw [release frames](desktop-release.json), [memory samples](desktop-release-memory.json), and [earlier debug frames](desktop-debug-before.json) are retained. The earlier debug p95 CPU value was 4.61 ms, but it used a different profile and no collapse toggles; it is not a controlled before/after speedup comparison. These measurements exclude GPU completion and input-to-display latency. Sampled RSS is not a leak test or GPU-memory measurement. Variable-height lists, trees, diff text, and domain workloads need their own qualification.

## User test steps and limits

Run from the repository:

```sh
cargo run -p gpuicn --release --example desktop --features native-fixture,gpui_platform/runtime_shaders
```

Type into the input, resize either separator with mouse and keyboard, collapse and restore repeatedly, switch themes and personalization, then open/dismiss the dialog. Try the reduced-motion toggle. Narrow the window: pane minima remain intact and the group scrolls. The local catalog is at `http://127.0.0.1:8790` while its dev server is running.

The CLI is unpublished. Use `cargo install --path crates/cli --locked` from this checkout; [registry setup](../registry.md) shows a local registry and upgrade workflow. Each installed file is atomic, but a filesystem error while committing a batch can leave some files updated. Inspect the diff before overwriting local edits.

Full dialog/toast/disclosure exit animation waits on Base GPUI presence support. Applications must wire the OS reduced-motion preference. Base composition helper callbacks replace their default skin; use tokens or edit installed source for small changes. See [theming](../theming.md). Full native catalog/VoiceOver review, Windows/Linux checks, exact MSRV CI, and Arya's acceptance remain open. Later list/tree/diff/graph/conflict batches remain in the plan.

## Follow-up: WASM motion clock

Arya's Accordion preview check exposed another browser-only failure: `std::time::Instant::now()` panics on `wasm32-unknown-unknown`. The shared transition helper now uses `web_time::Instant`, matching GPUI's clock implementation. Consumer Cargo setup includes `web-time = "1.1"`. Native motion tests and Clippy, installed-source compilation, release WASM, and web build/lint pass. In the browser, Accordion opens and closes, Switch toggles, and Navigation Menu opens with its disclosure icon. Nonfatal zero-size SVG messages appeared during the initial Accordion layout; the controls and icons rendered correctly.
