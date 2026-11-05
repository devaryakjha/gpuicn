//! Nova Date Time Picker: a trigger that opens a `Calendar` and a time field
//! in one popover, for scheduling.
//!
//! Visual source: shadcn/ui's date and time picker example, Nova style.
//! GPUI Kit's `Popover` owns opening, dismissal and focus return. The time
//! field is one tab stop with hour, minute and (on a 12-hour clock) AM/PM
//! segments: Left and Right pick a segment, Up and Down step it, digits type
//! it, and A or P sets the half of the day.

use gpui_icons::{LucideIcon, lucide};
use gpui_kit::base::Popover;
use gpui_kit::base::TestSupportExt as _;
use gpui_kit::{
    App, AppContext as _, Context, ElementId, Entity, EventEmitter, FocusHandle, Focusable,
    FontWeight, InteractiveElement as _, IntoElement, KeyDownEvent, ParentElement as _, RenderOnce,
    Role, SharedString, StatefulInteractiveElement as _, StyleRefinement, Styled, Subscription,
    Window, div, prelude::FluentBuilder as _,
};

use super::button::{Button, ButtonSize};
use super::calendar::{Calendar, CalendarDate, CalendarEvent, CalendarState};
use super::date_picker::calendar_popup;
use super::popover::popover_trigger;
use super::theme::{
    UiText, UiTheme, apply_style,
    locale::{self, DirectionalStyled as _},
    primitives::{control, focus, surface},
};

/// A time of day to the minute, on a 24-hour clock.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CalendarTime {
    hour: u8,
    minute: u8,
}

impl CalendarTime {
    /// The time, or `None` past 23:59.
    pub fn new(hour: u8, minute: u8) -> Option<Self> {
        (hour < 24 && minute < 60).then_some(Self { hour, minute })
    }

    /// The hour, 0 to 23.
    pub fn hour(self) -> u8 {
        self.hour
    }

    /// The minute, 0 to 59.
    pub fn minute(self) -> u8 {
        self.minute
    }

    /// The time on `cycle`'s clock, such as "9:30 AM" or "09:30".
    pub fn label(self, cycle: HourCycle) -> String {
        match cycle {
            HourCycle::H23 => format!("{:02}:{:02}", self.hour, self.minute),
            HourCycle::H12 => {
                let hour = match self.hour % 12 {
                    0 => 12,
                    hour => hour,
                };
                let half = if self.hour < 12 { "AM" } else { "PM" };
                format!("{hour}:{:02} {half}", self.minute)
            }
        }
    }

    /// The time as [`CalendarTime::label`] writes it, with the application's
    /// AM and PM.
    pub fn label_in(self, cycle: HourCycle, cx: &App) -> String {
        match cycle {
            HourCycle::H23 => self.label(cycle),
            HourCycle::H12 => {
                let hour = match self.hour % 12 {
                    0 => 12,
                    hour => hour,
                };
                let half = locale::text(
                    cx,
                    if self.hour < 12 {
                        UiText::Am
                    } else {
                        UiText::Pm
                    },
                );
                format!("{hour}:{:02} {half}", self.minute)
            }
        }
    }
}

/// How the time field counts the hours of a day.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum HourCycle {
    /// 00 to 23.
    #[default]
    H23,
    /// 12, 1 to 11, with AM and PM.
    H12,
}

impl HourCycle {
    /// The usual clock for a BCP 47 locale such as `en-US` (12-hour) or
    /// `en-GB` (24-hour).
    pub fn for_locale(locale: &str) -> Self {
        let locale = locale.replace('_', "-").to_ascii_lowercase();
        let region = locale.split('-').nth(1).unwrap_or("");
        let language = locale.split('-').next().unwrap_or("");
        let twelve_hour_regions = [
            "us", "ca", "au", "nz", "in", "ph", "pk", "eg", "sa", "co", "mx", "bd", "my",
        ];
        if twelve_hour_regions.contains(&region)
            || (region.is_empty() && ["en", "hi", "ar", "bn", "ko", "ur"].contains(&language))
        {
            Self::H12
        } else {
            Self::H23
        }
    }
}

