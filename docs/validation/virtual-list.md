# Virtual List validation

Local macOS checks on 2026-09-08. This is the fixed-height T05/T06 slice, not full desktop-list or performance qualification. No release or deployment was run.

## Observed behavior

- The native showcase renders 100,000 rows using retained application metadata. Clicking a row and Shift+Down select a range.
- Right-clicking a selected row preserves the range. Clicking Inspect selection reports the original stable row identity and both selected rows. This check caught and fixed clicks reaching rows beneath the context-menu portal; a showcase regression test covers it.
- Refresh adds a row while retaining both selected identities. Reversing the order and switching every row from 32 to 48 pixels retain selection. Jump to 50,000 reveals the requested identity after reversal.
- Native accessibility inspection exposed named list/options, selected states, disabled-row descriptions and named showcase controls. Full VoiceOver navigation has not been qualified.
- The automated 100,000-row check builds fewer than 40 rows in a 480×320 viewport, exercises pointer and keyboard navigation, skips disabled rows, activates a stable ID, and reveals the final row. This is a virtualization check, not frame-time evidence.
- Metadata checks cover range selection after reorder, disabled and stale IDs, surviving focus/viewport identity, and atomic rejection of duplicate IDs.

## Reproducible checks

```sh
cargo test --workspace --features gpui_platform/runtime_shaders
cargo clippy --workspace --all-targets --features gpui_platform/runtime_shaders -- -D warnings
cargo fmt --all -- --check
cargo run -p gpuicn-cli -- build
node scripts/sync-catalog.mjs
cd web
bun run build
bun run lint
```

These passed, as did the release native showcase and WASM builds. The runtime-shaders feature avoids this machine's missing Xcode Metal compiler. Existing upstream future-compatibility warnings remain.

A fresh temporary Rust project installed `button virtual-list` through the native CLI from the generated local registry. Its locked Cargo check passed with `gpui_platform/runtime_shaders`; the list, shared theme, scrollbar and tooltip sources compiled outside this workspace.

## Open gates

The native resize/scroll benchmark did not finish a fresh sample in bounded attempts. Prior Sidebar measurements do not qualify this list; no frame-time, peak-memory or 60/120 Hz claim is made here. The desktop fixture now uses the actual VirtualList for a future measurement run. Budgets still need agreement.

Variable-height rows remain unimplemented. The pinned GPUI variable list discards offscreen height estimates and rebuilds its height tree on width changes; qualify that behavior before using it for wrapped diff content. Windows, Linux and full assistive-technology behavior remain unverified.
