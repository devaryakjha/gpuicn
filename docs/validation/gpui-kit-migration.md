# GPUI Kit migration qualification — 2026-09-09

This migration is on `codex/gpui-kit-migration`. The current gpuicn public beta
remains unchanged. The icon dependency has been released separately.

## Changes

- GPUI Kit and Longbridge `gpui-base`: 0.6.1.
- GPUI runtime/platform family: `gpui-pre` 0.3.4, Zed snapshot
  `69164008341295ad481bb11c0334a712ca8c23e3`.
- Native toolchain: Rust 1.97.1. WASM: nightly-2026-08-17,
  wasm-bindgen 0.2.121, Trunk 0.21.14.
- Showcase and installation starter use Kit's application entry point.
- Theme initialization installs Longbridge's base layer once alongside the
  retained Base GPUI controls. Theme transitions delegate to upstream motion;
  gpuicn retains durations, easing, and its reduced-motion preference.
- `crates/gpui-compat` re-exports the same runtime for LukeTandjung's retained
  `base-gpui`, whose manifest still names the old GPUI Git package. It contains
  no separate runtime or component behavior. Applications must apply the Cargo
  patch in their root manifest; dependency patches do not propagate.
- [`gpui-icons` 0.4.0](https://github.com/devaryakjha/gpui-icons/releases/tag/v0.4.0) is released: Lucide 1.43.0,
  1,818 canonical icons, 259 aliases, GPUI 0.3.4, and manifest schema 3.
  All previous canonical names and aliases still resolve. `Trash2` remains a
  compatibility constant for `Trash`. SVGs stay embedded on native and WASM.
- Registry JSON, installation examples, catalog counts, and current dependency
  guidance follow the migration.

GPUI 0.3.1 passed debug checking but failed an optimized build: its macro crate
called an inspector helper that was compiled out. Version 0.3.4 fixes that
conditional-compilation error and satisfies Kit 0.6.1's dependency range.
No inspector workaround or patch to cached upstream source was added.

## Deliberately retained behavior

The input engine remains unchanged. Longbridge 0.6.1 constructs its own focus
handle inside `InputBaseState::new_in_mode` and registers listeners against it;
the public API does not accept a caller-supplied handle. gpuicn's public
`Input::focus_handle`, NumberField, Toolbar, and task editing rely on that
contract. Replacing it now would require upstream API work or a behavioral
change. Native and WASM checks below exercise the retained editor on the new
runtime; they do not claim that the editor itself migrated.

Forms, dialog focus, keyboard-accessible splitters, and stable list selection
also retain their existing contracts. Before adding tree, editor, Markdown, or
docking code, evaluate Kit's existing components against the required behavior.

## Evidence

| Check | Result |
| --- | --- |
| Workspace tests on GPUI 0.3.4 | 44 passed |
| Workspace Clippy, all targets, warnings denied | Passed |
| Rust formatting and diff whitespace | Passed |
| Independent registry fixture, all installed components | Passed |
| New temporary app, all components, fresh lockfile | Passed again with immutable remote pins and no copied lockfile |
| Optimized WASM build with pinned tools | Passed |
| Web build, TypeScript, ESLint, highlighting and preview-message checks | Passed |
| Icon crate test | All 1,818 SVGs rasterize; bytes, metadata hashes, aliases and license match |
| Icon crate Clippy, formatting and whitespace | Passed |
| Native app with isolated temporary data | Rendered; task creation, automatic input focus, typing, undo and redo worked |
| Actual WASM input preview | Typing, undo/redo, read-only field and keyboard traversal worked |
| Actual icon catalog | Bridge rendered at 16/24/32/48px; `trash-2` search found Trash |

The native test used a separate app bundle and `GPUICN_WORKSPACE_DATA` under
`/tmp/gpuicn-kit-review`; it did not use the user's saved workspace. Computer-use
inspection was initially slow and pointer automation later lost its window
mapping; accessibility and keyboard actions verified task entry. This is not a
complete manual interaction audit. OS IME composition, Windows/Linux runtime,
and performance budgets have not been qualified in this pass.

## Release status

- `gpui-icons` 0.4.0 passed GitHub CI and is published at commit
  `01ac07dd83e97f9d6a3526466413732fbdfc2975`.
- The workspace and installation fixture use that immutable icon pin.
- The installation fixture and guide pin the compatibility package to gpuicn
  commit `64d1bd9678d365c2f1367431c700c36184fb5142`. Consumers do not need a
  separate local compatibility checkout. Keep the root Cargo patch until the
  retained base dependency adopts the shared package directly.
- [PR #55](https://github.com/devaryakjha/gpuicn/pull/55) tracks clean remote
  native and WASM CI for this migration. No new gpuicn release or production
  deployment has been made.

## Review disposition

Standards review identified current toolchain/catalog references and the missing
migration note; those are updated. Historical validation notes keep their
original dates and toolchains. The upstream motion implementation owns reversal
testing; the local check verifies the theme's reduced-motion adapter. Platform
helpers in small examples remain valid because they resolve to the same runtime.

Spec review identified the unpublished icon path and the changed manifest
contract. The icon dependency now has an immutable release pin; the manifest
has schema 3 and generator version 0.3.0. Future Lucide renames require API
review during the next update; no speculative compatibility framework was added.