/// A segment of the time field.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TimeSegment {
    /// The hour.
    #[default]
    Hour,
    /// The minute.
    Minute,
    /// AM or PM, on a 12-hour clock.
    Period,
}

/// Notifications from a date and time picker.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DateTimeEvent {
    /// The date or the time changed.
    Change {
        /// The picked date, if any.
        date: Option<CalendarDate>,
        /// The time.
        time: CalendarTime,
    },
}

/// A date, a time and the limits on both.
pub struct DateTimeState {
    calendar: Entity<CalendarState>,
    time: CalendarTime,
    cycle: HourCycle,
    segment: TimeSegment,
    typed: Option<u8>,
    min: Option<(CalendarDate, CalendarTime)>,
    max: Option<(CalendarDate, CalendarTime)>,
    focus: FocusHandle,
    _subscription: Subscription,
}

impl EventEmitter<DateTimeEvent> for DateTimeState {}

impl Focusable for DateTimeState {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl DateTimeState {
    /// Creates a picker at 09:00 with no date and a 24-hour clock.
    pub fn new(cx: &mut Context<Self>) -> Self {
        let calendar = cx.new(CalendarState::new);
        let subscription = cx.subscribe(&calendar, |this, _, event, cx| {
            if let CalendarEvent::Select(_) = event {
                this.time = this.bounded(this.time, cx);
                this.changed(cx);
            }
        });
        Self {
            calendar,
            time: CalendarTime { hour: 9, minute: 0 },
            cycle: HourCycle::H23,
            segment: TimeSegment::Hour,
            typed: None,
            min: None,
            max: None,
            focus: cx.focus_handle(),
            _subscription: subscription,
        }
    }

    /// The calendar inside, for today, the first weekday and the shown month.
    pub fn calendar(&self) -> &Entity<CalendarState> {
        &self.calendar
    }

    /// The picked date.
    pub fn date(&self, cx: &App) -> Option<CalendarDate> {
        self.calendar.read(cx).selected()
    }

    /// The time.
    pub fn time(&self) -> CalendarTime {
        self.time
    }

    /// Sets the date and time.
    pub fn set_value(
        &mut self,
        date: Option<CalendarDate>,
        time: CalendarTime,
        cx: &mut Context<Self>,
    ) {
        self.calendar
            .update(cx, |calendar, cx| calendar.set_selected(date, cx));
        self.time = self.bounded(time, cx);
        cx.notify();
    }

    /// The clock the time field shows.
    pub fn hour_cycle(&self) -> HourCycle {
        self.cycle
    }

    /// Sets the clock, for example `HourCycle::for_locale("en-US")`.
    pub fn set_hour_cycle(&mut self, cycle: HourCycle, cx: &mut Context<Self>) {
        self.cycle = cycle;
        if cycle == HourCycle::H23 && self.segment == TimeSegment::Period {
            self.segment = TimeSegment::Minute;
        }
        cx.notify();
    }

    /// Limits the date and time to the range from `min` to `max`, such as
    /// "not in the past" for a scheduled message.
    pub fn set_limits(
        &mut self,
        min: Option<(CalendarDate, CalendarTime)>,
        max: Option<(CalendarDate, CalendarTime)>,
        cx: &mut Context<Self>,
    ) {
        self.min = min;
        self.max = max;
        self.calendar.update(cx, |calendar, cx| {
            calendar.set_limits(min.map(|(date, _)| date), max.map(|(date, _)| date), cx)
        });
        self.time = self.bounded(self.time, cx);
        cx.notify();
    }

    /// The segment the keyboard edits.
    pub fn segment(&self) -> TimeSegment {
        self.segment
    }

