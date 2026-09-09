//! Keyboard checks for Kit-based controls.
use gpui_kit::{
    AppContext, Context, Entity, Focusable as _, IntoElement, ParentElement as _, Render, Styled,
    TestAppContext, Window, div, px,
};
use gpuicn::{
    input::{Input, InputState},
    toolbar::{Toolbar, ToolbarButton},
};

struct EditorToolbar {
    input: Entity<InputState>,
    after: Entity<InputState>,
}
impl Render for EditorToolbar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w(px(400.))
            .child(
                Toolbar::new("tools", "Formatting", cx)
                    .button(ToolbarButton::new("first", "First", cx).child("First"))
                    .button(
                        ToolbarButton::new("disabled", "Disabled", cx)
                            .disabled(true)
                            .child("Disabled"),
                    )
                    .input(&self.input, "Find", false)
                    .button(ToolbarButton::new("last", "Last", cx).child("Last")),
            )
            .child(Input::new(&self.after).aria_label("After"))
    }
}
#[gpui_kit::test]
fn toolbar_keeps_editing_and_one_tab_stop(cx: &mut TestAppContext) {
    cx.update(gpuicn::init);
    let (view, cx) = cx.add_window_view(|window, cx| EditorToolbar {
        input: cx.new(|cx| InputState::new(window, cx).default_value("hello café")),
        after: cx.new(|cx| InputState::new(window, cx)),
    });
    cx.update(|window, cx| {
        window.activate_window();
        window.draw(cx).clear(cx);
    });
    cx.simulate_keystrokes("tab right");
    cx.update(|window, cx| {
        assert!(
            view.read(cx)
                .input
                .read(cx)
                .focus_handle(cx)
                .is_focused(window)
        );
    });
    cx.simulate_input("alpha beta");
    cx.update(|_, cx| assert_eq!(view.read(cx).input.read(cx).value().as_ref(), "alpha beta"));
    cx.simulate_keystrokes("cmd-z cmd-z");
    cx.update(|_, cx| assert_eq!(view.read(cx).input.read(cx).value().as_ref(), "hello café"));
    cx.simulate_keystrokes("cmd-shift-z cmd-shift-z alt-left");
    cx.update(|window, cx| {
        assert!(
            view.read(cx)
                .input
                .read(cx)
                .focus_handle(cx)
                .is_focused(window)
        )
    });
    cx.simulate_keystrokes("cmd-left left");
    cx.update(|window, cx| {
        assert!(
            !view
                .read(cx)
                .input
                .read(cx)
                .focus_handle(cx)
                .is_focused(window)
        )
    });
    cx.simulate_keystrokes("right tab");
    cx.update(|window, cx| {
        assert!(
            view.read(cx)
                .after
                .read(cx)
                .focus_handle(cx)
                .is_focused(window)
        )
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.simulate_keystrokes("shift-tab");
    cx.update(|window, cx| {
        assert!(
            view.read(cx)
                .input
                .read(cx)
                .focus_handle(cx)
                .is_focused(window)
        )
    });
}

struct Choices {
    selected: gpui_kit::SharedString,
    radio: bool,
}
impl Render for Choices {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity().downgrade();
        let change = move |value, cx: &mut gpui_kit::App| {
            view.update(cx, |this, cx| {
                this.selected = value;
                cx.notify();
            })
            .unwrap();
        };
        if self.radio {
            use gpuicn::radio_group::{RadioGroup, RadioItem};
            RadioGroup::new("choices")
                .value(self.selected.clone())
                .item(RadioItem::new("a", "a").label("Alpha"))
                .item(RadioItem::new("b", "b").label("Unavailable").disabled(true))
                .item(RadioItem::new("c", "c").label("Charlie"))
                .on_change(move |value, _, cx| change(value, cx))
                .into_any_element()
        } else {
            use gpuicn::tabs::{Tab, Tabs};
            Tabs::new("choices")
                .selected(self.selected.clone())
                .item(Tab::new("a", "a", "Alpha"))
                .item(Tab::new("b", "b", "Unavailable").disabled(true))
                .item(Tab::new("c", "c", "Charlie"))
                .on_change(move |value, _, cx| change(value, cx))
                .into_any_element()
        }
    }
}
#[gpui_kit::test]
fn choice_arrows_skip_disabled_items_and_select(cx: &mut TestAppContext) {
    cx.update(gpuicn::init);
    for radio in [false, true] {
        let (view, cx) = cx.add_window_view(move |_, _| Choices {
            selected: "a".into(),
            radio,
        });
        cx.update(|window, cx| {
            window.activate_window();
            window.draw(cx).clear(cx);
        });
        cx.simulate_keystrokes("tab right");
        cx.update(|_, cx| assert_eq!(view.read(cx).selected.as_ref(), "c"));
        cx.simulate_keystrokes("right");
        cx.update(|_, cx| assert_eq!(view.read(cx).selected.as_ref(), "a"));
        cx.simulate_keystrokes("end");
        cx.update(|_, cx| assert_eq!(view.read(cx).selected.as_ref(), "c"));
        cx.simulate_keystrokes("home");
        cx.update(|_, cx| assert_eq!(view.read(cx).selected.as_ref(), "a"));
    }
}

struct SliderView {
    state: Entity<gpuicn::slider::SliderState>,
    disabled: bool,
}
impl Render for SliderView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(300.)).child(
            gpuicn::slider::Slider::new(&self.state)
                .disabled(self.disabled)
                .aria_label("Volume"),
        )
    }
}
#[gpui_kit::test]
fn slider_keys_respect_range_and_disabled_state(cx: &mut TestAppContext) {
    cx.update(gpuicn::init);
    let (view, cx) = cx.add_window_view(|_, cx| SliderView {
        state: cx.new(|_| {
            gpuicn::slider::SliderState::new()
                .min(10.)
                .max(20.)
                .step(2.)
                .default_value(14.)
        }),
        disabled: false,
    });
    cx.update(|window, cx| {
        window.activate_window();
        window.draw(cx).clear(cx);
    });
    cx.simulate_keystrokes("tab right");
    cx.update(|_, cx| assert_eq!(view.read(cx).state.read(cx).value().end(), 16.));
    cx.simulate_keystrokes("end right");
    cx.update(|_, cx| assert_eq!(view.read(cx).state.read(cx).value().end(), 20.));
    cx.simulate_keystrokes("home left");
    cx.update(|_, cx| assert_eq!(view.read(cx).state.read(cx).value().end(), 10.));
    view.update(cx, |this, cx| {
        this.disabled = true;
        cx.notify();
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.simulate_keystrokes("right end");
    cx.update(|_, cx| assert_eq!(view.read(cx).state.read(cx).value().end(), 10.));
}

struct ModalView {
    handle: gpuicn::dialog::DialogHandle,
    outside: Entity<InputState>,
    first: Entity<InputState>,
    last: Entity<InputState>,
}
impl Render for ModalView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let popup = gpuicn::dialog::dialog_popup("popup", "Edit profile", cx)
            .child(Input::new(&self.first).aria_label("First"))
            .child(Input::new(&self.last).aria_label("Last"));
        div()
            .w(px(400.))
            .child(Input::new(&self.outside).aria_label("Outside"))
            .child(gpuicn::dialog::dialog(
                "profile",
                &self.handle,
                popup,
                window,
                cx,
            ))
    }
}
#[gpui_kit::test]
fn modal_traps_tab_and_restores_focus(cx: &mut TestAppContext) {
    cx.update(gpuicn::init);
    let (view, cx) = cx.add_window_view(|window, cx| ModalView {
        handle: gpuicn::dialog::DialogHandle::new(false),
        outside: cx.new(|cx| InputState::new(window, cx)),
        first: cx.new(|cx| InputState::new(window, cx)),
        last: cx.new(|cx| InputState::new(window, cx)),
    });
    cx.update(|window, cx| {
        window.activate_window();
        window.draw(cx).clear(cx);
        view.read(cx)
            .outside
            .read(cx)
            .focus_handle(cx)
            .focus(window, cx);
        view.read(cx).handle.clone().open(window, cx);
        window.draw(cx).clear(cx);
    });
    for (key, first) in [
        ("tab", true),
        ("tab", false),
        ("tab", true),
        ("shift-tab", false),
    ] {
        cx.simulate_keystrokes(key);
        cx.update(|window, cx| {
            let state = if first {
                &view.read(cx).first
            } else {
                &view.read(cx).last
            };
            assert!(
                state.read(cx).focus_handle(cx).is_focused(window),
                "{key}, first={first}, expected={:?}, focused={:?}",
                state.read(cx).focus_handle(cx),
                window.focused(cx)
            );
        });
    }
    cx.simulate_keystrokes("escape");
    cx.update(|window, cx| {
        window.draw(cx).clear(cx);
        assert!(!view.read(cx).handle.is_open());
        assert!(
            view.read(cx)
                .outside
                .read(cx)
                .focus_handle(cx)
                .is_focused(window)
        );
    });
}

