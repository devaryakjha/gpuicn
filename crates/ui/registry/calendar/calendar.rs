//! Nova Calendar: a month grid for picking one date with the keyboard or
//! the pointer.
//!
//! Visual source: shadcn/ui 4.19.0 `calendar.tsx` (react-day-picker), Nova
//! style. The grid is one tab stop: arrow keys move by day and week, Page Up
//! and Page Down by month (with Shift, by year), Home and End to the ends of
//! the week, Enter or Space picks, and Escape cancels. GPUI Kit's
//! `Calendar` has no keyboard grid, so this one keeps its own cursor, and
//! dates are a small `CalendarDate` type rather than a chrono dependency.

use std::fmt;
use std::rc::Rc;

use gpui_icons::{LucideIcon, lucide};
use gpui_kit::base::TestSupportExt as _;
use gpui_kit::{
    App, Context, ElementId, Entity, EventEmitter, FocusHandle, Focusable, FontWeight,
    InteractiveElement as _, IntoElement, KeyDownEvent, ParentElement as _, Render, RenderOnce,
    Role, SharedString, StatefulInteractiveElement as _, StyleRefinement, Styled, Window, div,
    prelude::FluentBuilder as _,
};

use super::button::{Button, ButtonSize, ButtonVariant};
use super::theme::{
    UiText, UiTheme, apply_style,
    locale::{self, DirectionalStyled as _},
    primitives::{disabled, focus},
};

/// A calendar date with no time or time zone, such as 2026-03-14.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CalendarDate {
    year: i32,
    month: u8,
    day: u8,
}

/// A day of the week.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Weekday {
    /// Sunday, the first day of the week in the United States.
    #[default]
    Sunday,
    /// Monday, the first day of the week in most of Europe.
    Monday,
    /// Tuesday.
    Tuesday,
    /// Wednesday.
    Wednesday,
    /// Thursday.
    Thursday,
    /// Friday.
    Friday,
    /// Saturday.
    Saturday,
}

impl Weekday {
    const ALL: [Weekday; 7] = [
        Weekday::Sunday,
        Weekday::Monday,
        Weekday::Tuesday,
        Weekday::Wednesday,
        Weekday::Thursday,
        Weekday::Friday,
        Weekday::Saturday,
    ];

    /// Days after Sunday, 0 to 6.
    pub fn index(self) -> usize {
        self as usize
    }

    /// The English name, such as "Monday".
    pub fn name(self) -> &'static str {
        [
            "Sunday",
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
        ][self.index()]
    }

    /// The two-letter column heading, such as "Mo".
    pub fn short(self) -> &'static str {
        &self.name()[..2]
    }
}

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// The English name of month 1 to 12, such as "March".
pub fn month_name(month: u8) -> &'static str {
    MONTHS[(month.clamp(1, 12) - 1) as usize]
}

fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// The number of days in `month` (1 to 12) of `year`.
pub fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap(year) => 29,
        _ => 28,
    }
}

impl CalendarDate {
    /// The date, or `None` when the month or day does not exist.
    pub fn new(year: i32, month: u8, day: u8) -> Option<Self> {
        ((1..=12).contains(&month) && day >= 1 && day <= days_in_month(year, month))
            .then_some(Self { year, month, day })
    }

    /// The year.
    pub fn year(self) -> i32 {
        self.year
    }

    /// The month, 1 to 12.
    pub fn month(self) -> u8 {
        self.month
    }

    /// The day of the month, from 1.
    pub fn day(self) -> u8 {
        self.day
    }

