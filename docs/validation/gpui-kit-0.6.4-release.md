# GPUI Kit 0.6.4 release audit

This audit compares GPUI Kit 0.6.1 with 0.6.4 against gpuicn's public source
registry. It separates changes inherited through `gpui-base` from changes in
GPUI Kit's styled component layer. A version bump alone does not transfer a
styled component fix to a gpuicn wrapper that renders its own surface.

## Required for v0.5.0-beta.4

| Work | Why | Files and acceptance |
| --- | --- | --- |
| Ship Carousel, Empty and Input Group | They are the three public components added in GPUI Kit 0.6.2. Omitting them would make this a dependency refresh rather than a complete component release. | Add registry source, exports, registry metadata, usage examples, parity notes and showcase previews for all three. Carousel covers horizontal and vertical layouts, keyboard navigation, controlled selection, looping, pagination and reduced motion. Input Group covers inline and block addons, buttons and a textarea in one frame. Empty exposes its composable parts. |
| Expose Textarea as its own install | gpuicn already styled `Textarea` in the Input source, but users could not find or install it as a named component. | Add a Textarea registry wrapper, crate module, Usage example, preview, catalog entry and parity note. Make `Textarea` implement `FieldControl` so a labeled multi-line control composes like Input. |
| Enable and initialize the component layer | Carousel's state and gesture engine live behind GPUI Kit's `component` feature and register actions in `gpui_kit::init`. Calling only `gpui_kit::base::init` leaves that engine partly uninitialized. | Set `features = ["component"]` on the pinned production dependency and include `component` beside `test-support` for dev dependencies in the root, documented install snippet and registry fixture. In `registry/theme/theme.rs`, call `gpui_kit::init(cx)` once from the existing guarded initializer instead of calling `gpui_kit::base::init(cx)`. This keeps crate and copied-source startup on one path. |
| Expose paste interception on gpuicn inputs | GPUI Kit added `on_paste` to its styled `Input`, `Textarea` and `Editor`. gpuicn renders `gpui_base::InputBase` directly, so this callback is not inherited. | Add an `on_paste` builder to gpuicn `Input` and `Textarea` in `registry/input/input.rs`. Capture `gpui_kit::base::input::Paste`, read the `ClipboardItem`, stop propagation only when the callback returns `true`, and keep normal text paste when it returns `false`. Document that synchronous image/file interception is native-only because the upstream web clipboard read remains unavailable. |
| Clear Select search state on close | gpuicn's editable Select used the same editor value for the selected label and filter query. Reopening after choosing an item could filter the list to that item. The upstream component fix does not reach the custom state machine. | Keep display text and the active query distinct in `registry/select/select.rs`; closing restores the selected label and reopening starts with an empty query. A focused regression check must choose an item, close, reopen and observe all options. |
| Route dialog controls to their owning dialog | gpuicn's close, alert confirm and alert cancel helpers still dispatch actions through the window. GPUI Kit 0.6.2 changed these controls to dispatch from a node inside the owning dialog, which matters when another surface retains focus. | Update `registry/dialog/dialog.rs::dialog_close` and `registry/alert_dialog/alert_dialog.rs::{alert_dialog_action, alert_dialog_cancel}` to use the upstream anchored dialog control primitives, or the same focus-anchor behavior. `dialog_action`, which closes an explicit caller-owned handle, is already safe. Check nested dialogs and a dialog whose text field has focus. |

## Delta matrix