struct PickerView {
    state: Entity<gpuicn::select::SelectState>,
    editable: bool,
}
impl Render for PickerView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let picker = if self.editable {
            gpuicn::combobox::combobox(&self.state)
        } else {
            gpuicn::select::Select::new(&self.state)
        };
        picker
            .aria_label("Fruit")
            .placeholder("Choose fruit")
            .w(px(240.))
    }
}
#[gpui_kit::test]
fn picker_filters_commits_and_closes(cx: &mut TestAppContext) {
    cx.update(gpuicn::init);
    for editable in [false, true] {
        let (view, cx) = cx.add_window_view(move |window, cx| PickerView {
            state: cx.new(|cx| {
                gpuicn::select::SelectState::new(
                    [
                        gpuicn::select::SelectItem::new("apple", "Apple"),
                        gpuicn::select::SelectItem::new("orange", "Orange").disabled(true),
                        gpuicn::select::SelectItem::new("banana", "Banana"),
                    ],
                    window,
                    cx,
                )
            }),
            editable,
        });
        cx.update(|window, cx| {
            window.activate_window();
            window.draw(cx).clear(cx);
        });
        cx.simulate_keystrokes("tab");
        if editable {
            cx.simulate_input("ban");
        } else {
            cx.simulate_keystrokes("down down");
        }
        cx.simulate_keystrokes("enter");
        cx.update(|window, cx| {
            let picker = view.read(cx).state.read(cx);
            assert_eq!(
                picker.value().map(|s| s.as_ref()),
                Some("banana"),
                "editable={editable}"
            );
            assert!(!picker.is_open(), "editable={editable}");
            assert!(picker.focus_handle(cx).is_focused(window));
        });
        cx.simulate_keystrokes("down escape");
        cx.update(|_, cx| assert!(!view.read(cx).state.read(cx).is_open()));
    }
}