    /// Days since 1970-01-01, negative before it.
    pub fn to_days(self) -> i64 {
        // Howard Hinnant's days_from_civil.
        let year = i64::from(self.year) - i64::from(self.month <= 2);
        let era = year.div_euclid(400);
        let year_of_era = year - era * 400;
        let month = i64::from(self.month);
        let day_of_year =
            (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + i64::from(self.day) - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        era * 146_097 + day_of_era - 719_468
    }

    /// The date `days` after 1970-01-01.
    pub fn from_days(days: i64) -> Self {
        let days = days + 719_468;
        let era = days.div_euclid(146_097);
        let day_of_era = days - era * 146_097;
        let year_of_era =
            (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let month_index = (5 * day_of_year + 2) / 153;
        let day = (day_of_year - (153 * month_index + 2) / 5 + 1) as u8;
        let month = if month_index < 10 {
            month_index + 3
        } else {
            month_index - 9
        } as u8;
        let year = year_of_era + era * 400 + i64::from(month <= 2);
        Self {
            year: year as i32,
            month,
            day,
        }
    }

    /// Today in UTC. Apps that know the reader's time zone pass their own
    /// date to `CalendarState::set_today`.
    pub fn today_utc() -> Self {
        let seconds = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_secs() as i64);
        Self::from_days(seconds.div_euclid(86_400))
    }

    /// The day of the week.
    pub fn weekday(self) -> Weekday {
        // 1970-01-01 was a Thursday.
        Weekday::ALL[(self.to_days() + 4).rem_euclid(7) as usize]
    }

    /// The date `days` later, or earlier when negative.
    pub fn add_days(self, days: i64) -> Self {
        Self::from_days(self.to_days() + days)
    }

    /// The same day `months` later, or earlier, clamped to the month's last
    /// day: January 31 plus one month is February 28 or 29.
    pub fn add_months(self, months: i32) -> Self {
        let index = self.year * 12 + i32::from(self.month) - 1 + months;
        let year = index.div_euclid(12);
        let month = (index.rem_euclid(12) + 1) as u8;
        Self {
            year,
            month,
            day: self.day.min(days_in_month(year, month)),
        }
    }

    /// The first day of this date's month.
    pub fn first_of_month(self) -> Self {
        Self { day: 1, ..self }
    }

    /// Reads `YYYY-MM-DD`.
    pub fn parse_iso(text: &str) -> Option<Self> {
        let mut parts = text.trim().splitn(3, '-');
        let year = parts.next()?.parse().ok()?;
        let month = parts.next()?.parse().ok()?;
        let day = parts.next()?.parse().ok()?;
        Self::new(year, month, day)
    }

    /// The date read aloud, such as "Saturday, March 14, 2026".
    pub fn long_label(self) -> String {
        format!(
            "{}, {} {}, {}",
            self.weekday().name(),
            month_name(self.month),
            self.day,
            self.year
        )
    }

    /// The date read aloud, as a translatable [`UiText::LongDate`].
    pub fn long_text(self) -> UiText {
        UiText::LongDate {
            weekday: self.weekday().index() as u8,
            month: self.month,
            day: self.day,
            year: self.year,
        }
    }

    /// The compact label, as a translatable [`UiText::ShortDate`].
    pub fn short_text(self) -> UiText {
        UiText::ShortDate {
            month: self.month,
            day: self.day,
            year: self.year,
        }
    }

    /// A compact label, such as "Mar 14, 2026".
    pub fn short_label(self) -> String {
        format!(
            "{} {}, {}",
            &month_name(self.month)[..3],
            self.day,
            self.year
        )
    }
}

/// `YYYY-MM-DD`.
impl fmt::Display for CalendarDate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:04}-{:02}-{:02}",
            self.year, self.month, self.day
        )
    }
}

/// Notifications from a calendar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CalendarEvent {
    /// A date was picked with Enter, Space or a click.
    Select(CalendarDate),
    /// Escape was pressed.
    Cancel,
}

/// The selected date, the keyboard cursor, the shown month and the limits.
pub struct CalendarState {
    selected: Option<CalendarDate>,
    cursor: CalendarDate,
    today: CalendarDate,
    min: Option<CalendarDate>,
    max: Option<CalendarDate>,
    first_weekday: Weekday,
    #[doc(hidden)]
    pub focus: FocusHandle,
}

impl EventEmitter<CalendarEvent> for CalendarState {}

impl Focusable for CalendarState {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl CalendarState {
    /// Creates a calendar showing this month, with nothing selected.
    pub fn new(cx: &mut Context<Self>) -> Self {
        let today = CalendarDate::today_utc();
        Self {
            selected: None,
            cursor: today,
            today,
            min: None,
            max: None,
            first_weekday: Weekday::Sunday,
            focus: cx.focus_handle(),
        }
    }

    /// The selected date.
    pub fn selected(&self) -> Option<CalendarDate> {
        self.selected
    }

    /// Selects a date, or clears the selection, and shows its month.
    pub fn set_selected(&mut self, date: Option<CalendarDate>, cx: &mut Context<Self>) {
        self.selected = date;
        if let Some(date) = date {
            self.cursor = self.clamp(date);
        }
        cx.notify();
    }

    /// The date the keyboard cursor is on; its month is the one shown.
    pub fn cursor(&self) -> CalendarDate {
        self.cursor
    }

