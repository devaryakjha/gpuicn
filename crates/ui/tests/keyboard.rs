//! Real keystroke coverage for shared keyboard defaults.

use std::{cell::RefCell, rc::Rc};

use gpui_kit::{
    AppContext, Context, IntoElement, ParentElement, Render, Styled, TestAppContext,
    VisualTestContext, Window, div, px,
};
use gpuicn::{
    Button, UiTheme,
    input::{Input, InputEvent, InputState},
};

struct View(Rc<RefCell<Vec<String>>>, bool);

impl Render for View {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let first = self.0.clone();
        let input = self.0.clone();
        let last = self.0.clone();
        let editor = window
            .use_keyed_state("keyboard.input", cx, |window, cx| {
                let state = cx.new(|cx| InputState::new(window, cx).default_value("draft"));
                let subscription = cx.subscribe(&state, move |_, state, event, cx| {
                    if matches!(event, InputEvent::PressEnter { .. }) {
                        input.borrow_mut().push(state.read(cx).value().to_string());
                    }
                });
                (state, subscription)
            })
            .read(cx)
            .0
            .clone();
        div()
            .flex()
            .flex_col()
            .w(px(300.))
            .child(
                Button::new("first")
                    .child("First")
                    .on_click(move |_, _, _| first.borrow_mut().push("first".into())),
            )
            .child(
                Button::new("disabled")
                    .disabled(true)
                    .child("Disabled")
                    .on_click(|_, _, _| panic!("disabled button activated")),
            )
            .child(Input::new(&editor).disabled(self.1))
            .child(
                Button::new("last")
                    .child("Last")
                    .on_click(move |_, _, _| last.borrow_mut().push("last".into())),
            )
    }
}

#[gpui_kit::test]
fn tab_shift_tab_and_enter_work_across_controls(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpuicn::init(cx);
        UiTheme::set(cx, UiTheme::neutral_light());
    });
    let events = Rc::new(RefCell::new(Vec::new()));
    let window = cx.add_window({
        let events = events.clone();
        move |window, cx| {
            window.activate_window();
            window.refresh();
            let _ = cx;
            View(events, false)
        }
    });
    let mut visual = VisualTestContext::from_window(window.into(), cx);
    visual.run_until_parked();
    cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    for keys in [
        "tab",
        "enter",
        "tab",
        "enter",
        "tab",
        "enter",
        "shift-tab",
        "enter",
    ] {
        visual.simulate_keystrokes(keys);
        visual.update(|window, cx| {
            window.dispatch_event(
                gpui_kit::PlatformInput::KeyUp(gpui_kit::KeyUpEvent {
                    keystroke: gpui_kit::Keystroke::parse(keys).unwrap(),
                }),
                cx,
            );
        });
        visual.run_until_parked();
        cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
            .unwrap();
    }
    assert_eq!(*events.borrow(), ["first", "draft", "last", "draft"]);
    // A request may disable the field while it still has keyboard focus.
    window
        .update(cx, |view, _, cx| {
            view.1 = true;
            cx.notify();
        })
        .unwrap();
    cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    visual.simulate_keystrokes("enter");
    assert_eq!(*events.borrow(), ["first", "draft", "last", "draft"]);
}