    /// Moves keyboard focus to the time field.
    pub fn focus_time(&self, window: &mut Window, cx: &mut App) {
        self.focus.focus(window, cx);
    }

    fn changed(&mut self, cx: &mut Context<Self>) {
        cx.emit(DateTimeEvent::Change {
            date: self.calendar.read(cx).selected(),
            time: self.time,
        });
        cx.notify();
    }

    fn bounded(&self, time: CalendarTime, cx: &App) -> CalendarTime {
        let date = self.calendar.read(cx).selected();
        let mut time = time;
        if let (Some(date), Some((min_date, min_time))) = (date, self.min)
            && date == min_date
        {
            time = time.max(min_time);
        }
        if let (Some(date), Some((max_date, max_time))) = (date, self.max)
            && date == max_date
        {
            time = time.min(max_time);
        }
        time
    }

    fn set_time(&mut self, time: CalendarTime, cx: &mut Context<Self>) {
        let time = self.bounded(time, cx);
        if time != self.time {
            self.time = time;
            self.changed(cx);
        }
    }

    fn segments(&self) -> &'static [TimeSegment] {
        match self.cycle {
            HourCycle::H23 => &[TimeSegment::Hour, TimeSegment::Minute],
            HourCycle::H12 => &[TimeSegment::Hour, TimeSegment::Minute, TimeSegment::Period],
        }
    }

    fn step(&mut self, delta: i32, cx: &mut Context<Self>) {
        let CalendarTime { hour, minute } = self.time;
        let time = match self.segment {
            TimeSegment::Hour => CalendarTime {
                hour: (i32::from(hour) + delta).rem_euclid(24) as u8,
                minute,
            },
            TimeSegment::Minute => CalendarTime {
                hour,
                minute: (i32::from(minute) + delta).rem_euclid(60) as u8,
            },
            TimeSegment::Period => CalendarTime {
                hour: (hour + 12) % 24,
                minute,
            },
        };
        self.set_time(time, cx);
    }

    fn type_digit(&mut self, digit: u8, cx: &mut Context<Self>) {
        let (limit, segment) = match self.segment {
            TimeSegment::Hour => (
                if self.cycle == HourCycle::H12 { 12 } else { 23 },
                TimeSegment::Hour,
            ),
            TimeSegment::Minute => (59, TimeSegment::Minute),
            TimeSegment::Period => return,
        };
        let value = match self.typed {
            Some(first) if first * 10 + digit <= limit => first * 10 + digit,
            _ => digit,
        };
        // A second digit, or a first that no second could follow, moves on.
        let complete = self.typed.is_some() || value * 10 > limit;
        self.typed = (!complete).then_some(value);
        let CalendarTime { hour, minute } = self.time;
        let time = match segment {
            TimeSegment::Hour if self.cycle == HourCycle::H12 => {
                let afternoon = hour >= 12;
                let hour = value % 12 + if afternoon { 12 } else { 0 };
                CalendarTime { hour, minute }
            }
            TimeSegment::Hour => CalendarTime {
                hour: value,
                minute,
            },
            _ => CalendarTime {
                hour,
                minute: value,
            },
        };
        self.set_time(time, cx);
        if complete {
            self.move_segment(1, cx);
        }
    }

    fn move_segment(&mut self, delta: i32, cx: &mut Context<Self>) {
        let segments = self.segments();
        let index = segments
            .iter()
            .position(|segment| *segment == self.segment)
            .unwrap_or(0) as i32;
        let next = (index + delta).clamp(0, segments.len() as i32 - 1) as usize;
        self.segment = segments[next];
        self.typed = None;
        cx.notify();
    }