    /// Moves the cursor, and so the shown month, without selecting.
    pub fn set_cursor(&mut self, date: CalendarDate, cx: &mut Context<Self>) {
        self.cursor = self.clamp(date);
        cx.notify();
    }

    /// The date marked as today.
    pub fn today(&self) -> CalendarDate {
        self.today
    }

    /// Marks `date` as today, for apps that know the reader's time zone.
    pub fn set_today(&mut self, date: CalendarDate, cx: &mut Context<Self>) {
        if self.cursor == self.today && self.selected.is_none() {
            self.cursor = date;
        }
        self.today = date;
        cx.notify();
    }

    /// Limits picking to dates on or after `min` and on or before `max`.
    pub fn set_limits(
        &mut self,
        min: Option<CalendarDate>,
        max: Option<CalendarDate>,
        cx: &mut Context<Self>,
    ) {
        self.min = min;
        self.max = max;
        self.cursor = self.clamp(self.cursor);
        cx.notify();
    }

    /// The first column of each week, Sunday by default.
    pub fn first_weekday(&self) -> Weekday {
        self.first_weekday
    }

    /// Sets the first column of each week.
    pub fn set_first_weekday(&mut self, weekday: Weekday, cx: &mut Context<Self>) {
        self.first_weekday = weekday;
        cx.notify();
    }

    /// Whether `date` is within the limits.
    pub fn is_enabled(&self, date: CalendarDate) -> bool {
        self.min.is_none_or(|min| date >= min) && self.max.is_none_or(|max| date <= max)
    }

    /// Moves keyboard focus into the grid.
    pub fn focus(&self, window: &mut Window, cx: &mut App) {
        self.focus.focus(window, cx);
    }

    /// Selects `date` and emits `CalendarEvent::Select`, unless it is
    /// outside the limits.
    pub fn pick(&mut self, date: CalendarDate, cx: &mut Context<Self>) -> bool {
        if !self.is_enabled(date) {
            return false;
        }
        self.selected = Some(date);
        self.cursor = date;
        cx.emit(CalendarEvent::Select(date));
        cx.notify();
        true
    }

    /// Shows the previous (negative) or next month, keeping the cursor's day.
    pub fn shift_month(&mut self, months: i32, cx: &mut Context<Self>) {
        self.cursor = self.clamp(self.cursor.add_months(months));
        cx.notify();
    }

    fn clamp(&self, date: CalendarDate) -> CalendarDate {
        let date = self.min.map_or(date, |min| date.max(min));
        self.max.map_or(date, |max| date.min(max))
    }

    /// The six weeks shown for the cursor's month, starting on the first weekday.
    pub fn weeks(&self) -> Vec<[CalendarDate; 7]> {
        let first = self.cursor.first_of_month();
        let lead = (first.weekday().index() + 7 - self.first_weekday.index()) % 7;
        let start = first.add_days(-(lead as i64));
        (0..6)
            .map(|week| std::array::from_fn(|day| start.add_days((week * 7 + day) as i64)))
            .collect()
    }

    /// Handles a calendar key. Returns whether it was used.
    pub fn handle_key(&mut self, key: &str, shift: bool, cx: &mut Context<Self>) -> bool {
        let weekday_column = (self.cursor.weekday().index() + 7 - self.first_weekday.index()) % 7;
        let next = match key {
            "left" => self.cursor.add_days(-1),
            "right" => self.cursor.add_days(1),
            "up" => self.cursor.add_days(-7),
            "down" => self.cursor.add_days(7),
            "pageup" => self.cursor.add_months(if shift { -12 } else { -1 }),
            "pagedown" => self.cursor.add_months(if shift { 12 } else { 1 }),
            "home" => self.cursor.add_days(-(weekday_column as i64)),
            "end" => self.cursor.add_days(6 - weekday_column as i64),
            "enter" | "space" => {
                self.pick(self.cursor, cx);
                return true;
            }
            "escape" => {
                cx.emit(CalendarEvent::Cancel);
                return true;
            }
            _ => return false,
        };
        self.cursor = self.clamp(next);
        cx.notify();
        true
    }
}

impl Render for CalendarState {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        Calendar::new(&cx.entity())
    }
}

type SelectHandler = Rc<dyn Fn(CalendarDate, &mut Window, &mut App)>;
type CancelHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// A month grid bound to retained `CalendarState`.
///
/// It works on its own or inside a popover or menu, for example under a
/// search field's suggestions. Subscribe to `CalendarEvent`, or pass
/// `on_select` and `on_cancel` here.
#[derive(IntoElement)]
pub struct Calendar {
    state: Entity<CalendarState>,
    on_select: Option<SelectHandler>,
    on_cancel: Option<CancelHandler>,
    style: StyleRefinement,
}

