//! Modal keyboard traversal over GPUI's actual tab stops, including app-owned
//! inputs passed through Base GPUI's `child_any` slots.

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
};

use base_gpui::dialog::{DialogFocusNextAction, DialogFocusPreviousAction};
use gpui::{
    App, Div, ElementId, FocusHandle, InteractiveElement as _, IntoElement, RenderOnce,
    SharedString, Styled, Window, div, px,
};

/// Reconciles externally controlled transitions that Base's `sync_open_from_context`
/// does not send through its focus lifecycle. The Base root has already wired its
/// popup handles before rendering this zero-sized child.
#[derive(IntoElement)]
pub(crate) struct RootFocus {
    id: ElementId,
    handle: base_gpui::dialog::DialogHandle<()>,
}

impl RootFocus {
    pub(crate) fn new(id: ElementId, handle: base_gpui::dialog::DialogHandle<()>) -> Self {
        Self { id, handle }
    }
}

#[derive(Default)]
struct RootFocusState {
    open: bool,
    return_to: Option<FocusHandle>,
}

impl RenderOnce for RootFocus {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let Some(context) = self.handle.context() else {
            // A caller-supplied handle owns Base's normal open/close focus flow.
            return div().absolute().size(px(0.));
        };
        let state = window.use_keyed_state(
            ElementId::NamedChild(Arc::new(self.id), "controlled-focus".into()),
            cx,
            |_, _| RootFocusState::default(),
        );
        let popup = context.read(cx, |runtime, _| runtime.popup_focus_neighbor(None, false));
        let open = context.read(cx, |runtime, props| {
            runtime.open_value() && props.modal_mode().traps_focus()
        });
        state.update(cx, |state, cx| {
            let inside = popup
                .as_ref()
                .is_some_and(|popup| popup.contains_focused(window, cx));
            if open && !state.open {
                if inside {
                    // A trigger or handle already ran Base's focus transition.
                    state.return_to =
                        context.read(cx, |runtime, _| runtime.previous_focus_handle());
                } else {
                    state.return_to = window.focused(cx);
                    context.update(cx, |runtime| {
                        runtime.capture_previous_focus(state.return_to.clone())
                    });
                    if let Some(popup) = &popup {
                        popup.focus(window, cx);
                    }
                }
            } else if !open && state.open {
                // Preserve focus explicitly moved by a close handler or by Base.
                if (inside || window.focused(cx).is_none())
                    && let Some(handle) = state.return_to.take()
                {
                    handle.focus(window, cx);
                }
                state.return_to = None;
            }
            state.open = open;
        });
        div().absolute().size(px(0.))
    }
}

#[derive(Clone)]
pub(crate) struct ModalFocus {
    id: ElementId,
    enabled: Rc<Cell<bool>>,
    handles: Rc<RefCell<[Option<FocusHandle>; 2]>>,
}

impl ModalFocus {
    pub(crate) fn new(id: ElementId) -> Self {
        Self {
            id,
            enabled: Rc::default(),
            handles: Rc::default(),
        }
    }

    pub(crate) fn boundary(&self, end: bool) -> impl IntoElement {
        Boundary {
            scope: self.clone(),
            end,
        }
    }

    pub(crate) fn trap(&self, base: Div, enabled: bool) -> Div {
        self.enabled.set(enabled);
        if !enabled {
            return base;
        }
        let next = self.clone();
        let previous = self.clone();
        // Child controls can resolve Tab to the shared fallback bindings.
        let fallback_next = self.clone();
        let fallback_previous = self.clone();
        base.tab_group()
            .capture_action(move |_: &super::super::theme::FocusNext, window, cx| {
                fallback_next.advance(false, window, cx)
            })
            .capture_action(move |_: &super::super::theme::FocusPrevious, window, cx| {
                fallback_previous.advance(true, window, cx)
            })
            .capture_action(move |_: &DialogFocusNextAction, window, cx| {
                next.advance(false, window, cx)
            })
            .capture_action(move |_: &DialogFocusPreviousAction, window, cx| {
                previous.advance(true, window, cx)
            })
    }