    /// Handles a time-field key. Returns whether it was used.
    pub fn handle_key(&mut self, key: &str, cx: &mut Context<Self>) -> bool {
        match key {
            "up" => self.step(1, cx),
            "down" => self.step(-1, cx),
            "left" => self.move_segment(-1, cx),
            "right" => self.move_segment(1, cx),
            "a" | "p" if self.cycle == HourCycle::H12 => {
                let afternoon = key == "p";
                let CalendarTime { hour, minute } = self.time;
                let hour = hour % 12 + if afternoon { 12 } else { 0 };
                self.set_time(CalendarTime { hour, minute }, cx);
            }
            digit if digit.len() == 1 && digit.as_bytes()[0].is_ascii_digit() => {
                self.type_digit(digit.as_bytes()[0] - b'0', cx)
            }
            _ => return false,
        }
        true
    }

    /// The date and time as the trigger shows them, such as
    /// "Mar 14, 2026, 9:30 AM".
    pub fn label(&self, cx: &App) -> Option<String> {
        self.date(cx).map(|date| {
            format!(
                "{}, {}",
                locale::text(cx, date.short_text()),
                self.time.label_in(self.cycle, cx)
            )
        })
    }
}

/// The hour, minute and AM/PM segments, one tab stop.
#[derive(IntoElement)]
pub struct TimeField {
    state: Entity<DateTimeState>,
}

impl TimeField {
    /// Creates the time field of a date and time picker.
    pub fn new(state: &Entity<DateTimeState>) -> Self {
        Self {
            state: state.clone(),
        }
    }
}

impl RenderOnce for TimeField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let state = self.state.read(cx);
        let focused = state.focus.is_focused(window);
        let time = state.time;
        let cycle = state.cycle;
        let current = state.segment;
        let id = ElementId::from(("time-field", self.state.entity_id()));
        let display_hour = match cycle {
            HourCycle::H23 => format!("{:02}", time.hour),
            HourCycle::H12 => format!(
                "{:02}",
                if time.hour.is_multiple_of(12) {
                    12
                } else {
                    time.hour % 12
                }
            ),
        };
        let segment = |kind: TimeSegment, text: String, key: &'static str, name: SharedString| {
            let active = focused && kind == current;
            let state = self.state.clone();
            div()
                .id((id.clone(), key))
                .test_support()
                .role(Role::SpinButton)
                .aria_label(name)
                .aria_value(SharedString::from(text.clone()))
                .when(kind == current, |segment| segment.aria_active_descendant())
                .px(theme.space(1.))
                .rounded(theme.radius.sm)
                .when(active, |segment| {
                    segment
                        .bg(colors.accent)
                        .text_color(colors.accent_foreground)
                })
                .on_mouse_down(gpui_kit::MouseButton::Left, move |_, window, cx| {
                    state.update(cx, |state, cx| {
                        state.segment = kind;
                        state.typed = None;
                        state.focus.focus(window, cx);
                        cx.notify();
                    })
                })
                .child(text)
        };
        let key_state = self.state.clone();
        let label = time.label_in(cycle, cx);
        let field_label = locale::text(cx, UiText::TimeField(label.into()));
        let hour_label = locale::text(cx, UiText::Hour);
        let minute_label = locale::text(cx, UiText::Minute);
        let period_label = locale::text(cx, UiText::DayPeriod);
        let period = locale::text(
            cx,
            if time.hour < 12 {
                UiText::Am
            } else {
                UiText::Pm
            },
        );
        focus::shown(control::frame(div(), &theme, false), &theme, focused)
            .id(id.clone())
            .test_support()
            .key_context("TimeField")
            .track_focus(&state.focus)
            .role(Role::Group)
            .aria_label(field_label)
            .h(theme.space(8.))
            .px(theme.space(2.))
            .flex()
            .items_center()
            .gap(theme.space(0.5))
            .font_family(theme.fonts.mono.clone())
            .when_some(
                focused.then(|| theme.opaque(theme.control().background)),
                |root, fill| root.bg(fill),
            )
            .on_key_down(move |event: &KeyDownEvent, _, cx| {
                let modifiers = event.keystroke.modifiers;
                if modifiers.control || modifiers.alt || modifiers.platform {
                    return;
                }
                let key = event.keystroke.key.clone();
                if key_state.update(cx, |state, cx| state.handle_key(&key, cx)) {
                    cx.stop_propagation();
                }
            })
            .child(segment(TimeSegment::Hour, display_hour, "Hour", hour_label))
            .child(div().text_color(colors.muted_foreground).child(":"))
            .child(segment(
                TimeSegment::Minute,
                format!("{:02}", time.minute),
                "Minute",
                minute_label,
            ))
            .when(cycle == HourCycle::H12, |root| {
                root.child(segment(
                    TimeSegment::Period,
                    period.to_string(),
                    "AM or PM",
                    period_label.clone(),
                ))
            })
    }
}