impl Calendar {
    /// Creates a calendar for retained state.
    pub fn new(state: &Entity<CalendarState>) -> Self {
        Self {
            state: state.clone(),
            on_select: None,
            on_cancel: None,
            style: StyleRefinement::default(),
        }
    }

    /// Runs after a date is picked, for example to close a popover.
    pub fn on_select(
        mut self,
        handler: impl Fn(CalendarDate, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    /// Runs when Escape is pressed in the grid.
    pub fn on_cancel(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_cancel = Some(Rc::new(handler));
        self
    }
}

impl Styled for Calendar {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Calendar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let id = ElementId::from(("calendar", self.state.entity_id()));
        let state = self.state.read(cx);
        let focused = state.focus.is_focused(window);
        let cursor = state.cursor;
        let selected = state.selected;
        let today = state.today;
        let weeks = state.weeks();
        let first_weekday = state.first_weekday;
        let can_go_back = state
            .min
            .is_none_or(|min| cursor.first_of_month() > min.first_of_month());
        let can_go_forward = state
            .max
            .is_none_or(|max| cursor.first_of_month() < max.first_of_month());
        let enabled: Vec<[bool; 7]> = weeks
            .iter()
            .map(|week| week.map(|date| state.is_enabled(date)))
            .collect();
        let focus_handle = state.focus.clone();
        let title = locale::text(
            cx,
            UiText::MonthYear {
                month: cursor.month,
                year: cursor.year,
            },
        );
        let cell = theme.space(8.);
        let dir = theme.direction;
        let today_text = locale::text(cx, UiText::Today);
        let unavailable_text = locale::text(cx, UiText::Unavailable);
        let short_days: Vec<SharedString> = (0..7)
            .map(|day| locale::text(cx, UiText::WeekdayShort(day)))
            .collect();
        let previous_label = locale::text(cx, UiText::PreviousMonth);
        let next_label = locale::text(cx, UiText::NextMonth);
        let long_labels: Vec<Vec<SharedString>> = weeks
            .iter()
            .map(|week| {
                week.iter()
                    .map(|date| locale::text(cx, date.long_text()))
                    .collect()
            })
            .collect();

        let nav = |direction: i32, key: &'static str, label: SharedString, enabled: bool| {
            let state = self.state.clone();
            // Previous points toward the reading start.
            let icon = dir.icon(
                direction > 0,
                LucideIcon::ChevronLeft,
                LucideIcon::ChevronRight,
            );
            Button::new((id.clone(), key))
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Icon)
                .aria_label(label)
                .disabled(!enabled)
                .on_click(move |_, _, cx| {
                    state.update(cx, |state, cx| state.shift_month(direction, cx))
                })
                .child(
                    lucide(icon)
                        .size(theme.space(4.))
                        .text_color(theme.colors.foreground),
                )
        };
        let header = div()
            .flex_row_in(dir)
            .items_center()
            .justify_between()
            .h(cell)
            .child(nav(-1, "Previous month", previous_label, can_go_back))
            .child(
                div()
                    .id((id.clone(), "title"))
                    .role(Role::Heading)
                    .aria_label(title.clone())
                    .font_weight(FontWeight::MEDIUM)
                    .child(title.clone()),
            )
            .child(nav(1, "Next month", next_label, can_go_forward));

        let weekdays = div().flex_row_in(dir).children((0..7).map(|column| {
            let weekday = Weekday::ALL[(first_weekday.index() + column) % 7];
            div()
                .w(cell)
                .flex()
                .justify_center()
                .text_size(theme.text(12.8))
                .text_color(colors.muted_foreground)
                .child(short_days[weekday.index()].clone())
        }));

