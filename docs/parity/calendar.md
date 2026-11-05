# Calendar parity

Visual reference: shadcn/ui 4.19.0 `calendar.tsx` (react-day-picker),
Neutral Nova. Implementation: `crates/ui/registry/calendar/calendar.rs`,
using GPUI Kit 0.9.0.

GPUI Kit's calendar uses chrono, which it does not re-export, so gpuicn
keeps its own small `CalendarDate` (a proleptic Gregorian date with ISO
parsing, day arithmetic and labels) and draws the month itself: a caption
with ghost "Previous month" and "Next month" icon buttons, a muted weekday
row and 32px day cells with `radius.md`. The selected day is `primary`,
today is `accent`, days outside the month are muted, and days outside
`set_limits` are dimmed and cannot be picked.

The grid is one tab stop with the Grid role, named for the month. Arrow keys
move a day or a week, Page Up and Page Down a month (with Shift, a year),
Home and End the week's ends, Enter or Space picks, and Escape cancels.
Cells are GridCells named with the full date ("Saturday, March 14, 2026,
today"), mark the picked day as selected and the cursor as the active
descendant, and show the focus ring on the cursor. The first weekday is
Sunday by default (`set_first_weekday`). The application owns the date.
