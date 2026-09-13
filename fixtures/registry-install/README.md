# Source-registry consumer review

The default binary remains the installation starter. `readiness` is a small
review app built from the same copied sources and pinned dependencies.

From the repository root, create the fixture config once:

```sh
cargo run -p gpuicn-cli --locked -j 1 -- --config fixtures/registry-install/gpuicn.toml --registry ../../site/pages/r init
```

Then refresh the copied fixture sources and run the review app. The refresh
replaces local edits in this review fixture:

```sh
cargo run -p gpuicn-cli --locked -j 1 -- build
node scripts/sync-catalog.mjs
cargo run -p gpuicn-cli --locked -j 1 -- --config fixtures/registry-install/gpuicn.toml add --all --overwrite
cargo run --manifest-path fixtures/registry-install/Cargo.toml --locked -j 1 --bin readiness --features gpui_platform/runtime_shaders
```

Review Settings, Records, and Workspace with keyboard and pointer input. Use
the header controls for light/dark themes, a custom palette, compact spacing,
larger text, long content and disabled settings fields. Resize the window below
768px to review the mobile sidebar. Settings and record edits are kept in memory
for the current review session. The catalog sync copies the bundled Geist fonts;
`init` only creates the installer config. Skip `init` if that config already exists.

This provides a review surface for the current local registry branch. The
public `v0.5.0-beta.3` registry does not contain the readiness fixes until a new
release is published.