        let on_select = self.on_select.clone();
        let rows =
            weeks
                .into_iter()
                .zip(enabled)
                .zip(long_labels)
                .map(|((week, enabled), labels)| {
                    div().flex_row_in(dir).mt(theme.space(1.)).children(
                        week.into_iter().zip(enabled).zip(labels).map(
                            |((date, enabled), label)| {
                                let outside = date.month != cursor.month;
                                let is_selected = selected == Some(date);
                                let is_cursor = date == cursor;
                                let is_today = date == today;
                                let mut label = label.to_string();
                                if is_today {
                                    label.push_str(", ");
                                    label.push_str(&today_text);
                                }
                                if !enabled {
                                    label.push_str(", ");
                                    label.push_str(&unavailable_text);
                                }
                                let state = self.state.clone();
                                let on_select = on_select.clone();
                                div()
                                    .id((id.clone(), SharedString::from(date.to_string())))
                                    .test_support()
                                    .role(Role::GridCell)
                                    .aria_label(SharedString::from(label))
                                    .aria_selected(is_selected)
                                    .when(is_cursor, |cell| cell.aria_active_descendant())
                                    .size(cell)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded(theme.radius.md)
                                    .text_size(theme.text(14.))
                                    .text_color(if outside {
                                        colors.muted_foreground
                                    } else {
                                        colors.foreground
                                    })
                                    .when(is_today && !is_selected, |cell| {
                                        cell.bg(colors.accent).text_color(colors.accent_foreground)
                                    })
                                    .when(is_selected, |cell| {
                                        cell.bg(colors.primary)
                                            .text_color(colors.primary_foreground)
                                            .font_weight(FontWeight::MEDIUM)
                                    })
                                    .when(is_cursor && focused, |cell| {
                                        cell.shadow(focus::shadow(theme.control().ring))
                                    })
                                    .map(|cell| {
                                        if enabled {
                                            cell.cursor_pointer()
                                                .when(!is_selected, |cell| {
                                                    cell.hover(move |style| style.bg(colors.accent))
                                                })
                                                .on_click(move |_, window, cx| {
                                                    let picked = state.update(cx, |state, cx| {
                                                        state.pick(date, cx)
                                                    });
                                                    if picked && let Some(handler) = &on_select {
                                                        handler(date, window, cx);
                                                    }
                                                })
                                        } else {
                                            disabled::style(cell)
                                        }
                                    })
                                    .child(date.day.to_string())
                            },
                        ),
                    )
                });

        let key_state = self.state.clone();
        let on_select = self.on_select;
        let on_cancel = self.on_cancel;
        let root = div()
            .id(id.clone())
            .test_support()
            .key_context("Calendar")
            .track_focus(&focus_handle)
            .role(Role::Grid)
            .aria_label(title)
            .flex()
            .flex_col()
            .gap(theme.space(2.))
            .p(theme.space(3.))
            .font_family(theme.fonts.body.clone())
            .text_size(theme.text(14.))
            .text_color(colors.foreground)
            .on_key_down(move |event: &KeyDownEvent, window, cx| {
                let keystroke = &event.keystroke;
                let modifiers = keystroke.modifiers;
                if modifiers.control || modifiers.alt || modifiers.platform {
                    return;
                }
                // Days run toward the reading end, so the arrows swap in
                // right-to-left layouts.
                let key = UiTheme::read(cx)
                    .direction
                    .logical_key(keystroke.key.as_str());
                let handled =
                    key_state.update(cx, |state, cx| state.handle_key(key, modifiers.shift, cx));
                if !handled {
                    return;
                }
                cx.stop_propagation();
                match key {
                    "enter" | "space" => {
                        let picked = key_state.read(cx).selected;
                        if let (Some(date), Some(handler)) = (picked, &on_select) {
                            handler(date, window, cx);
                        }
                    }
                    "escape" => {
                        if let Some(handler) = &on_cancel {
                            handler(window, cx);
                        }
                    }
                    _ => {}
                }
            })
            .child(header)
            .child(div().flex().flex_col().child(weekdays).children(rows));
        apply_style(root, &self.style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{AppContext as _, TestAppContext, VisualTestContext, test::TestQueryExt as _};

    fn date(year: i32, month: u8, day: u8) -> CalendarDate {
        CalendarDate::new(year, month, day).unwrap()
    }

    #[test]
    fn dates_count_days_and_months() {
        assert_eq!(CalendarDate::from_days(0), date(1970, 1, 1));
        assert_eq!(date(2026, 3, 14).weekday(), Weekday::Saturday);
        assert_eq!(date(2024, 2, 28).add_days(1), date(2024, 2, 29));
        assert_eq!(date(2026, 1, 31).add_months(1), date(2026, 2, 28));
        assert_eq!(date(2026, 12, 15).add_months(1), date(2027, 1, 15));
        assert_eq!(date(2000, 3, 1).add_days(-1), date(2000, 2, 29));
        for days in [-800_000, -1, 0, 19_000, 2_000_000] {
            assert_eq!(CalendarDate::from_days(days).to_days(), days);
        }
        assert_eq!(
            CalendarDate::parse_iso("2026-03-14"),
            Some(date(2026, 3, 14))
        );
        assert_eq!(CalendarDate::parse_iso("2026-02-30"), None);
        assert_eq!(date(2026, 3, 4).to_string(), "2026-03-04");
        assert_eq!(date(2026, 3, 14).long_label(), "Saturday, March 14, 2026");
        assert!(CalendarDate::new(2026, 13, 1).is_none());
    }

    struct Picker {
        state: Entity<CalendarState>,
    }

    impl Render for Picker {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().p(gpui_kit::px(8.)).child(Calendar::new(&self.state))
        }
    }