/// A trigger showing the date and time that opens a calendar with a time
/// field and a Done button.
///
/// Picking a day moves focus to the time field; Done, Escape or a click
/// outside closes the popover. Subscribe to `DateTimeEvent` on the state.
#[derive(IntoElement)]
pub struct DateTimePicker {
    id: ElementId,
    state: Entity<DateTimeState>,
    placeholder: Option<SharedString>,
    label: Option<SharedString>,
    disabled: bool,
    style: StyleRefinement,
}

impl DateTimePicker {
    /// Creates a date and time picker over retained state.
    pub fn new(id: impl Into<ElementId>, state: &Entity<DateTimeState>) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            placeholder: None,
            label: None,
            disabled: false,
            style: StyleRefinement::default(),
        }
    }

    /// Sets the text shown before a date is picked.
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Sets the field's accessible name, such as "Send at".
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Disables the trigger.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Styled for DateTimePicker {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for DateTimePicker {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let text = self.state.read(cx).label(cx);
        let placeholder = self
            .placeholder
            .clone()
            .unwrap_or_else(|| locale::text(cx, UiText::PickDateTime));
        let done = locale::text(cx, UiText::Done);
        let dir = theme.direction;
        let label: SharedString = match (&self.label, &text) {
            (Some(label), Some(text)) => format!("{label}, {text}").into(),
            (Some(label), None) => label.clone(),
            (None, Some(text)) => text.clone().into(),
            (None, None) => placeholder.clone(),
        };
        let trigger = popover_trigger((self.id.clone(), "trigger"), cx)
            .disabled(self.disabled)
            .accessibility_label(label)
            .justify_start()
            .gap(theme.space(2.))
            .font_weight(FontWeight::NORMAL)
            .child(
                lucide(LucideIcon::CalendarClock)
                    .size(theme.space(4.))
                    .text_color(colors.muted_foreground),
            )
            .child(
                div()
                    .when(text.is_none(), |text| {
                        text.text_color(colors.muted_foreground)
                    })
                    .child(text.unwrap_or_else(|| placeholder.to_string())),
            );
        let state = self.state.clone();
        let calendar = state.read(cx).calendar.clone();
        let calendar_focus = calendar.read(cx).focus.clone();
        let done_id = (self.id.clone(), "done");
        let popover = Popover::new(self.id)
            .anchor(dir.anchor(gpui_kit::Anchor::TopLeft))
            .track_focus(&calendar_focus)
            .offset(theme.space(1.))
            .trigger(trigger)
            .content(move |_, _, cx| {
                let popover = cx.entity();
                let dismiss = popover.clone();
                let to_time = state.clone();
                calendar_popup(&theme)
                    .child(
                        Calendar::new(&calendar)
                            .on_select(move |_, window, cx| {
                                to_time.update(cx, |state, cx| state.focus_time(window, cx))
                            })
                            .on_cancel(move |window, cx| {
                                dismiss.update(cx, |popover, cx| popover.dismiss(window, cx))
                            }),
                    )
                    .child(surface::divider(div(), &theme, gpui_kit::Axis::Horizontal))
                    .child(
                        div()
                            .flex_row_in(dir)
                            .items_center()
                            .gap(theme.space(2.))
                            .p(theme.space(3.))
                            .child(
                                lucide(LucideIcon::Clock)
                                    .size(theme.space(4.))
                                    .text_color(colors.muted_foreground),
                            )
                            .child(div().flex_1().child(TimeField::new(&state)))
                            .child(
                                Button::new(done_id)
                                    .size(ButtonSize::Sm)
                                    .label(done.clone())
                                    .on_click(move |_, window, cx| {
                                        popover
                                            .update(cx, |popover, cx| popover.dismiss(window, cx))
                                    }),
                            ),
                    )
            });
        apply_style(popover, &self.style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{Render, TestAppContext, VisualTestContext, test::TestQueryExt as _};

    #[test]
    fn clocks_follow_the_locale() {
        assert_eq!(HourCycle::for_locale("en-US"), HourCycle::H12);
        assert_eq!(HourCycle::for_locale("en_GB"), HourCycle::H23);
        assert_eq!(HourCycle::for_locale("de"), HourCycle::H23);
        let time = CalendarTime::new(0, 5).unwrap();
        assert_eq!(time.label(HourCycle::H12), "12:05 AM");
        assert_eq!(time.label(HourCycle::H23), "00:05");
        assert!(CalendarTime::new(24, 0).is_none());
    }

    struct Schedule {
        state: Entity<DateTimeState>,
    }

    impl Render for Schedule {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .p(gpui_kit::px(8.))
                .child(DateTimePicker::new("send-at", &self.state).aria_label("Send at"))
        }
    }

    #[gpui_kit::test]
    fn a_date_and_time_are_entered_by_keyboard(cx: &mut TestAppContext) {
        cx.update(super::super::theme::init);
        let window = cx.add_window(|_, cx| Schedule {
            state: cx.new(|cx| {
                let mut state = DateTimeState::new(cx);
                state.set_hour_cycle(HourCycle::H12, cx);
                let today = CalendarDate::new(2026, 3, 14).unwrap();
                state
                    .calendar
                    .update(cx, |calendar, cx| calendar.set_today(today, cx));
                state.set_limits(Some((today, CalendarTime::new(13, 0).unwrap())), None, cx);
                state
            }),
        });
        let view = window.root(cx).unwrap();
        let mut cx = VisualTestContext::from_window(window.into(), cx);
        cx.update(|window, cx| {
            window.activate_window();
            window.draw(cx).clear(cx);
        });
        let trigger = cx.update(|window, _| window.find_by_label("Send at"));
        cx.simulate_click(trigger.bounds().center(), gpui_kit::Modifiers::default());
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let state = cx.update(|_, cx| view.read(cx).state.clone());
        // Today is picked; the earliest time today is 1 PM.
        cx.simulate_keystrokes("enter");
        cx.update(|window, cx| {
            assert!(
                state.read(cx).focus.is_focused(window),
                "focus moves to the time"
            );
            assert_eq!(state.read(cx).time(), CalendarTime::new(13, 0).unwrap());
            window.draw(cx).clear(cx);
        });
        // 4, 5 types 4 PM... then 45 minutes.
        cx.simulate_keystrokes("4 4 5");
        cx.update(|_, cx| {
            assert_eq!(state.read(cx).time(), CalendarTime::new(16, 45).unwrap());
            assert_eq!(
                state.read(cx).label(cx).as_deref(),
                Some("Mar 14, 2026, 4:45 PM")
            );
        });
        cx.simulate_keystrokes("right a");
        cx.update(|_, cx| {
            // AM would be before the limit.
            assert_eq!(state.read(cx).time(), CalendarTime::new(13, 0).unwrap());
        });
        cx.simulate_keystrokes("left left up");
        cx.update(|_, cx| assert_eq!(state.read(cx).time(), CalendarTime::new(14, 0).unwrap()));
    }
}
