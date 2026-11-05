# Composer parity

shadcn/ui has no chat composer. Implementation:
`crates/ui/registry/composer/composer.rs`, built on GPUI Kit 0.8.1's
`TextareaState` and gpuicn's `Textarea`.

`Composer` draws one Nova frame around a borderless textarea and an optional
toolbar row of caller children, such as attach, format, emoji and send
buttons. The frame takes the ring color while the editor has focus. The editor
grows from one line to the `max_rows` passed to `ComposerState::new`, then
scrolls.

Enter emits `ComposerEvent::Submit` with the text and Shift+Enter adds a line;
`set_submit_on_enter(false)` turns that off. The application sends the text and
clears the composer with `set_value("")`. Keys during IME composition go to the
input method, so Enter confirms a CJK candidate instead of sending.

`ComposerState` exposes the editing primitives a chat client needs:

- `value` and `set_value`;
- `selection` and `set_selection`, as byte ranges;
- `insert`, which replaces the selection or inserts at the caret;
- `wrap_selection`, for Markdown shortcuts such as bold;
- `caret_bounds`, the caret rectangle in window coordinates;
- `is_composing`, true while an input method has marked text;
- `set_on_key`, which sees each key press before the composer and the editor
  and returns `true` to claim it;
- `set_on_paste`, which sees each paste first and returns `true` to claim it,
  for example to upload pasted images or files. Browser builds only see text
  pastes through this hook.

Autocomplete starts when the caret follows a trigger character at the start of
the text or after whitespace: `@`, `~` and `:` anywhere, `/` only at the start.
`set_triggers` changes the set. The composer emits
`ComposerEvent::Autocomplete { trigger, query }` as the query changes and the
application answers with `set_suggestions`, synchronously or after a server
search. Suggestions show in a popover anchored above the trigger character,
with an optional icon, or a caller element from `set_suggestion_leading` such
as an avatar or a custom emoji image. Up and Down move the highlight, Enter or
Tab accepts, and Escape dismisses the popup until the caret leaves that
trigger. Accepting replaces the trigger and query with the suggestion's value
and a space, and emits `ComposerEvent::Accept`.

The popup is a `ListBox` with `ListBoxOption` rows named by their labels, and
the highlighted row is the active descendant.

Triggers inside inline code or a fenced block no longer open suggestions
(`set_ignore_triggers_in_code`). `open_suggestions(range, items)` opens the
popup for a range the application chose, such as a slash command's
argument, and accepting replaces that range (`ComposerEvent::AcceptRange`).
`set_misspellings` underlines words in the composer as in `Textarea`.
