//! Nova Date Picker: an outline trigger that opens a `Calendar` popover.
//!
//! Visual source: shadcn/ui 4.19.0's date picker example (a Popover with a
//! Calendar), Nova style. GPUI Kit's `Popover` supplies the open state,
//! outside-click and Escape dismissal and focus return; focus moves into the
//! calendar grid when it opens, and picking a date closes it.

use gpui_icons::{LucideIcon, lucide};
use gpui_kit::base::Popover;
use gpui_kit::{
    App, ElementId, Entity, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    SharedString, StyleRefinement, Styled, Window, div, prelude::FluentBuilder as _,
};

use super::calendar::{Calendar, CalendarDate, CalendarState};
use super::popover::popover_trigger;
use super::theme::{UiText, UiTheme, apply_style, locale, primitives::surface};

type Format = std::rc::Rc<dyn Fn(CalendarDate) -> SharedString>;

/// A button showing the picked date that opens a calendar under it.
///
/// The date lives in the `CalendarState` you pass; subscribe to its
/// `CalendarEvent::Select` to hear about picks.
#[derive(IntoElement)]
pub struct DatePicker {
    id: ElementId,
    state: Entity<CalendarState>,
    placeholder: Option<SharedString>,
    label: Option<SharedString>,
    format: Option<Format>,
    disabled: bool,
    style: StyleRefinement,
}

impl DatePicker {
    /// Creates a date picker over retained calendar state.
    pub fn new(id: impl Into<ElementId>, state: &Entity<CalendarState>) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            placeholder: None,
            label: None,
            format: None,
            disabled: false,
            style: StyleRefinement::default(),
        }
    }

    /// Sets the text shown before a date is picked.
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Sets the field's accessible name, such as "Due date"; the picked
    /// date is read after it.
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Formats the picked date on the trigger; the default is "Mar 14, 2026".
    pub fn format(mut self, format: impl Fn(CalendarDate) -> SharedString + 'static) -> Self {
        self.format = Some(std::rc::Rc::new(format));
        self
    }

    /// Disables the trigger.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Styled for DatePicker {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

/// The popover surface that holds a calendar or a calendar and a time field.
#[doc(hidden)]
pub fn calendar_popup(theme: &UiTheme) -> gpui_kit::Div {
    surface::popover(div(), theme)
        .occlude()
        .flex()
        .flex_col()
        .overflow_hidden()
}

impl RenderOnce for DatePicker {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let selected = self.state.read(cx).selected();
        let text = selected.map(|date| match &self.format {
            Some(format) => format(date),
            None => locale::text(cx, date.short_text()),
        });
        let placeholder = self
            .placeholder
            .clone()
            .unwrap_or_else(|| locale::text(cx, UiText::PickDate));
        let spoken = selected.map(|date| locale::text(cx, date.long_text()));
        let label: SharedString = match (&self.label, spoken) {
            (Some(label), Some(date)) => format!("{label}, {date}").into(),
            (Some(label), None) => label.clone(),
            (None, Some(date)) => date,
            (None, None) => placeholder.clone(),
        };
        let trigger = popover_trigger((self.id.clone(), "trigger"), cx)
            .disabled(self.disabled)
            .accessibility_label(label)
            .justify_start()
            .gap(theme.space(2.))
            .font_weight(gpui_kit::FontWeight::NORMAL)
            .child(
                lucide(LucideIcon::Calendar)
                    .size(theme.space(4.))
                    .text_color(colors.muted_foreground),
            )
            .child(
                div()
                    .when(text.is_none(), |text| {
                        text.text_color(colors.muted_foreground)
                    })
                    .child(text.unwrap_or(placeholder)),
            );
        let state = self.state.clone();
        let focus = state.read(cx).focus.clone();
        let popover = Popover::new(self.id)
            .anchor(theme.direction.anchor(gpui_kit::Anchor::TopLeft))
            .track_focus(&focus)
            .offset(theme.space(1.))
            .trigger(trigger)
            .content(move |_, _, cx| {
                let popover = cx.entity();
                let dismiss = popover.clone();
                calendar_popup(&theme).child(
                    Calendar::new(&state)
                        .on_select(move |_, window, cx| {
                            popover.update(cx, |popover, cx| popover.dismiss(window, cx))
                        })
                        .on_cancel(move |window, cx| {
                            dismiss.update(cx, |popover, cx| popover.dismiss(window, cx))
                        }),
                )
            });
        apply_style(popover, &self.style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{
        AppContext as _, Context, Render, TestAppContext, VisualTestContext,
        test::TestQueryExt as _,
    };

    struct Form {
        state: Entity<CalendarState>,
    }

    impl Render for Form {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .p(gpui_kit::px(8.))
                .child(DatePicker::new("due", &self.state).aria_label("Due date"))
        }
    }

    #[gpui_kit::test]
    fn picking_by_keyboard_fills_the_trigger_and_closes(cx: &mut TestAppContext) {
        cx.update(super::super::theme::init);
        let window = cx.add_window(|_, cx| Form {
            state: cx.new(|cx| {
                let mut state = CalendarState::new(cx);
                let today = CalendarDate::new(2026, 3, 14).unwrap();
                state.set_today(today, cx);
                state
            }),
        });
        let view = window.root(cx).unwrap();
        let mut cx = VisualTestContext::from_window(window.into(), cx);
        cx.update(|window, cx| {
            window.activate_window();
            window.draw(cx).clear(cx);
        });
        let trigger = cx.update(|window, _| window.find_by_label("Due date"));
        cx.simulate_click(trigger.bounds().center(), gpui_kit::Modifiers::default());
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let state = cx.update(|_, cx| view.read(cx).state.clone());
        cx.update(|window, cx| {
            assert!(
                state.read(cx).focus.is_focused(window),
                "the grid takes focus"
            );
            window.find_by_label("March 2026");
        });
        cx.simulate_keystrokes("right right enter");
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.update(|window, cx| {
            assert_eq!(state.read(cx).selected(), CalendarDate::new(2026, 3, 16));
            window.find_by_label("Due date, Monday, March 16, 2026");
            assert!(window.try_find_by_label("March 2026").is_none(), "closed");
        });
    }
}
