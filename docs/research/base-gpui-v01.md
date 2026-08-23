# Base GPUI audit for the `imajha/ui` v0.1 slice

Research ticket: [Audit Base GPUI against the v0.1 slice](https://github.com/devaryakjha/ui/issues/10)

## Verdict

Base GPUI is the right behavior layer for the first visual port, but not a complete foundation for production Dialog parity yet.

| v0.1 surface | What Base GPUI already supplies | What `imajha/ui` must supply | Verdict |
| --- | --- | --- | --- |
| Button | Role, focus, Space/Enter activation, disabled guard, children, state-aware styling | shadcn default visuals, variants/sizes, theme tokens, stable IDs | Ready for the v0.1 slice |
| Checkbox | Controlled/uncontrolled checked state, indeterminate state, Space/click behavior, indicator presence, role/toggled semantics, state-aware styling | shadcn visuals, Check/Minus icons, theme tokens, stable IDs | Ready for the v0.1 slice, with documented semantic limits |
| Dialog | Compound parts, controlled/uncontrolled open state, portal/backdrop, Escape/outside/close dismissal, focus restore, state-aware styling | shadcn visuals, X icon, theme tokens, stable IDs; upstream behavior work for a complete focus trap and accessibility links | Fine for a narrow proof; not ready to claim full Dialog parity |
| Icon | Only component-specific icon *slots* such as `SelectIcon`; no general icon renderer or icon catalog | `gpui-icons` plus a small GPUI icon element/API | Missing, as expected |
| Browser preview | A real GPUI-on-WASM/WebGPU showcase, one shared app selected by `?demo=...`, embedded in component pages | reuse/adapt the build and fallback pattern; style demos with `imajha/ui` | Proven feasible on supported desktop browsers |

The intended dependency flow is therefore:

```text
base-gpui (behavior, state, semantics)
        + gpui-icons (glyph data + renderer)
        + imajha/ui source (shadcn visual identity + theme)
        -> real GPUI component
        -> same component compiled into the WASM docs preview
```

## Revisions inspected

- Base GPUI: [`64b22337b6a790c636aab248e768e4875bb28ba8`](https://github.com/LukeTandjung/base-gpui/tree/64b22337b6a790c636aab248e768e4875bb28ba8), the tip of `main` inspected on 2026-08-24.
- GPUI: [`59b2ebf10351b5c0b5cd4403f01ed0460eeec06d`](https://github.com/zed-industries/zed/tree/59b2ebf10351b5c0b5cd4403f01ed0460eeec06d), the exact revision in Base GPUI's manifest and WASM showcase ([root manifest](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/Cargo.toml#L14-L25), [showcase manifest](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/site/Cargo.toml#L13-L16)).
- Base GPUI is MIT-licensed, pre-1.0, and `publish = false`; consumers currently need a Git dependency ([manifest](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/Cargo.toml#L1-L16), [license](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/LICENSE)).

There is revision drift in Base GPUI's own install text: the manifest pins `59b2…`, while the README shows `1d029…` and the generated quick-start source shows `1764…` ([README](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/README.md#L18-L27), [site template](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/scripts/site-template.mjs#L517-L532)). `imajha/ui` must use one tested Base/GPUI revision pair and emit that pair through its registry metadata; it should not copy either upstream documentation example.

## Observed architecture and styling seam

Base GPUI is intentionally headless. Its public model is compound parts, controlled/uncontrolled state, GPUI key actions, and `style_with_state`, not a React API translation ([README](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/README.md#L13-L16), [architecture](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/docs/component-architecture.md#L1-L18)). Components implement GPUI's `Styled` and usually `ParentElement`, so callers can apply normal GPUI styles and children directly. Behavioral flags are exposed through component-specific style-state structs and passed into `style_with_state` ([architecture](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/docs/component-architecture.md#L180-L194)).

This is the correct seam for a visual port. `imajha/ui` should compose and style Base parts; it should not fork their state machines or imitate React props. The Base showcase itself proves this separation: its colors live in a site-local `theme.rs`, not in `base-gpui` ([showcase theme](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/site/src/theme.rs#L1-L69)).

Typed child enums preserve context wiring for known compound parts, while `AnyElement` escape hatches remain visually composable but do not receive component-specific wiring ([Dialog child wiring](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/dialog/child_wiring.rs#L151-L202)). This matters most for Dialog focus, described below.

Stable IDs are part of the behavior contract. Button focus, Checkbox runtime, and Dialog runtime use `Window::use_keyed_state`; Base defaults use fixed IDs, while GPUI requires keys to be unique within the current element namespace ([Button keyed focus](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/button/layers/button_root.rs#L157-L164), [Checkbox context](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/checkbox/context.rs#L27-L48), [Dialog context](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/dialog/context.rs#L31-L59), [GPUI keyed state](https://github.com/zed-industries/zed/blob/59b2ebf10351b5c0b5cd4403f01ed0460eeec06d/crates/gpui/src/window.rs#L3702-L3754)). `imajha/ui` needs a clear stable-ID rule; silently relying on Base's defaults is unsafe for normal screens with several buttons or checkboxes.

## Button

### Observed

- `ButtonRoot` accepts arbitrary children and direct GPUI styling. `style_with_state` receives only `disabled` and `focused`, which is enough to render shadcn button states ([implementation](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/button/layers/button_root.rs#L52-L69), [style state](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/button/style_state.rs#L1-L13)).
- It supplies `Role::Button`, an optional literal accessible label, a tracked focus handle, Space/Enter actions, pointer activation, and one shared disabled guard ([render path](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/button/layers/button_root.rs#L58-L105), [key bindings](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/button/actions.rs#L1-L14)).
- The source deliberately omits disabled accessibility state because the pinned GPUI API has no `aria_disabled` builder ([Button note](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/button/layers/button_root.rs#L74-L87)). The pinned GPUI surface exposes roles, labels, expanded/toggled states, and explicit accessibility actions, but no disabled/required/read-only builders in that trait section ([GPUI source](https://github.com/zed-industries/zed/blob/59b2ebf10351b5c0b5cd4403f01ed0460eeec06d/crates/gpui/src/elements/div.rs#L1244-L1457)).

### Gap `imajha/ui` must fill

All visual behavior: semantic color tokens, typography, radius, spacing, hover/pressed/focus-visible treatment, disabled opacity/cursor, and the pinned shadcn default variants and sizes. No new behavior abstraction is needed for v0.1. Disabled state will remain visually clear but not announced as disabled until GPUI/Base gains that semantic.

## Checkbox

### Observed

- Base supports uncontrolled `default_checked`, controlled `checked`, indeterminate, disabled, read-only, required, cancelable change callbacks, field/fieldset/group integration, and a conditionally mounted `CheckboxIndicator` ([root](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/checkbox/layers/checkbox_root.rs#L23-L74), [context transitions](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/checkbox/context.rs#L60-L139), [indicator](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/checkbox/layers/checkbox_indicator.rs#L50-L91)).
- Space toggles; Enter does not. Pointer and keyboard paths share the state transition, and disabled/read-only states stop the transition ([key binding](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/checkbox/actions.rs#L1-L13), [root interaction](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/checkbox/layers/checkbox_root.rs#L200-L243), [runtime guard](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/checkbox/runtime.rs#L84-L106)).
- It exposes checked, unchecked, disabled, read-only, required, indeterminate, focused, and indicator-present state for styling ([style state](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/checkbox/style_state.rs#L1-L51)).
- Accessibility gets `Role::CheckBox`, a literal label, and `Toggled::True`, `False`, or `Mixed` ([root semantics](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/checkbox/layers/checkbox_root.rs#L192-L212)). Disabled, read-only, and required are available to styling/form metadata but not added to the accessibility node because the pinned GPUI surface lacks those setters.

### Gap `imajha/ui` must fill

The default shadcn box, border, focus ring, disabled treatment, and indicator layout. Checked and indeterminate visuals need real `Check` and `Minus` icons from `gpui-icons`; Base intentionally accepts arbitrary indicator children and supplies no glyph. The same upstream semantic limit as Button must be documented.

## Dialog

### Observed strengths

- The compound anatomy already matches the visual composition needed by shadcn: Root, Trigger, Portal, Backdrop, Viewport, Popup, Title, Description, and Close ([exports](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/dialog/mod.rs#L35-L55)).
- Root supports controlled/uncontrolled open state, modal/non-modal/trap-focus modes, pointer-dismissal control, callbacks, a handle, and state-aware styling ([root render](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/dialog/layers/dialog_root.rs#L69-L124)).
- Trigger supplies Button role, expanded state, literal label, focus, pointer/keyboard/assistive-action activation, and a disabled guard ([trigger](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/dialog/layers/dialog_trigger.rs#L107-L210)).
- Portal renders a viewport-sized deferred anchored layer; Backdrop occludes underlying pointer input and can close on outside press; Popup handles Escape and Tab/Shift-Tab; close returns focus to the active trigger or prior focus ([portal](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/dialog/layers/dialog_portal.rs#L44-L83), [backdrop](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/dialog/layers/dialog_backdrop.rs#L44-L90), [popup](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/dialog/layers/dialog_popup.rs#L120-L163), [focus restore](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/dialog/context.rs#L212-L245)).

### Verified gaps

1. **The modal focus trap does not include arbitrary interactive popup children.** Dialog wiring registers the Popup's focus handle and `DialogClose` handles, but `DialogPopupChild::Any` passes through without registration ([popup registration](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/dialog/layers/dialog_popup.rs#L167-L191), [close registration](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/dialog/layers/dialog_close.rs#L142-L163), [Any passthrough](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/dialog/child_wiring.rs#L260-L290)). The trap then cycles only that registered vector ([runtime](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/dialog/runtime.rs#L457-L489)). A text input or normal Button placed in a Dialog through `child_any` will not join the trap. This needs a Base GPUI fix or an explicit focus-registration API before `imajha/ui` claims a general accessible Dialog.
2. **Accessible relationships and modal inertness are missing upstream.** Base documents the absent `aria-haspopup`, `aria-controls`, `aria-labelledby`, `aria-describedby`, disabled semantics, `aria-modal`, and outside-content inertness. Popup falls back to a literal `aria_label` and kept-mounted closed content is role-less ([documented gaps](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/dialog/mod.rs#L3-L21), [popup semantics](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/dialog/layers/dialog_popup.rs#L120-L128)). Styling cannot repair these.
3. **Modal mode does not itself control pointer blocking.** `DialogModalMode::blocks_pointer()` says only `Modal` blocks, but `DialogBackdrop` always starts from `div().occlude()` and never reads the mode ([mode](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/dialog/style_state.rs#L5-L21), [backdrop default](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/dialog/layers/dialog_backdrop.rs#L26-L35)). The consumer's composition decides whether a backdrop blocks input. `imajha/ui` can compose the correct backdrop for its default modal Dialog, but should not advertise NonModal parity without a behavior check.
4. **Nested-dialog state is present but not wired.** `nested` and `nested_dialog_count` are initialized and exposed to style state, but no Dialog code mutates them ([runtime fields/defaults](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/dialog/runtime.rs#L216-L257)). Nested dialogs should stay out of v0.1 claims.

### Gap `imajha/ui` must fill

The visual layer is straightforward: overlay color, centered content surface, width/padding/radius/shadow, title/description text styles, close-button placement, focus states, and the X icon. The v0.1 preview can prove open/close, outside click, Escape, focus return, and the default visual composition. It should not claim complete Dialog accessibility until the arbitrary-child focus trap and GPUI relationship APIs are resolved.

## Icon audit

There is no public generic `Icon` module or type in this Base GPUI revision ([crate exports](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/lib.rs#L1-L39)). Types named `SelectIcon`, `ComboboxIcon`, and `NavigationMenuIcon` are state-aware container slots; they render caller-provided children, and the first two fall back to the text glyph `⌄` ([SelectIcon](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/select/layers/select_icon.rs#L10-L80), [NavigationMenuIcon](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/src/navigation_menu/layers/navigation_menu_icon.rs#L14-L63)).

That confirms the separate `gpui-icons` effort. For this v0.1 slice, `imajha/ui` only needs the renderer/API contract and the exact Lucide glyphs used by Button/Checkbox/Dialog previews—at minimum Check, Minus, and X. Do not turn `base-gpui`'s structural `*Icon` parts into the icon library.

## Interactive WASM previews

This is already proven in first-party source; no JavaScript imitation is needed.

- The showcase calls `gpui_platform::web_init()`, runs the real app with `run_embedded`, calls `base_gpui::init`, and selects a demo from `?demo=<slug>` ([entrypoint](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/site/src/main.rs#L7-L58)).
- Component pages embed that one shared WASM app in an iframe keyed by slug ([docs generator](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/scripts/build-site.mjs#L180-L195), [iframe template](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/scripts/site-template.mjs#L418-L443)).
- The build requires nightly Rust, rebuilt std, wasm atomics/shared memory, Trunk, and `wasm-bindgen = 0.2.120`; the project disables `wasm-opt` because it breaks shared-memory builds ([WASM config](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/site/.cargo/config.toml#L1-L22), [manifest constraints](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/site/Cargo.toml#L18-L34), [Pages workflow](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/.github/workflows/pages.yml#L17-L52)).
- The current bundle is about 18 MB and needs WebGPU. The docs code already handles missing adapters, mobile limits, boot errors, and worker cleanup ([fallback UI](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/scripts/site-template.mjs#L418-L443), [runtime fallbacks](https://github.com/LukeTandjung/base-gpui/blob/64b22337b6a790c636aab248e768e4875bb28ba8/site/site.js#L8-L82)).

Recommendation: reuse this architecture, not necessarily its site generator. Build one WASM showcase binary with a slug registry, embed the real `imajha/ui` demos, and show an honest fallback on unsupported browsers. Compiling a separate WASM app per component would multiply download and browser memory costs for no gain.

## Required v0.1 work, in dependency order

1. Pin one tested tuple: Base GPUI commit, GPUI commit, `gpui_platform` commit, and WASM glue version.
2. Define the semantic shadcn theme/token contract and light/dark defaults. Base supplies no reusable theme.
3. Define the stable-ID contract for copied components.
4. Land the minimal `gpui-icons` renderer plus Check, Minus, and X under their upstream license and attribution terms.
5. Build styled Button and Checkbox wrappers over Base parts; do not copy their behavior.
6. Build the narrow default Dialog composition and preview, but mark the focus-trap and accessibility relationship gaps as upstream blockers for full parity.
7. Adapt Base's single-bundle WASM showcase pattern for real interactive component-page previews.

## Validation boundary

This was a source audit of the pinned revisions. I did not mutate either repository or GitHub, and I did not run Base GPUI's test suite or build its WASM site during this ticket. The report distinguishes source-backed behavior from recommendations; browser support claims above come from the pinned showcase's own build and fallback code, not a fresh cross-browser run.
