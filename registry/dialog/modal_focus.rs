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
        base.tab_group()
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
    use gpui::{AppContext as _, Context, ParentElement as _, Render, TestAppContext};

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
                    .trap(div().track_focus(&self.popup), self.trap)
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
            for (reverse, expected) in [
                (false, &first),
                (false, &last),
                (false, &first),
                (true, &last),
                (true, &first),
            ] {
                scope.advance(reverse, window, cx);
                assert!(
                    expected.is_focused(window),
                    "reverse={reverse}, expected={expected:?}, actual={:?}",
                    window.focused(cx)
                );
                assert!(!outside.is_focused(window));
            }
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
