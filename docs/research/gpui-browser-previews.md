# Assess real GPUI browser previews

Research snapshot: 2026-08-24. Base GPUI source inspected at [`64b22337`](https://github.com/LukeTandjung/base-gpui/commit/64b22337b6a790c636aab248e768e4875bb28ba8); Zed/GPUI `main` checked at [`d9ad6aff`](https://github.com/zed-industries/zed/commit/d9ad6aff67e47de43abb270d22de75dd950f1b48). Base GPUI pins GPUI to [`59b2ebf1`](https://github.com/zed-industries/zed/commit/59b2ebf10351b5c0b5cd4403f01ed0460eeec06d).

## Verdict

**Real, interactive GPUI previews in a browser are supported today and are credible for `ui`.** This is no longer a speculative GPUI feature: Base GPUI already builds its actual Button, Checkbox, Dialog, Menu, and other components to `wasm32-unknown-unknown`, embeds one selected demo in each docs page, and deploys the result to GitHub Pages. See its [Pages build](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/.github/workflows/pages.yml), [WASM showcase entry point](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/site/src/main.rs), and live [Button](https://luketandjung.github.io/base-gpui/components/button.html), [Checkbox](https://luketandjung.github.io/base-gpui/components/checkbox.html), and [Dialog](https://luketandjung.github.io/base-gpui/components/dialog.html) pages.

The right v0.1 decision is therefore:

- require the real GPUI component for behavior-bearing previews;
- copy Base GPUI's existing showcase shape instead of inventing another renderer;
- support desktop WebGPU first and show an honest fallback elsewhere;
- treat browser screen-reader parity and broad mobile support as separate work, not as proven by the current canvas demo.

## Verified facts

### GPUI browser platform and renderer

- Browser support is real but still early: `gpui_web` is version `0.1.0`, while GPUI itself remains pre-1.0, and an open upstream issue still reports a runtime closure failure in the official web example. Treat the browser platform as an experimental preview target, not desktop parity. [`gpui_web` manifest](https://github.com/zed-industries/zed/blob/d9ad6aff67e47de43abb270d22de75dd950f1b48/crates/gpui_web/Cargo.toml) · [GPUI README](https://github.com/zed-industries/zed/blob/d9ad6aff67e47de43abb270d22de75dd950f1b48/crates/gpui/README.md) · [upstream issue](https://github.com/zed-industries/zed/issues/59582)
- GPUI has a first-party `gpui_web` platform for `target_family = "wasm"`. It owns one document canvas and supports one top-level GPUI window. A second window, reopening a closed window, and native anchored popup, floating, popup, or dialog windows are rejected. [Pinned module contract](https://github.com/zed-industries/zed/blob/59b2ebf10351b5c0b5cd4403f01ed0460eeec06d/crates/gpui_web/src/gpui_web.rs) · [Pinned platform implementation](https://github.com/zed-industries/zed/blob/59b2ebf10351b5c0b5cd4403f01ed0460eeec06d/crates/gpui_web/src/platform.rs)
- Rendering uses GPUI's wgpu renderer. `Auto` tries browser WebGPU, then WebGL2; callers can force either backend. [Pinned platform initialization](https://github.com/zed-industries/zed/blob/59b2ebf10351b5c0b5cd4403f01ed0460eeec06d/crates/gpui_web/src/platform.rs) · [Current `gpui_wgpu` backend selection](https://github.com/zed-industries/zed/blob/d9ad6aff67e47de43abb270d22de75dd950f1b48/crates/gpui_wgpu/src/wgpu_context.rs)
- Zed includes a first-party interactive browser example using `gpui_platform::web_init()`, a web platform application, and an ordinary GPUI window. It targets `wasm32-unknown-unknown` through Trunk. [Example app](https://github.com/zed-industries/zed/blob/d9ad6aff67e47de43abb270d22de75dd950f1b48/crates/gpui_web/examples/hello_web/main.rs) · [HTML host](https://github.com/zed-industries/zed/blob/d9ad6aff67e47de43abb270d22de75dd950f1b48/crates/gpui_web/examples/hello_web/index.html)

### Toolchain and hosting constraints

- The official browser example uses nightly Rust, `rust-src`, `wasm32-unknown-unknown`, `build-std`, atomics, and shared memory. [Toolchain pin](https://github.com/zed-industries/zed/blob/d9ad6aff67e47de43abb270d22de75dd950f1b48/crates/gpui_web/examples/hello_web/rust-toolchain.toml) · [WASM compiler flags](https://github.com/zed-industries/zed/blob/d9ad6aff67e47de43abb270d22de75dd950f1b48/crates/gpui_web/examples/hello_web/.cargo/config.toml) · [Cargo `build-std` documentation](https://doc.rust-lang.org/cargo/reference/unstable.html#build-std)
- Base GPUI copies that threaded setup. Its own config says the single-threaded fallback is broken at the pinned GPUI revision, so its working site uses the threaded build. [Base GPUI WASM config](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/site/.cargo/config.toml)
- Shared-memory WASM needs a cross-origin-isolated page. Base GPUI sends COOP/COEP headers in local Trunk serving and uses a service worker to inject them on GitHub Pages. Cross-origin resources must cooperate, so Base GPUI self-hosts its fonts. [Trunk config](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/site/Trunk.toml) · [service worker](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/site/coi-serviceworker.js) · [site generator deployment notes](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/scripts/build-site.mjs)
- Base GPUI pins `wasm-bindgen = 0.2.120`; its manifest records a runtime closure regression with newer glue at the pinned GPUI revision. It also disables `wasm-opt` because it breaks this shared-memory build. [Showcase manifest](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/site/Cargo.toml) · [Trunk HTML](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/site/index.html)
- Base GPUI reports roughly 18 MB of WASM per demo bundle. Each iframe owns the compiled module, worker threads, and a GPU device; its docs unload the iframe on page hide to avoid exhausting browser executable memory. It skips auto-loading on phones and asks users to opt in. [Demo shell](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/scripts/site-template.mjs) · [lifecycle and mobile guards](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/site/site.js)
- Although GPUI itself has WebGL2 fallback code, Base GPUI's deployed docs currently gate live previews on a usable WebGPU adapter. The proven support target is therefore desktop WebGPU, not every browser with WebGL2. [Base GPUI demo shell](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/scripts/site-template.mjs) · [client capability check](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/site/site.js)

### Base GPUI compatibility

- Base GPUI explicitly accounts for WASM time by using `web-time`; its current Pages workflow builds the showcase with its pinned GPUI revision. [Root manifest](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/Cargo.toml) · [Pages workflow](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/.github/workflows/pages.yml)
- The showcase runs the same Base GPUI crate used natively. A `?demo=<slug>` query selects one real component renderer from a shared binary; there is no JavaScript imitation of component behavior. [Showcase entry point](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/site/src/main.rs) · [demo registry](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/site/src/demos/mod.rs)
- Real Button, Checkbox, and Dialog demos already compile under this arrangement. [Button demo](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/site/src/demos/button.rs) · [Checkbox demo](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/site/src/demos/checkbox.rs) · [Dialog demo](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/site/src/demos/dialog.rs)

### Input, focus, and overlays

- The pinned browser platform maps DOM pointer down/up/move/leave, wheel, context menu, keyboard down/up, paste, IME composition, focus, and blur events into GPUI input. A hidden HTML input owns keyboard and composition focus while pointer events come from the canvas. [Pinned event bridge](https://github.com/zed-industries/zed/blob/59b2ebf10351b5c0b5cd4403f01ed0460eeec06d/crates/gpui_web/src/events.rs) · [Pinned window host](https://github.com/zed-industries/zed/blob/59b2ebf10351b5c0b5cd4403f01ed0460eeec06d/crates/gpui_web/src/window.rs)
- GPUI's current web platform handles IME composition text but leaves `update_ime_position` empty, so it cannot anchor the browser's IME candidate window to the drawn caret. This does not affect Button, Checkbox, or Dialog, but it matters for later text-input previews. [Composition bridge](https://github.com/zed-industries/zed/blob/d9ad6aff67e47de43abb270d22de75dd950f1b48/crates/gpui_web/src/events.rs#L553-L584) · [empty positioning hook](https://github.com/zed-industries/zed/blob/d9ad6aff67e47de43abb270d22de75dd950f1b48/crates/gpui_web/src/window.rs#L809-L817)
- Component overlays do not require forbidden browser-level windows. Base GPUI's `DialogPortal`, backdrop, viewport, and popup stay inside the one GPUI window/canvas, and its live Dialog preview proves this path renders and receives input. [Dialog demo](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/site/src/demos/dialog.rs) · [live Dialog page](https://luketandjung.github.io/base-gpui/components/dialog.html)
- Dialog implements Escape close and Tab/Shift-Tab routing through GPUI actions; modal mode can trap focus and the runtime retains trigger details for focus return. [Dialog popup behavior](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/dialog/layers/dialog_popup.rs) · [Dialog runtime](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/dialog/runtime.rs)
- Browser-level file dialogs, credentials, native popup windows, and synchronous clipboard reads are unsupported or limited in `gpui_web`. None is needed for Button, Checkbox, or Dialog, but later components must be checked against the platform implementation. [Pinned platform implementation](https://github.com/zed-industries/zed/blob/59b2ebf10351b5c0b5cd4403f01ed0460eeec06d/crates/gpui_web/src/platform.rs)
- The web platform does not load system fonts; GPUI currently bundles IBM Plex Sans and Lilex into its web platform. A shadcn visual port must explicitly load and test its chosen fonts instead of assuming browser or OS font access. [Current font setup](https://github.com/zed-industries/zed/blob/d9ad6aff67e47de43abb270d22de75dd950f1b48/crates/gpui_web/src/platform.rs#L26-L35) · [current text system](https://github.com/zed-industries/zed/blob/d9ad6aff67e47de43abb270d22de75dd950f1b48/crates/gpui_web/src/platform.rs#L135-L144)

### Accessibility boundary

- Base GPUI components create GPUI accessibility roles, labels, and state where GPUI exposes the primitives. For example, the Button and Dialog guides call out their accessible-name and role behavior. [Button guide](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/docs/components/button.md) · [Dialog guide](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/docs/components/dialog.md)
- **Not verified:** current `gpui_web` source at `d9ad6aff` has no browser accessibility bridge module and renders the interface to a canvas plus hidden input. I found no first-party evidence that GPUI's internal accessibility tree reaches browser assistive technology. Do not claim screen-reader or browser accessibility-tree parity for these previews. [Current `gpui_web` tree](https://github.com/zed-industries/zed/tree/d9ad6aff67e47de43abb270d22de75dd950f1b48/crates/gpui_web/src) · [current window host](https://github.com/zed-industries/zed/blob/d9ad6aff67e47de43abb270d22de75dd950f1b48/crates/gpui_web/src/window.rs)

## Smallest credible proof for `ui`

Do not start with a generic docs engine. Port the smallest existing Base GPUI showcase shape:

1. Add one WASM showcase binary pinned to the same Base GPUI and GPUI revisions as the native library.
2. Register exactly three render functions selected by `?demo=button`, `?demo=checkbox`, or `?demo=dialog`.
3. Apply the shadcn default visual tokens in those render functions; use the actual styled `ui` components, not parallel demo-only copies.
4. Embed one lazy iframe on each component page. Load the shared bundle, not one bundle per component.
5. Serve COOP/COEP headers locally. If GitHub Pages remains the host, copy Base GPUI's narrow service-worker solution.
6. For v0.1, advertise desktop WebGPU support and show a clear native-run fallback on unsupported/mobile browsers. Investigate WebGL2 only after the three-component proof works.

### Acceptance checks

- **Button:** mouse click works; hover and focus styles appear; Enter and Space activate it.
- **Checkbox:** mouse and Space toggle the real component; checked and focus states repaint.
- **Dialog:** trigger opens the in-canvas overlay; Escape closes it; Tab and Shift-Tab stay within the modal; outside press follows the configured dismissal rule; close returns focus to the trigger.
- **Hosting:** first visit reaches an isolated context and renders; a hard refresh still works; leaving and returning to a docs page does not keep old iframe workers alive.
- **Failure UI:** missing WebGPU, null adapter, boot errors, and phone access get explicit states instead of a blank canvas.

Passing those checks proves all hard v0.1 layers: the actual styled crate, Base GPUI state, GPUI input/focus, in-canvas overlays, the WASM renderer, and docs embedding.

## Inferences and cautions

- Because Base GPUI already runs these exact component classes in-browser, a new renderer or JavaScript mirror would add risk without proving more.
- One shared showcase bundle is the least costly starting point. Loading many live iframes on one long catalog page would multiply workers, GPU devices, and memory; render one preview per component page and lazy-load it.
- The existing Base GPUI site proves current source and deployment compatibility, but I did not rebuild it locally: this machine lacks the WASM target and Trunk. The deployed pages and GitHub Pages workflow are the runtime/build evidence.
- GPUI and Base GPUI are pre-1.0 and pinned together. Treat the GPUI revision, `wasm-bindgen` pin, and WASM config as one tested set; upgrade them through a dedicated compatibility issue.

## Decision

**Adopt real interactive GPUI/WASM previews for behavior-bearing components.** Make the three-component proof the first implementation gate, based on Base GPUI's existing showcase. Do not promise browser accessibility parity, broad mobile support, or every graphics backend in v0.1.
