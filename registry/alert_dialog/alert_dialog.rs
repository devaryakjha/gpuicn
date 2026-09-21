//! Nova confirmation dialog using Kit's non-dismissible alert host.
use super::{
    button::ButtonVariant,
    dialog::{
        DialogControl, DialogControlAction, DialogHandle, dialog_backdrop, modal_focus,
        modal_viewport,
    },
    theme::UiTheme,
};
pub use gpui_kit::base::AlertDialog;
use gpui_kit::{
    App, ElementId, InteractiveElement as _, IntoElement, ParentElement as _, Styled, Window, div,
};

/// Keep this host mounted while closed; validate the action in `.on_ok(...)`.
pub fn alert_dialog(
    id: impl Into<ElementId>,
    handle: &DialogHandle,
    popup: impl IntoElement,
    window: &mut Window,
    cx: &mut App,
) -> AlertDialog {
    let id = id.into();
    let focus = modal_focus::prepare(id.clone(), handle.is_open(), window, cx);
    let scope = modal_focus::ModalFocus::new(id.clone());
    let popup = scope
        .trap(div(), true)
        .id(id.clone())
        .track_focus(&focus)
        .w_full()
        .max_w(UiTheme::read(cx).space(96.))
        .child(scope.boundary(false))
        .child(scope.boundary(true))
        .child(modal_viewport((id, "viewport"), popup, window, cx));
    AlertDialog::new(cx)
        .handle(handle.clone())
        .close_on_escape(false)
        .backdrop(dialog_backdrop(cx))
        .popup(
            div()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .p(UiTheme::read(cx).space(4.))
                .child(popup),
        )
}
/// Explicit confirm button; the host's callback may veto dismissal.
pub fn alert_dialog_action(id: impl Into<ElementId>) -> DialogControl {
    DialogControl::new(
        id,
        DialogControlAction::Confirm,
        "gpuicn-alert-dialog-action-anchor",
    )
}
/// Cancel button with keyboard and focus behavior from Kit.
pub fn alert_dialog_cancel(id: impl Into<ElementId>) -> DialogControl {
    DialogControl::new(
        id,
        DialogControlAction::Cancel,
        "gpuicn-alert-dialog-cancel-anchor",
    )
    .variant(ButtonVariant::Outline)
}