struct Menus {
    menu: Entity<gpuicn::menu::MenuState>,
    context: bool,
}
impl Render for Menus {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let menu = gpuicn::menu::Menu::new(&self.menu, "Actions");
        div().w(px(400.)).h(px(400.)).child(if self.context {
            menu.context_area(div().w(px(200.)).h(px(100.)).child("Right click"))
        } else {
            menu
        })
    }
}
#[gpui_kit::test]
fn menu_skips_disabled_opens_submenu_and_restores_focus(cx: &mut TestAppContext) {
    use gpuicn::menu::{MenuItem, MenuState};
    use std::{cell::Cell, rc::Rc};
    cx.update(gpuicn::init);
    for context in [false, true] {
        let called = Rc::new(Cell::new(0));
        let callback = called.clone();
        let (view, cx) = cx.add_window_view(move |_, cx| Menus {
            menu: cx.new(move |cx| {
                MenuState::new(
                    [
                        MenuItem::new("first", "First"),
                        MenuItem::new("unavailable", "Unavailable").disabled(true),
                        MenuItem::submenu(
                            "more",
                            "More",
                            [MenuItem::new("run", "Run")
                                .on_click(move |_, _, _| callback.set(callback.get() + 1))],
                        ),
                    ],
                    cx,
                )
            }),
            context,
        });
        cx.update(|window, cx| {
            window.activate_window();
            window.draw(cx).clear(cx);
        });
        cx.simulate_keystrokes("tab");
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.simulate_keystrokes(if context { "shift-f10" } else { "enter" });
        if !context {
            cx.update(|window, cx| {
                window.dispatch_event(
                    gpui_kit::PlatformInput::KeyUp(gpui_kit::KeyUpEvent {
                        keystroke: gpui_kit::Keystroke::parse("enter").unwrap(),
                    }),
                    cx,
                );
            });
        }
        cx.update(|_, cx| assert!(view.read(cx).menu.read(cx).is_open(), "context={context}"));
        for key in ["down", "right", "enter"] {
            cx.update(|window, cx| window.draw(cx).clear(cx));
            cx.simulate_keystrokes(key);
        }
        cx.update(|window, cx| {
            assert_eq!(called.get(), 1);
            let menu = view.read(cx).menu.read(cx);
            assert!(!menu.is_open());
            assert!(menu.trigger_focus().is_focused(window));
        });
    }
}
