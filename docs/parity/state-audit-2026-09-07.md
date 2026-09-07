# Component state and preview audit — 7 September 2026

Scope: all 37 component wrappers, their shared styling and compiled examples. This audit prepares local changes for the next release; it does not publish a release.

## Evidence

- Read the wrappers and traced the affected Base GPUI state and event paths.
- A native smoke test draws all 37 previews in light and dark themes, with the showcase's controlled flags off and on: 148 render cases. This checks rendering, including open controlled dialogs, but does not simulate every interaction or every state combination.
- Native regression checks cover checkbox-group label/box clicks and disabled cascading; switch preview label/control clicks; grouped combobox trigger open/close; button variant cursors; and checkbox cursors.
- Browser checks below use the rebuilt GPUI/WASM preview. A native smoke pass alone is not a browser, accessibility, or visual pass.

Final local checks passed: 17 Rust tests (including 148 preview render cases), workspace Clippy, usage-example check, release WASM build, web production build and lint, formatting and diff checks. The stock shadcn installer accepted all items; installed Rust sources matched the registry and the install fixture compiled. Dependency future-compatibility warnings and unused re-export warnings in the fixture remain.

## Changes

- Shared Button styling now gives enabled controls a pointer cursor and disabled controls a blocked cursor. This also reaches button-based triggers and actions.
- Checkbox, checkbox-group, radio, switch, slider, menu, menubar and navigation links now distinguish their interactive states with the cursor. Disabled menu/select options do not retain an active highlight.
- Checkbox-group labels now live inside the actual checkbox hit target, retain a single accessible name, and obey both item and group disabled state.
- The switch preview no longer toggles twice through a bubbling parent click handler. The label and switch each change its controlled value once.
- Number steppers reflect read-only, disabled, minimum and maximum state instead of appearing actionable when they cannot change the value.
- Select and combobox inputs expose invalid borders/rings; OTP uses the error ring when focused and invalid. Input focus and ToolbarInput alignment/state styling now follow the shared input treatment.
- Avatar images apply their circular radius and cover crop to the GPUI image itself. The Base image node still owns loading/fallback state. The preview uses the supplied pinned image, initials, an overlapping group, and three sizes, following the compact composition of [shadcn's avatar examples](https://ui.shadcn.com/docs/components/base/avatar).
- The Line tab variant clears the default all-sided border before drawing its underline.
- Preview coverage now includes an icon button, avatar images and sizes, a combobox clear control, field error text, labelled number-field modes, both separator directions, visible progress/meter values, a live stepped slider value, both tab variants, and both toggle-group selection modes. Navigation examples now have real destinations.
- Select, autocomplete and combobox keep the popup at least as wide as its anchor; searchable empty states have horizontal padding.
- Combobox trigger presses no longer conflict with the input-group open handler and popup outside-click handler: one click opens, the next closes.
- Toolbar now demonstrates a working text input; OTP rows have explicit editable, disabled and read-only labels.

## Component checklist

All rows received source inspection and the native render smoke check. The browser column records only checks actually performed in this audit, not implied coverage.

| Component | States and example concerns reviewed | Browser evidence |
| --- | --- | --- |
| Accordion | Closed/open, focus, disabled styling, chevron, wrapping | Pointer expands content; Enter toggles focused trigger. Arrow navigation needs a separate native check. |
| Alert dialog | Closed/open, cancel/action, modal focus, backdrop | Opens within the preview and Cancel closes it. |
| Autocomplete | Input focus, filtering, disabled options, empty results | Typing `bu` filters to Button; `zz` shows the empty state. |
| Avatar | Image, loading/error fallback, sizes, circular crop | Supplied image, initials and group render in both themes. |
| Button | Six variants, sizes, icon button, focus, disabled | Six variants and size examples fit; icon button increments the visible counter. |
| Checkbox | Checked/unchecked/mixed, read-only, disabled, label target | Box and label toggle; pointer/blocked cursor verified. |
| Checkbox group | Selection, item/group disabled, label target | Weekly digest label checks the box; native test covers disabled guards. |
| Collapsible | Controlled open/closed, trigger state, content | Folder trigger collapses its children; surrounding rows remain. |
| Combobox | Input/group focus, invalid, read-only/disabled, clear, options | Typing filters, Apple selects, clear restores the placeholder and options; arrow opens on one click and closes on the next. |
| Context menu | Right-click open, item/checkbox/radio selection, disabled | Right-click opens within the frame; bookmark checkbox changes while the menu stays open. |
| Dialog | Open/close, input focus, generic child tab order | Fields/actions fit; Escape closes and returns focus to the trigger. |
| Drawer | Four sides, close, actions, focus containment | Bottom drawer fits; increment changes 350 to 360. |
| Field | Label, description, invalid/error, disabled, orientation | Normal and invalid fields fit, with visible error treatment. |
| Fieldset | Legend, description, disabled cascading | Legend, labels and both text fields fit. |
| Form | Required validation, submit callback, first invalid focus | Empty submit focuses the field and shows error; entering text clears it; submit shows success. |
| Input | Focus, text, disabled, read-only, invalid | Editable, read-only and disabled examples fit with distinct styling. |
| Menu | Open/close, highlighted/disabled, checkbox/radio items | Popup fits; disabled item and checked markers render; radio click dismisses. |
| Menubar | Trigger focus, open menus, shared item states | File opens; hovering Edit replaces its popup. |
| Meter | Value/min/max, visible label, semantics | 68% label and fill render. |
| Navigation menu | Trigger, popup, active links, activation, generic children | Hover opens the aligned Docs panel; link rows fit. Keyboard limitation remains below. |
| Number field | Editing, minimum/maximum, disabled/read-only steppers | Seven increments from 4 stop at 10; upper stepper dims. All three modes fit. |
| OTP field | Empty/filled, focus, invalid, disabled, read-only | Typing 456 fills the remaining cells; disabled and read-only rows render. |
| Popover | Trigger, open/close, title/description, positioning | Open panel and fields fit within the preview. |
| Preview card | Hover/focus trigger, open/close, content | Delayed hover opens readable content within the frame. |
| Progress | Determinate/indeterminate, labels | Determinate and labelled indeterminate bars render; indeterminate has no animation in this port. |
| Radio group | Selected/unselected, disabled, full label hit target | Clicking Compact label changes the selected marker. |
| Scroll area | Overflow, scrollbars, thumb, viewport | Wheel scrolling moves visible rows from 1–5 to 8–12. |
| Select | Placeholder/selection, invalid, disabled/read-only, options | Opens with selected/disabled options; clicking Light changes the trigger value. |
| Separator | Horizontal/vertical, decorative semantics | Both directions render in the example. |
| Slider | Controlled value, step, range, disabled, focus | Clicking the track changes displayed volume from 50% to 80%. |
| Switch | Off/on, controlled label, read-only, disabled | Control turns on; label turns off, once each. |
| Tabs | Default/Line, selected, disabled, panel switching | Underline renders correctly; clicking Activity changes underline and panel. |
| Toast | Open/dismiss/action, visible feedback | Show toast displays its title/description; closed state returns. |
| Toggle | Pressed/unpressed, disabled, variants | Bold gains the pressed background when clicked; other variants fit. |
| Toggle group | Single/multiple, joined/separate, disabled | Both modes fit; clicking Italic keeps Bold selected in multiple mode. |
| Toolbar | Buttons, active styling, input focus/invalid/disabled | Typing in the input stays vertically centered with a focus ring; Copy is visibly disabled. |
| Tooltip | Hover/focus open, dismissal, label | Hover opens the tooltip above its trigger. |

## Limits requiring explicit follow-up

- Base GPUI NavigationMenuContent accepts generic children. Nested NavigationMenuLink nodes lose the typed focus wiring there. The pinned link implementation invokes activation on pointer and accessibility Click but does not itself register Enter/Space activation. Do not claim complete keyboard parity for these links. Resolve in Base GPUI before declaring this component keyboard-complete.
- GPUI's pinned accessibility API lacks some invalid, required, read-only and description relationships. See each component's parity notes; visual errors are not proof of screen-reader announcements.
- Short preview content is not proof that long modal content remains reachable at every small window size.
- Avatar badges/groups remain compositions, not new public primitives. URL images require an application HTTP client; the web showcase supplies one.

The 390px browser-emulation check was inconclusive: the reported device pixel ratio was 1 while the 348px canvas had a 696px backing buffer, so the preview rendered at half scale. The viewport override was reset. This is not counted as a mobile visual pass or a confirmed application defect.
