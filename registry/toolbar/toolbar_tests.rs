use super::*;
use gpui::{
    Context, IntoElement, ParentElement as _, Render, TestAppContext, VisualTestContext, Window,
    div,
};
use std::{cell::RefCell, rc::Rc};

struct View {
    value: Rc<RefCell<SharedString>>,
    focus: Rc<RefCell<[bool; 3]>>,
    disabled: bool,
    vertical: bool,
}
impl Render for View {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let first = self.focus.clone();
        let input = self.focus.clone();
        let last = self.focus.clone();
        let value = self.value.clone();
        div()
            .w(px(400.))
            .child(
                toolbar(cx)
                    .id("tools")
                    .orientation(if self.vertical {
                        ToolbarOrientation::Vertical
                    } else {
                        ToolbarOrientation::Horizontal
                    })
                    .child(
                        toolbar_button(cx)
                            .id("first")
                            .child("First")
                            .style_with_state(move |state, base| {
                                first.borrow_mut()[0] = state.focused;
                                base.w(px(60.)).h(px(30.))
                            }),
                    )
                    .child(
                        toolbar_group(cx).id("group").disabled(self.disabled).child(
                            toolbar_input(cx)
                                .id("find")
                                .default_value("hello café")
                                .focusable_when_disabled(false)
                                .on_value_change(move |next| *value.borrow_mut() = next)
                                .style_with_state(move |state, base| {
                                    input.borrow_mut()[1] = state.input.focused;
                                    base.w(px(180.)).h(px(30.))
                                }),
                        ),
                    )
                    .child(
                        toolbar_button(cx)
                            .id("last")
                            .child("Last")
                            .style_with_state(move |state, base| {
                                last.borrow_mut()[2] = state.focused;
                                base.w(px(60.)).h(px(30.))
                            }),
                    ),
            )
            .child(super::super::input::Input::new("after"))
    }
}

#[gpui::test]
fn editing_history_and_word_selection_keep_roving_focus_and_disabled_cascade(
    cx: &mut TestAppContext,
) {
    cx.update(super::super::theme::init);
    let value = Rc::new(RefCell::new(SharedString::from("hello café")));
    let focus = Rc::new(RefCell::new([false; 3]));
    let window = cx.add_window({
        let value = value.clone();
        let focus = focus.clone();
        move |_, _| View {
            value,
            focus,
            disabled: false,
            vertical: false,
        }
    });
    let mut visual = VisualTestContext::from_window(window.into(), cx);
    let draw =
        |visual: &mut VisualTestContext| visual.update(|window, cx| window.draw(cx).clear(cx));
    visual.update(|window, _| window.activate_window());
    draw(&mut visual);
    visual.simulate_keystrokes("tab right");
    draw(&mut visual);
    assert_eq!(*focus.borrow(), [false, true, false]);
    // Roving entry selects all text, so typing replaces the initial phrase.
    visual.simulate_input("alpha beta");
    draw(&mut visual);
    assert_eq!(value.borrow().as_ref(), "alpha beta");
    for (keys, expected) in [
        ("cmd-z", "hello café"),
        ("cmd-shift-z", "alpha beta"),
        ("alt-backspace", "alpha "),
        ("cmd-z", "alpha beta"),
    ] {
        visual.simulate_keystrokes(keys);
        draw(&mut visual);
        assert_eq!(value.borrow().as_ref(), expected, "{keys}");
        assert_eq!(*focus.borrow(), [false, true, false]);
    }
    visual.simulate_keystrokes("alt-right alt-shift-left");
    visual.simulate_input("世界");
    draw(&mut visual);
    assert_eq!(value.borrow().as_ref(), "alpha 世界");
    visual.simulate_keystrokes("cmd-z");
    draw(&mut visual);
    // Modified arrows stay in the editor even at its boundary.
    visual.simulate_keystrokes("alt-left alt-left alt-left");
    draw(&mut visual);
    assert_eq!(*focus.borrow(), [false, true, false]);
    visual.simulate_keystrokes("left");
    draw(&mut visual);
    assert_eq!(*focus.borrow(), [true, false, false]);
    visual.simulate_keystrokes("right right");
    draw(&mut visual);
    assert_eq!(*focus.borrow(), [false, true, false]);
    visual.simulate_keystrokes("right");
    draw(&mut visual);
    assert_eq!(*focus.borrow(), [false, false, true]);
    // The toolbar has one tab stop, including when that stop is its editor.
    visual.simulate_keystrokes("left tab");
    draw(&mut visual);
    assert_eq!(*focus.borrow(), [false; 3]);
    visual.simulate_keystrokes("shift-tab");
    draw(&mut visual);
    assert_eq!(*focus.borrow(), [false, true, false]);
    window
        .update(cx, |view, _, cx| {
            view.disabled = true;
            cx.notify();
        })
        .unwrap();
    draw(&mut visual);
    visual.simulate_keystrokes("cmd-z alt-backspace");
    visual.simulate_input("ignored");
    draw(&mut visual);
    assert_eq!(value.borrow().as_ref(), "alpha beta");
    visual.simulate_keystrokes("right right right");
    draw(&mut visual);
    assert_ne!(*focus.borrow(), [false, true, false]);
    // Vertical arrows still traverse the composite; left/right continue editing.
    window
        .update(cx, |view, _, cx| {
            view.disabled = false;
            view.vertical = true;
            cx.notify();
        })
        .unwrap();
    draw(&mut visual);
    visual.simulate_keystrokes("up");
    draw(&mut visual);
    let focused = *focus.borrow();
    visual.simulate_keystrokes("down");
    draw(&mut visual);
    assert_ne!(*focus.borrow(), focused);
}