| GPUI Kit 0.6.2–0.6.4 change | gpuicn disposition | Concrete reason or check |
| --- | --- | --- |
| Carousel | **Add for this release** | New public component; reuse the upstream state and gesture engine and apply gpuicn's theme rather than reimplementing snapping and input handling. |
| Empty | **Add for this release** | New composable presentation component with no separate runtime state. |
| Input Group | **Add for this release** | New composition needed to present addons, actions, single-line input and textarea in one accessible frame. |
| Input/Textarea/Editor `on_paste` | **Wrapper change required** | gpuicn owns Input and Textarea presentation and therefore must forward the new hook. gpuicn has no Editor component, so the Editor hook is outside the current registry surface. |
| Text descender clipping | **Inherited** | gpuicn renders GPUI Base input state directly; glyph layout comes from the upgraded base engine. Check one `gypq` sample in Input and Textarea during visual review. |
| Touch text selection, edit menu and double-tap selection | **Inherited but not qualified** | The behavior is in GPUI Base input state. The release may say the runtime contains it, but gpuicn has not qualified iOS or Android and must not advertise mobile support from this dependency update. |
| Public editor search and runtime language parser factories | **Outside the current surface** | These APIs belong to `EditorState` and the language registry. gpuicn currently ships Input/Textarea controls, not a code editor or language service. No placeholder wrapper is warranted. |
| Markdown plugins, inline math, data URLs, stream fade and TextView rendering fixes | **Outside the current surface** | gpuicn has no Markdown or TextView registry component. These APIs remain available to applications through the pinned GPUI Kit dependency; this release must not claim a styled gpuicn Markdown component. |
| Motion `Sequence` | **Available upstream; no wrapper needed** | gpuicn's motion helpers already use GPUI Base transitions and presence directly. A second sequence abstraction would duplicate the pinned dependency. |
| OS reduced-motion initialization | **Inherited after full init** | `gpui_kit::init` calls Base initialization, which reads the OS preference into `App::reduce_motion`. Carousel and existing gpuicn motion then share the same setting. Check that the guarded theme initializer calls full Kit init exactly once. |
| Component hover, active and focus theme | **Bridged for this release** | Carousel arrows and pagination retain GPUI Component's control behavior. `UiTheme::set` projects the mode, focus ring, shared colors, fonts and radii used by those controls and the upstream carousel root into GPUI Component and refreshes its Base snapshot, including later light/dark or custom theme changes. |
| External iOS/Android hosting and `is_mobile()` | **Not release-qualified** | No gpuicn iOS or Android host, build or interaction pass exists. Keep the release note explicit that responsive sidebar examples are not mobile-platform qualification. |
| Scroll bounce and touch scrollbar behavior | **Base behavior available; not mobile-qualified** | Scroll Area uses GPUI Base's scrollbar, so its thumb fixes transfer. gpuicn does not currently wrap `ScrollBounce`; adding a wrapper without a qualified mobile host would be speculative. |
| Mobile Hover Card tap behavior | **Inherited** | Preview Card reexports and uses `gpui_base::HoverCard`, where the mobile tap behavior was added. Do not advertise it until mobile qualification exists. |
| Mobile tooltip suppression | **Inherited for gpuicn tooltips** | gpuicn's tooltip owns its trigger lifecycle but requests display through `gpui_base::TooltipOverlay`; the upgraded overlay ignores requests on iOS and Android. |
| Dock canvas removal, title-bar opt-out and `select_panel` | **Outside the current surface** | gpuicn does not ship a Dock component and has no dependency on the removed tiles-canvas API. No migration is needed. |
| Ghost button hover/press colors | **Reviewed; no gpuicn change** | The upstream change is in GPUI Component's theme-specific Button. gpuicn deliberately styles GPUI Base Button from the shadcn Nova palette; its neutral `muted` and `accent` tokens resolve to the same surfaces in the pinned light and dark themes. Adding GPUI Component's `button_active` token would mix theme systems. |
| Dialog window bounds | **Already covered by gpuicn composition** | gpuicn mounts a Base dialog in its owning window and constrains the popup through `modal_viewport`. Retain the short-window check; the action-routing row above is the part that still needs transfer. |
| Dialog control routing | **Wrapper change required** | The upstream focus-anchor fix is bypassed by gpuicn helpers that call `window.dispatch_action` themselves. |
| Scrollable menu submenus | **Independent implementation; qualify** | gpuicn does not use `gpui_component::PopupMenu`. Its menu puts every submenu in its own deferred positioned layer and records row bounds after scrolling, which is the same structural fix. Before release, open a submenu from a scrolled root menu in the native showcase and confirm it is visible and clickable. No speculative rewrite is needed. |
| Menu shortcut hints on first frame | **Outside the current API** | gpuicn Menu has no shortcut-hint field. It must not claim this upstream styled-menu feature. Add it only with a public shortcut API and a consumer need. |
| Tab scrolling and `flex_1` fixes | **Outside the current Tabs contract** | gpuicn Tabs is a controlled, fixed tab list built on GPUI Base roles; it does not expose GPUI Component's scrollable TabBar. Existing selection, arrow, Home and End behavior remains the release contract. |
| List/Table selection fixes and performance | **No direct transfer claim** | gpuicn has no GPUI Component List or Table. Virtual List is its own fixed-row, caller-owned selection implementation and already draws selection without the upstream outline. |
| Select query reset | **Wrapper change required** | Custom `SelectState` owns its query and selection; see the required work above. |
| Setting step/clamp and selected-page fixes | **Outside the current surface** | gpuicn has Number Field but no GPUI Component Setting delegate or settings-page component. Number Field continues to use GPUI Base Number Input. |
| Avatar OkLCH fallback | **Reviewed; no gpuicn change** | gpuicn Avatar intentionally uses the shadcn themed muted fallback supplied by the caller. Importing GPUI Component's generated initials color would change the component's visual source. |
| Color Picker accessibility IDs | **Outside the current surface** | No gpuicn Color Picker exists. |
| Chart, Plot, Pie Chart and FPS fixes | **Outside the current surface** | gpuicn ships none of these components. The 0.6.4 Pie Chart radius and leader-line fixes therefore need no wrapper change. |
| Linux tiled-window popup clamp (0.6.4) | **Inherited** | gpuicn Menu and Select use `gpui_base::Positioner`, so their popups receive the per-edge client-inset fix automatically. Include a tiled Linux edge case in the native release check. |
| Upstream website font and docs fixes | **Not product code** | They apply to gpui-kit.com. gpuicn has its own pinned Geist fonts, registry docs and WASM site checks. |

## Release evidence

The release is ready only when the generated registry contains the three new
GPUI Kit components plus the standalone Textarea entry, a clean consumer fixture installs each one, native CI and the WASM
catalog compile from the final commit, and the showcase exercises the required
interaction cases above. Linux startup evidence does not prove carousel gesture
quality, mobile support or broad desktop compatibility; state those limits in
the release notes instead of converting them into product claims.
