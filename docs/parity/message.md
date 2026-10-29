# Message parity

shadcn/ui has no team-chat message row. gpuicn's Message follows the slot model
of GPUI Kit 0.8.1's `Message` (avatar, header, content, footer), laid out for
left-aligned team chat as in Slack and Mattermost rather than Kit's bubble
layout. Implementation: `crates/ui/registry/message/message.rs`.

- A row has a 32px avatar gutter, a semibold name, a muted time, optional header
  children such as a Bot tag, and body content at 14px with a 1.45 line height.
- `continuation(true)` hides the avatar and header for consecutive messages from
  one sender; hovering shows `gutter_time` in the gutter instead.
- Hover tints the row and shows the `actions` bar at the top trailing edge.
  `MessageHighlight::Selected` keeps the bar visible, so a keyboard selection
  model can expose the same actions.
- `MessageHighlight::Mention` tints the row with the primary color and draws a
  primary bar in the gutter.
- `MessageStatus::Pending` dims the row. `MessageStatus::Failed` adds a
  destructive notice with Retry and Delete link buttons.
- Reaction chips are outline pill buttons with the emoji and count, filled with
  a primary tint when the reader reacted. They take focus and report the emoji
  through `on_reaction`; `on_add_reaction` adds an icon button for a picker.

`day_divider` and `new_messages_divider` draw the date pill and the destructive
"New messages" rule. Message content, grouping rules, time formatting and the
emoji picker stay with the application.