    #[gpui_kit::test]
    fn the_keyboard_alone_picks_a_date(cx: &mut TestAppContext) {
        cx.update(super::super::theme::init);
        let events = Rc::new(std::cell::RefCell::new(Vec::new()));
        let sink = events.clone();
        let window = cx.add_window(move |_, cx| {
            let state = cx.new(|cx| {
                let mut state = CalendarState::new(cx);
                state.set_today(date(2026, 3, 14), cx);
                state.set_first_weekday(Weekday::Monday, cx);
                state.set_limits(None, Some(date(2026, 6, 30)), cx);
                state
            });
            cx.subscribe(&state, move |_, _, event: &CalendarEvent, _| {
                sink.borrow_mut().push(*event)
            })
            .detach();
            Picker { state }
        });
        let view = window.root(cx).unwrap();
        let mut cx = VisualTestContext::from_window(window.into(), cx);
        let state = cx.update(|_, cx| view.read(cx).state.clone());
        cx.update(|window, cx| {
            window.activate_window();
            state.update(cx, |state, cx| state.focus(window, cx));
            window.draw(cx).clear(cx);
        });
        cx.update(|window, _| {
            window.find_by_label("Saturday, March 14, 2026, today");
            window.find_by_label("Previous month");
        });

        // Right, Down, Home, End, then a month forward and back.
        cx.simulate_keystrokes("right down");
        cx.update(|_, cx| assert_eq!(state.read(cx).cursor(), date(2026, 3, 22)));
        cx.simulate_keystrokes("home");
        cx.update(|_, cx| assert_eq!(state.read(cx).cursor(), date(2026, 3, 16)));
        cx.simulate_keystrokes("end");
        cx.update(|_, cx| assert_eq!(state.read(cx).cursor(), date(2026, 3, 22)));
        cx.simulate_keystrokes("pagedown");
        cx.update(|_, cx| assert_eq!(state.read(cx).cursor(), date(2026, 4, 22)));
        cx.simulate_keystrokes("pageup left");
        cx.update(|_, cx| assert_eq!(state.read(cx).cursor(), date(2026, 3, 21)));
        // The limit stops the cursor.
        cx.simulate_keystrokes("shift-pagedown");
        cx.update(|_, cx| assert_eq!(state.read(cx).cursor(), date(2026, 6, 30)));
        cx.simulate_keystrokes("pageup enter");
        cx.update(|window, cx| {
            assert_eq!(state.read(cx).selected(), Some(date(2026, 5, 30)));
            window.draw(cx).clear(cx);
        });
        cx.update(|window, _| {
            let picked = window.find_by_label("Saturday, May 30, 2026");
            assert_eq!(picked.selected(), Some(true));
        });
        cx.simulate_keystrokes("escape");
        assert_eq!(
            events.borrow().as_slice(),
            [
                CalendarEvent::Select(date(2026, 5, 30)),
                CalendarEvent::Cancel
            ]
        );
    }

    #[gpui_kit::test]
    fn weeks_start_on_the_chosen_weekday(cx: &mut TestAppContext) {
        let window = cx.add_empty_window();
        window.update(|_, cx| {
            let state = cx.new(|cx| {
                let mut state = CalendarState::new(cx);
                state.set_cursor(date(2026, 3, 14), cx);
                state
            });
            let weeks = state.read(cx).weeks();
            assert_eq!(weeks.len(), 6);
            assert_eq!(
                weeks[0][0],
                date(2026, 3, 1),
                "March 2026 starts on a Sunday"
            );
            state.update(cx, |state, cx| state.set_first_weekday(Weekday::Monday, cx));
            assert_eq!(state.read(cx).weeks()[0][0], date(2026, 2, 23));
        });
    }
}