    fn advance(&self, reverse: bool, window: &mut Window, cx: &mut App) {
        let [Some(start), Some(end)] = self.handles.borrow().clone() else {
            return;
        };
        let original = window.focused(cx);
        if reverse {
            // The initially focused popup is the ancestor of both boundaries.
            // Enter from the end without focusing anything behind the modal.
            if start.within_focused(window, cx) {
                end.focus(window, cx);
            }
            window.focus_prev(cx);
            if start.is_focused(window) {
                end.focus(window, cx);
                window.focus_prev(cx);
            }
        } else {
            window.focus_next(cx);
            if start.is_focused(window) {
                window.focus_next(cx);
            }
            if end.is_focused(window) {
                start.focus(window, cx);
                window.focus_next(cx);
            }
        }
        if (start.is_focused(window) || end.is_focused(window))
            && let Some(original) = original
        {
            original.focus(window, cx);
        }
        window.prevent_default();
        cx.stop_propagation();
    }
}

#[derive(IntoElement)]
struct Boundary {
    scope: ModalFocus,
    end: bool,
}

impl RenderOnce for Boundary {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let index = usize::from(self.end);
        let id = ElementId::NamedChild(
            Arc::new(self.scope.id),
            SharedString::from(if self.end { "tab-end" } else { "tab-start" }),
        );
        let handle = window
            .use_keyed_state(id.clone(), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        self.scope.handles.borrow_mut()[index] = Some(handle.clone());
        // Out of layout and role-less. The end's group index follows all normal
        // controls, including custom positive tab indices, although both boundaries are inserted first.
        div().id(id).absolute().size(px(0.)).track_focus(
            &handle
                .tab_stop(self.scope.enabled.get())
                .tab_index(if self.end { isize::MAX } else { 0 }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{
        AppContext as _, Context, ParentElement as _, Render, TestAppContext,
        prelude::FluentBuilder as _,
    };

    struct ControlledView {
        open: bool,
        modal: bool,
        handle: Option<base_gpui::dialog::DialogHandle<()>>,
        outside: FocusHandle,
        first: FocusHandle,
        last: FocusHandle,
    }
    impl Render for ControlledView {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            use super::super::*;
            let view = cx.entity().downgrade();
            dialog_root("controlled-test")
                .open(self.open)
                .modal(self.modal)
                .map(|root| match &self.handle {
                    Some(handle) => root.handle(handle.clone()),
                    None => root,
                })
                .on_open_change(move |open, _, _, cx| {
                    let _ = view.update(cx, |view, cx| {
                        view.open = open;
                        cx.notify();
                    });
                })
                .child_any(
                    base_gpui::input::Input::new()
                        .id("outside-input")
                        .focus_handle(self.outside.clone()),
                )
                .child(dialog_trigger("trigger", cx).child("Open"))
                .child(
                    dialog_portal().child(
                        dialog_viewport(cx).child(
                            dialog_popup("controlled-popup", "Confirmation", cx)
                                .child_any(div().track_focus(&self.first).child("First"))
                                .child_any(div().track_focus(&self.last).child("Last")),
                        ),
                    ),
                )
        }
    }

    #[gpui::test]
    fn controlled_dialog_moves_traps_and_restores_focus(cx: &mut TestAppContext) {
        cx.update(super::super::super::theme::init);
        let outside = cx.update(|cx| cx.focus_handle().tab_stop(true));
        let first = cx.update(|cx| cx.focus_handle().tab_stop(true));
        let last = cx.update(|cx| cx.focus_handle().tab_stop(true));
        let window = cx.add_window({
            let (outside, first, last) = (outside.clone(), first.clone(), last.clone());
            move |_, _| ControlledView {
                open: false,
                modal: true,
                handle: None,
                outside,
                first,
                last,
            }
        });
        cx.update_window(window.into(), |_, window, cx| {
            window.activate_window();
            window.draw(cx).clear(cx);
            outside.focus(window, cx);
        })
        .unwrap();
        let set_open = |cx: &mut TestAppContext, open, modal| {
            cx.update_window(window.into(), |view, window, cx| {
                view.downcast::<ControlledView>()
                    .unwrap()
                    .update(cx, |view, cx| {
                        view.open = open;
                        view.modal = modal;
                        cx.notify();
                    });
                window.draw(cx).clear(cx);
            })
            .unwrap();
        };
        set_open(cx, true, true);
        for expected in [&first, &last, &first] {
            cx.simulate_keystrokes(window.into(), "tab");
            cx.update_window(window.into(), |_, window, cx| {
                assert!(
                    expected.is_focused(window),
                    "modal Tab must stay in popup; expected={expected:?} actual={:?}",
                    window.focused(cx)
                );
                assert!(!outside.is_focused(window));
            })
            .unwrap();
        }
        cx.simulate_keystrokes(window.into(), "escape");
        cx.update_window(window.into(), |view, window, cx| {
            assert!(!view.downcast::<ControlledView>().unwrap().read(cx).open);
            window.draw(cx).clear(cx);
            assert!(outside.is_focused(window), "Escape restores input focus");
        })
        .unwrap();
        set_open(cx, true, true);
        set_open(cx, false, true);
        cx.update_window(window.into(), |_, window, _| {
            assert!(
                outside.is_focused(window),
                "controlled close restores input focus"
            )
        })
        .unwrap();
        set_open(cx, true, false);
        cx.update_window(window.into(), |_, window, _| {
            assert!(
                outside.is_focused(window),
                "nonmodal opening must not steal focus"
            )
        })
        .unwrap();
        set_open(cx, false, true);
        cx.simulate_keystrokes(window.into(), "tab");
        cx.run_until_parked();
        let trigger = cx
            .update_window(window.into(), |_, window, cx| window.focused(cx).unwrap())
            .unwrap();
        cx.simulate_keystrokes(window.into(), "enter");
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
            assert!(!trigger.is_focused(window), "trigger opens popup");
        })
        .unwrap();
        cx.simulate_keystrokes(window.into(), "escape");
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
            assert!(
                trigger.is_focused(window),
                "trigger focus restoration remains intact"
            );
        })
        .unwrap();
        let handle = base_gpui::dialog::DialogHandle::new();
        cx.update_window(window.into(), |view, window, cx| {
            view.downcast::<ControlledView>()
                .unwrap()
                .update(cx, |view, cx| {
                    view.handle = Some(handle.clone());
                    cx.notify();
                });
            window.draw(cx).clear(cx);
            outside.focus(window, cx);
            assert!(handle.open("missing-trigger", window, cx));
            window.draw(cx).clear(cx);
            assert!(!outside.is_focused(window), "caller handle opens popup");
        })
        .unwrap();
        cx.simulate_keystrokes(window.into(), "escape");
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
            assert!(
                outside.is_focused(window),
                "caller handle restores previous focus"
            );
        })
        .unwrap();
    }

    struct View {
        popup: FocusHandle,
        first: FocusHandle,
        last: FocusHandle,
        outside: FocusHandle,
        scope: ModalFocus,
        trap: bool,
    }
    impl Render for View {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().child(div().track_focus(&self.outside)).child(
                self.scope
                    .trap(
                        div()
                            .key_context(base_gpui::dialog::DIALOG_POPUP_KEY_CONTEXT)
                            .track_focus(&self.popup),
                        self.trap,
                    )
                    .child(self.scope.boundary(false))
                    .child(self.scope.boundary(true))
                    .child(div().track_focus(&self.first))
                    .child(div().track_focus(&self.last)),
            )
        }
    }

    #[test]
    fn modal_tab_order_includes_generic_children_and_wraps_both_ways() {
        let mut cx = TestAppContext::single();
        cx.update(super::super::super::theme::init);
        let (popup, first, last, outside) = cx.update(|cx| {
            (
                cx.focus_handle().tab_stop(true),
                cx.focus_handle().tab_stop(true),
                cx.focus_handle().tab_stop(true),
                cx.focus_handle().tab_stop(true),
            )
        });
        let scope = ModalFocus::new("test-modal".into());
        let window = cx.add_window({
            let (popup, first, last, outside, scope) = (
                popup.clone(),
                first.clone(),
                last.clone(),
                outside.clone(),
                scope.clone(),
            );
            move |_, _| View {
                popup,
                first,
                last,
                outside,
                scope,
                trap: true,
            }
        });
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
            popup.focus(window, cx);
        })
        .unwrap();
        for (keys, expected) in [
            ("tab", &first),
            ("tab", &last),
            ("tab", &first),
            ("shift-tab", &last),
            ("shift-tab", &first),
        ] {
            cx.simulate_keystrokes(window.into(), keys);
            cx.update_window(window.into(), |_, window, _| {
                assert!(expected.is_focused(window), "{keys}");
                assert!(!outside.is_focused(window));
            })
            .unwrap();
        }
        cx.update_window(window.into(), |_, window, cx| {
            popup.focus(window, cx);
            scope.advance(true, window, cx);
            assert!(last.is_focused(window));
        })
        .unwrap();
        cx.update_window(window.into(), |view, window, cx| {
            view.downcast::<View>()
                .unwrap()
                .update(cx, |view, _| view.trap = false);
            window.draw(cx).clear(cx);
            popup.focus(window, cx);
            window.focus_next(cx);
            assert!(
                first.is_focused(window),
                "non-modal traversal must skip hidden boundaries"
            );
        })
        .unwrap();
    }
}
