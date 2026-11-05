# Date Picker parity

Visual reference: shadcn/ui 4.19.0's date picker example (a Popover holding
a Calendar), Neutral Nova. Implementation:
`crates/ui/registry/date_picker/date_picker.rs`, using GPUI Kit 0.9.0.

GPUI Kit's `Popover` supplies the open state, outside-click and Escape
dismissal and focus return to the trigger. gpuicn draws the trigger as the
outline popover trigger with a Lucide `Calendar` icon and the date
("Mar 14, 2026", or `format`), muted placeholder text before a pick, and the
popover surface around a `Calendar`. Focus moves into the grid when it
opens, and picking a date or pressing Escape closes it.

The trigger is named with `aria_label` and the full date ("Due date,
Monday, March 16, 2026"). The date lives in the `CalendarState` the
application passes.
