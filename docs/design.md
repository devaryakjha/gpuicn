# Design rules

How a gpuicn component looks and behaves is decided once, in the theme and its
primitives, and checked by tests. This page is the short version of those
rules for anyone adding or changing a component; the long version is the code
in `crates/ui/registry/theme/` and the checks in `crates/ui/tests/`.

## Source of truth

- **Visuals**: shadcn/ui at the revision in `release-pins.toml` (Nova style,
  Neutral base color, Geist). A component matches its shadcn counterpart at
  the default size in light and dark mode; the parity note in `docs/parity/`
  records what differs and why.
- **Behavior**: GPUI Kit's base components supply focus, keyboard navigation,
  dismissal and accessibility roles. gpuicn styles them and composes them; it
  does not reimplement them.
- **Ownership**: the installed source belongs to the app. Anything an app may
  want to change is a theme token or a builder method, never a constant in a
  component.

## Tokens, not literals

Every color, size, radius, shadow and duration comes from `UiTheme`:

| Need | Use |
| --- | --- |
| A color | `theme.colors.<token>`; add a semantic token when none fits |
| Text size and line height | `text::xs/sm/base/lg/xl` or `theme.text_style(..)` from `theme::primitives` |
| An icon size | `icon::xs/sm/md/lg/xl` |
| Spacing | `theme.space(n)`; one unit is 4px by default |
| A radius | `theme.radius.<step>` or `control::box_radius(&theme)` |
| A shadow | `theme.shadows.<step>.to_vec()` |
| A surface | `surface::popover/menu/modal/sheet/toast/card/floating/tooltip` |
| Focus | `focus::ring` through `focus_visible`: a 3px outer ring in `ring` at 50% |
| Disabled | `disabled::style`, `disabled::interactive` or `disabled::dim` |
| Hover and press | `theme.control()` colors, applied with `.hover(..)`; actions revealed on hover use `hover_reveal_in_group` |
| Motion | `theme.motion` durations and `UiEasing`; `transition_with` / `presence_with` |
| Mode differences | the derived `control_*` tokens; a component never branches on `ThemeMode` |
| Key symbols | `kbd::key_label`; the bundled fonts have no glyphs for ⌘ ⇧ ⌥ ⌃ |
| Images | `clipped_image`, because GPUI clips to rectangles |

The registry lint (`crates/ui/tests/registry_lint.rs`) fails a pull request
that adds a raw color, a `text_size(px(..))`, an `opacity(0.5)`, a
`ThemeMode::Dark` branch, a popover background outside the primitives, a
`"hovered"` state, a key glyph, a `pub(crate)` item (ejected files reach the
crate through `super::`) or a focus style keyed on `is_focused`. Its baseline
can only go down.

## Every component has

- a module `crates/ui/registry/<name>/<name>.rs` declared in
  `crates/ui/src/lib.rs` and `registry.toml`, with `missing_docs` satisfied;
- a test, in its own `#[cfg(test)]` module or in `crates/ui/tests/catalog.rs`,
  that at least draws it in both themes (the lint lists untested files);
- a catalog preview in `site/src/previews/<name>.rs`, listed in
  `site/src/previews/mod.rs` and `site/src/gallery.rs`;
- a Usage example in `crates/ui/examples/usage.rs` that compiles against the
  crate and against the installed copy;
- a parity note in `docs/parity/<name>.md`.

`cargo xtask new-component <name>` creates all of them, so the gates pass on
the first build and the work is the component itself.

## Evidence

Every visible change comes with before and after screenshots at the same
window size, light and dark. `scripts/screenshots.mjs` shoots the whole
catalog from the built site and diffs it against the newest folder under
`docs/screenshots/`; CI attaches the sheet to each run and each release ships
it as `catalog-screenshots-<tag>.zip`. Release notes link the sheet.

## Keeping apps consistent with little effort

- `gpuicn init` writes one theme file (`assets/theme.css`) from a shadcn base
  color and generates `src/ui/app_theme.rs`; `app::run` applies it and follows
  the OS appearance. Changing a base color, radius or appearance is one
  `gpuicn theme set` command.
- `gpuicn doctor` reports anything that drifted from the release the CLI was
  built from, with the command that fixes it; `gpuicn update` moves an app to
  a newer release in one step.
- Components read every visual from the theme, so an app that changes a token
  changes every component the same way.
