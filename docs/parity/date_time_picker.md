# Date Time Picker parity

Visual reference: shadcn/ui's date and time picker example (a Calendar and a
time input in one Popover), Neutral Nova. Implementation:
`crates/ui/registry/date_time_picker/date_time_picker.rs`, using GPUI Kit
0.9.0.

The trigger matches `DatePicker` with a `CalendarClock` icon and "Mar 14,
2026, 9:30 AM". The popover holds a `Calendar`, a divider and a row with a
Lucide `Clock` icon, a time field and a small Done button. Picking a day
moves focus to the time field; Done, Escape in the grid or a click outside
closes it.

The time field is one tab stop in the control frame with the mono font.
Left and Right pick the hour, minute or AM/PM segment (shown in `accent`),
Up and Down step it, digits type it (a second digit, or a first no second
could follow, moves on), and A or P sets the half of the day. Each segment
is a SpinButton with its value. `HourCycle::for_locale` picks a 12- or
24-hour clock, and `set_limits` keeps the date and time inside a range, such
as "not in the past" for a scheduled message. `DateTimeEvent::Change`
reports each change; the application owns the value.
