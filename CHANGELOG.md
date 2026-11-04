# Changelog

Notable changes to gpuicn. Detailed notes for each release are in
[docs/releases](docs/releases/). Installed source never updates itself; run
`gpuicn diff` after upgrading to see what a release changes in your copy.

## 0.9.5

[Notes](docs/releases/v0.9.5.md)

### Added

- Message: `on_author_click` makes the name a focusable, link-styled button
  whose handler gets the name's bounds, and `author_element` replaces the
  name with any element, such as a popover trigger (#123).
- Toast: `Notice::on_dismiss` reports why a notice left (`NoticeDismissal`),
  `ToastState::contains(id)` says whether one is still showing, and
  `set_placement` / `set_inset` move the stack (#122).

## 0.9.4

[Notes](docs/releases/v0.9.4.md)

### Added

- Avatar: `status(AvatarStatus)` draws a presence dot (Online, Away, Dnd,
  a hollow Offline ring, or `Custom`) with a `ring_color` surface ring, and
  adds the status to the accessible name (#115).
- Popover: `hover_popover`, a popover that opens on hover without taking
  focus, and on click, Enter or Space with focus moving into it, for
  profile cards (#118).
- Embed: `QuotedMessage`, a quoted-post card for permalinks and forwards
  (#119).
- Sortable: a new component whose rows drag to reorder and move between
  groups, with Alt+Shift+Up and Down and an announcement; groups reorder too
  with `reorder_groups` (#121).
- A `test-support` crate feature with `gpuicn::audit`: `unnamed_controls`
  lists interactive elements without an accessible name, and
  `A11yAuditOverlay` outlines them in a debug build (#120).

### Changed

- Menu panels use the room in the window before they scroll, instead of
  stopping at 288px, and submenus open after resting the pointer on their
  row for 200ms (#117).
- Command and composer options, emoji picker cells and tabs, the OTP field,
  Slider, the Resizable handle, Virtual List and context-menu areas register
  for GPUI Kit's element queries with their role and name (#120).

## 0.9.3

[Notes](docs/releases/v0.9.3.md)

### Added

- Embed: `ImagePlaceholder`, a click-to-load placeholder for remote images,
  through `LinkPreview::image_placeholder`, `MessageEmbed::image_placeholder`
  and `MessageEmbed::thumbnail_placeholder` (#111).

## 0.9.2

[Notes](docs/releases/v0.9.2.md)

### Added

- Embed: `LinkPreview` (Open Graph previews with collapse and remove) and
  `MessageEmbed` (integration attachments with a colored bar, fields and a
  footer) for cards under a chat message (#108).

### Fixed

- A dialog taller than the window scrolls with the wheel over its surface:
  `dialog_popup` now blocks the mouse except scrolling, so the wheel reaches
  `modal_viewport` (#109).

## 0.9.1

[Notes](docs/releases/v0.9.1.md)

### Added

- Message `timestamp_tooltip` and `on_gutter_click`.
- Toast `Notice::action` and `Notice::cancel`.
- MessageList `on_jump` and `show_jump`.
- Attachment `dimensions` and `max_size` for image thumbnails.
- Sidebar `roving_focus`: one tab stop with arrow-key navigation.
- Composer `max_height`, `ComposerState::set_max_rows` and `max_rows`.
- ScrollArea `track_scroll`.

## 0.9.0

[Notes](docs/releases/v0.9.0.md)

### Added

- `app::run` and `AppConfig`: one-call application start with icons, fonts,
  a theme pair and the first window.
- `Appearance`, `UiTheme::set_pair`, `UiTheme::set_appearance` and
  `theme::follow_system`.
- CLI: shadcn base colors, `init --base-color/--radius/--appearance/--theme/--all`,
  `theme set|import|preview`, `update`, `doctor`, `remove --unused`; config
  version 2 with a `[theme]` table.
- `theme::primitives` (text, icon, focus, disabled, surface, control,
  clipped_image, dismiss, hover_reveal_in_group), status colors, typography
  and breakpoint tokens, `transition_with`, `presence_with`, `UiEasing::Emphasized`.
- `MessageListScrollbar` and `MessageList::scrollbar_handle`.
- `MessageDensity::Compact` for Message.
- `cargo xtask new-component`, `docs/design.md`, the registry lint and the
  catalog screenshot sheet in every release.

### Changed

- Every component reads colors, sizes, radii, shadows and motion from the
  theme through the primitives; focus is one 3px outer ring everywhere.
- Input and Textarea share `InputCore`; dialogs fade in and let the host
  viewport scroll.
- Toast uses deadlines instead of a 50ms poll; one Escape interceptor serves
  every tooltip.
- `UiTheme::shadows` entries are `Arc<[BoxShadow]>`; `sidebar_is_mobile`
  takes the window.

### Fixed

- The MessageList scrollbar thumb tracks position through unmeasured rows (#83).
- Every GPUI Kit component color follows a gpuicn token in both modes.

## 0.8.0

[Notes](docs/releases/v0.8.0.md)

### Added

- Emoji Picker: `EmojiPickerState::open`, which applies recents held since
  the last pick.
- Kbd: `key_label` for shortcut hints outside a key cap.

### Changed

- GPUI Kit 0.9.0 on `gpui-pre` 0.3.11, and gpui-icons 0.6.0.
- The dark modal scrim is 50% black.
- Kbd and Command draw modifier and arrow symbols as icons.

### Fixed

- Rounded frames in Command, Emoji Picker, Attachment, Markdown images and
  thumbnails no longer show square corners or lose their borders.
- Dialogs and the mobile sidebar close on a backdrop press.
- Kit colors gpuicn doesn't set, such as scrollbar tracks, follow dark mode.
- Message continuation rows don't grow when the hover time shows.

## 0.7.1

[Notes](docs/releases/v0.7.1.md)

### Changed

- No component, API or CLI changes. The website preview is smaller, so the
  0.7.0 components can deploy, and releases now run entirely in CI.

## 0.7.0

[Notes](docs/releases/v0.7.0.md)

### Added

- Composer, Emoji Picker and Lightbox.
- Message: custom emoji images and who-reacted tooltips on reactions, a
  permalink highlight and per-row accessible descriptions.
- Message List: keyboard selection mode.
- Markdown: `selection_root` for selecting and copying across messages,
  `CustomEmoji` inline images and `MarkdownImages` for app-supplied images.
- Command: leading and trailing item slots.

### Changed

- Message now depends on Tooltip in the registry.

## 0.6.0

[Notes](docs/releases/v0.6.0.md)

### Added

- Chat and app-shell components: Command, Message, Message List, Attachment,
  Title Bar and Markdown (with an `EntityLinks` plugin for mention and channel
  chips).
- Small primitives: Kbd, Badge, Tag, Spinner and Skeleton.
- A `syntect` feature on the `gpuicn` crate that turns on code-block
  highlighting in Markdown.

### Changed

- The theme now passes link, selection, table and code-highlight colors to
  GPUI Kit. Copied-source apps should run `gpuicn add theme --overwrite` so
  Markdown picks them up.

## 0.5.0

[Notes](docs/releases/v0.5.0.md)

The first release without a beta suffix. gpuicn is still experimental, and
APIs can change between minor versions.

### Changed

- GPUI Kit 0.8.1 on its pinned `gpui-pre` 0.3.10 runtime (was 0.6.4 on 0.3.5).
  Update `gpui-kit`, `gpui_platform` and `gpui-icons` together; `gpuicn init`
  writes the new pins.
- gpui-icons 0.5.0 with Lucide 1.63.0: 1,924 icons, up from 1,818. Icons
  Lucide renamed, such as `LucideIcon::Music`, still compile.
- The WASM build uses wasm-bindgen 0.2.127.
- The minimum Rust is now 1.98, which gpui-pre 0.3.10 needs on macOS. CI
  builds with 1.99.1.

## 0.5.0-beta.5

[Notes](docs/releases/v0.5.0-beta.5.md)

### Added

- `gpuicn new <path>` creates an app ready for `cargo run`.
- `gpuicn init` now sets up an existing app. It adds the release's pinned Cargo
  dependencies, downloads Geist with checksum verification, and writes a starter
  window over the `cargo new` default. `--config-only` keeps the old behavior.
- `gpuicn status`, `gpuicn diff` and `gpuicn remove` for managing installed source.
- A `cargo gpuicn` alias is installed with the CLI.
- Crate mode: `gpuicn new --crate` and `gpuicn init --crate` depend on the
  `gpuicn` crate instead of copying source. `gpuicn eject <component>` copies
  one component into `src/ui` to edit, and `gpuicn remove` hands it back to the
  crate. `ui::<component>` paths work the same in both modes.
- The `gpuicn` crate is ready for crates.io, and gpui-icons is pinned to 0.4.2.
- `UiTheme::from_shadcn_css` imports shadcn and tweakcn theme CSS.
- Component names accept `alert_dialog` or `alert-dialog`. Unknown names
  suggest close matches, and `add` warns about missing Cargo dependencies.

### Changed (breaking)

Component APIs now share one shape: required inputs go to `new`, everything
optional is a builder method, and `cx` is taken only when a function reads it.
Run `gpuicn diff` to see the new source, then update call sites:

| Before | After |
| --- | --- |
| `toasts.push(id, title, description, Some(timeout), cx)` | `toasts.push(id, Notice::new(title).description(description).timeout(timeout), cx)`; use `.persistent()` for no timeout |
| `Resizable::new(id, label, size, first, second, on_resize)` | `Resizable::new(id, label, size).first(first).second(second).on_resize(on_resize)` |
| `SidebarLayout::new(id, state, mobile, sidebar, content, on_change)` | `SidebarLayout::new(id, state).mobile(mobile).sidebar(sidebar).content(content).on_change(on_change)` |
| `dialog_trigger(id, &handle, cx)` | `dialog_trigger(id, &handle)` |
| `dialog_action(id, &handle, cx)` | `dialog_action(id, &handle)` |
| `fieldset_root(id, cx)` | `fieldset(id, cx)` |

### Changed

- Component source moved from `registry/` to `crates/ui/registry/` so the
  `gpuicn` crate can be packaged. Crate-internal helpers are now public but
  hidden from the docs, because ejected components reach them from outside.
- The CLI shares the gpuicn version, and its default registry is the snapshot
  for that release tag rather than the live site.
- Minimum supported Rust is now 1.97 (was 1.97.1), with gpui-icons pinned to a
  revision that supports it.
- `UiTheme::read` panics with instructions to call `ui::theme::init(cx)` when
  the theme is missing.
- CI runs the CLI and UI tests on Linux on the minimum Rust, and sets up a new
  app with the CLI.

## 0.5.0-beta.4

GPUI Kit 0.6.4, GPUI 0.3.5 and gpui-icons 0.4.1. Adds Carousel, Empty and Input
Group, and Textarea as its own install target. [Notes](docs/releases/v0.5.0-beta.4.md)

## 0.5.0-beta.3

Focus is now shown with a border color only. `UiTheme::focus_ring()` and
`UiTheme::destructive_focus_ring()` were removed. [Notes](docs/releases/v0.5.0-beta.3.md)

## 0.5.0-beta.2

Moves every component and gpuicn Workspace to GPUI Kit 0.6.1, with Kit-based
input, menu, selector, form, dialog and drawer APIs. [Notes](docs/releases/v0.5.0-beta.2.md)

## 0.5.0-beta.1

Adds Sidebar, Resizable, Virtual List, the native source-installation CLI and
the gpuicn Workspace example app. [Notes](docs/releases/v0.5.0-beta.1.md)
