//! Modal focus entry, traversal and restoration for Kit hosts.
use gpui_kit::{
    App, Div, ElementId, FocusHandle, InteractiveElement as _, IntoElement, RenderOnce,
    SharedString, Styled, Window, div, px,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
};

struct FocusState {
    open: bool,
    popup: FocusHandle,
    return_to: Option<FocusHandle>,
}
pub(crate) fn prepare(id: ElementId, open: bool, window: &mut Window, cx: &mut App) -> FocusHandle {
    let state = window.use_keyed_state((id, "modal-focus"), cx, |_, cx| FocusState {
        open: false,
        popup: cx.focus_handle(),
        return_to: None,
    });
    state.update(cx, |state, cx| {
        if open && !state.open {
            state.return_to = window.focused(cx);
            state.popup.focus(window, cx);
        } else if !open && state.open {
            if (state.popup.contains_focused(window, cx) || window.focused(cx).is_none())
                && let Some(previous) = state.return_to.take()
            {
                previous.focus(window, cx);
            }
            state.return_to = None;
        }
        state.open = open;
        state.popup.clone()
    })
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
        base.tab_group()
            .capture_action(move |_: &super::super::theme::FocusNext, window, cx| {
                next.advance(false, window, cx)
            })
            .capture_action(move |_: &super::super::theme::FocusPrevious, window, cx| {
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
