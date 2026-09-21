//! Nova modal surfaces using Kit's dialog host and caller-owned handles.
#[path = "modal_focus.rs"]
pub(crate) mod modal_focus;
use super::{
    button::{Button, ButtonSize, ButtonVariant},
    theme::UiTheme,
};
use gpui_icons::{LucideIcon, lucide};
pub use gpui_kit::base::{Dialog, DialogChangeReason, DialogHandle};
use gpui_kit::{
    AnyElement, App, Div, ElementId, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement, SharedString, Stateful, StatefulInteractiveElement as _, Styled, Window, div,
    px,
};

#[derive(Clone, Copy)]
pub(crate) enum DialogControlAction {
    Cancel,
    Confirm,
}

/// A styled dialog button that routes its action through its own dialog tree.
#[derive(IntoElement)]
pub struct DialogControl {
    button: Button,
    action: DialogControlAction,
    anchor_key: &'static str,
    anchor_id: ElementId,
}

impl DialogControl {
    pub(crate) fn new(
        id: impl Into<ElementId>,
        action: DialogControlAction,
        anchor_key: &'static str,
    ) -> Self {
        let id = id.into();
        Self {
            button: Button::new(id.clone()),
            action,
            anchor_key,
            anchor_id: id,
        }
    }

    /// Sets the visual variant.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.button = self.button.variant(variant);
        self
    }

    /// Sets the visual size.
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.button = self.button.size(size);
        self
    }

    /// Prevents activation and removes the button from tab order.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.button = self.button.disabled(disabled);
        self
    }

    /// Sets the accessible name for icon or custom content.
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.button = self.button.aria_label(label);
        self
    }

    /// Adds visible text and uses it as the accessible name.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.button = self.button.label(label);
        self
    }
}

impl ParentElement for DialogControl {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.button.extend(elements);
    }
}

impl Styled for DialogControl {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        self.button.style()
    }
}

impl gpui_kit::RenderOnce for DialogControl {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let anchor = window
            .use_keyed_state((self.anchor_id, self.anchor_key), cx, |_, cx| {
                cx.focus_handle()
            })
            .read(cx)
            .clone();
        let dispatch_from = anchor.clone();
        let action = self.action;

        self.button
            .child(div().absolute().size_0().track_focus(&anchor))
            .on_click(move |_, window, cx| match action {
                DialogControlAction::Cancel => {
                    dispatch_from.dispatch_action(&gpui_kit::base::actions::Cancel, window, cx)
                }
                DialogControlAction::Confirm => dispatch_from.dispatch_action(
                    &gpui_kit::base::actions::Confirm { secondary: false },
                    window,
                    cx,
                ),
            })
    }
}

/// Keep the host mounted while closed so it can restore focus after dismissal.
pub fn dialog(
    id: impl Into<ElementId>,
    handle: &DialogHandle,
    popup: impl IntoElement,
    window: &mut Window,
    cx: &mut App,
) -> Dialog {
    let id = id.into();
    let focus = modal_focus::prepare(id.clone(), handle.is_open(), window, cx);
    let scope = modal_focus::ModalFocus::new(id.clone());
    let popup = scope
        .trap(div(), true)
        .id(id.clone())
        .track_focus(&focus)
        .w_full()
        .max_w(UiTheme::read(cx).space(96.))
        .on_key_down(|event, window, cx| {
            if event.keystroke.key == "escape" {
                window.dispatch_action(Box::new(gpui_kit::base::actions::Cancel), cx);
                cx.stop_propagation();
            }
        })
        .child(scope.boundary(false))
        .child(scope.boundary(true))
        .child(modal_viewport((id.clone(), "viewport"), popup, window, cx));
    Dialog::new(cx)
        .handle(handle.clone())
        .close_on_escape(false)
        .flex()
        .items_center()
        .justify_center()
        .p(UiTheme::read(cx).space(4.))
        .backdrop(dialog_backdrop(cx))
        .popup(popup)
        // Enter in an arbitrary child must not dismiss an unfinished form.
        .on_ok(|_, _, _| false)
}
/// Keeps a modal's full content reachable even in a short window.
pub(crate) fn modal_viewport(
    id: impl Into<ElementId>,
    popup: impl IntoElement,
    window: &Window,
    cx: &App,
) -> Stateful<Div> {
    div()
        .id(id)
        .max_h((window.viewport_size().height - UiTheme::read(cx).space(8.)).max(px(0.)))
        .overflow_y_scroll()
        .child(popup)
}

/// A keyboard-accessible button opening the caller's handle.
pub fn dialog_trigger(id: impl Into<ElementId>, handle: &DialogHandle, _cx: &App) -> Button {
    let handle = handle.clone();
    Button::new(id)
        .variant(ButtonVariant::Outline)
        .on_click(move |_, window, cx| handle.open(window, cx))
}
/// Full-window modal backdrop.
pub fn dialog_backdrop(cx: &App) -> Div {
    div()
        .absolute()
        .inset_0()
        .bg(UiTheme::read(cx).colors.overlay)
        .occlude()
}
/// Stacked dialog heading content.
pub fn dialog_header(cx: &App) -> Div {
    div().flex().flex_col().gap(UiTheme::read(cx).space(2.))
}
/// Inset action footer.
pub fn dialog_footer(cx: &App) -> Div {
    let t = UiTheme::read(cx);
    div()
        .mx(-t.space(4.))
        .mb(-t.space(4.))
        .flex()
        .justify_end()
        .gap(t.space(2.))
        .rounded_b(t.radius.xl)
        .border_t_1()
        .border_color(t.colors.border)
        .bg(t.colors.muted.opacity(0.5))
        .p(t.space(4.))
}
/// Named popup surface with modal Tab traversal, including caller-owned inputs.
pub fn dialog_popup(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    cx: &App,
) -> Stateful<Div> {
    let t = UiTheme::read(cx);
    let id = id.into();
    div()
        .id(id)
        .aria_label(label)
        .occlude()
        .w_full()
        .min_w(px(0.))
        .max_w(t.space(96.))
        .max_h(t.space(100.))
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .gap(t.space(4.))
        .rounded(t.radius.xl)
        .border_1()
        .border_color(t.colors.foreground.opacity(0.1))
        .p(t.space(4.))
        .bg(t.colors.popover)
        .text_color(t.colors.popover_foreground)
        .font_family(t.fonts.body.clone())
        .text_size(t.text(14.))
}
/// Medium-weight dialog title.
pub fn dialog_title(id: impl Into<ElementId>, cx: &App) -> Stateful<Div> {
    let t = UiTheme::read(cx);
    div()
        .id(id)
        .role(gpui_kit::Role::Heading)
        .aria_level(2)
        .font_family(t.fonts.heading.clone())
        .font_weight(FontWeight::MEDIUM)
        .text_size(t.text(16.))
        .line_height(t.text(16.))
        .text_color(t.colors.popover_foreground)
}
/// Muted dialog description.
pub fn dialog_description(id: impl Into<ElementId>, cx: &App) -> Stateful<Div> {
    let t = UiTheme::read(cx);
    div()
        .id(id)
        .font_family(t.fonts.body.clone())
        .text_size(t.text(14.))
        .text_color(t.colors.muted_foreground)
}
/// Icon-only close control; Kit routes Cancel through the owning dialog's veto callback.
pub fn dialog_close(id: impl Into<ElementId>, cx: &App) -> DialogControl {
    let t = UiTheme::read(cx);
    DialogControl::new(
        id,
        DialogControlAction::Cancel,
        "gpuicn-dialog-close-anchor",
    )
    .aria_label("Close")
    .variant(ButtonVariant::Ghost)
    .size(ButtonSize::IconSm)
    .absolute()
    .top(t.space(2.))
    .right(t.space(2.))
    .child(
        lucide(LucideIcon::X)
            .size(t.space(4.))
            .text_color(t.colors.popover_foreground),
    )
}
/// Create form actions with `Button`; close the handle after successful validation.
pub fn dialog_action(id: impl Into<ElementId>, handle: &DialogHandle, _cx: &App) -> Button {
    let handle = handle.clone();
    Button::new(id).on_click(move |_, window, cx| handle.close(window, cx))
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use gpui_kit::{Context, FocusHandle, Render, TestAppContext, point};

    use super::*;

    struct DialogControlHarness {
        dialog_focus: FocusHandle,
        retained_focus: FocusHandle,
        handle: DialogHandle,
        cancel_count: Rc<Cell<usize>>,
    }

    impl Render for DialogControlHarness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let cancel_count = self.cancel_count.clone();
            div()
                .child(div().absolute().size_0().track_focus(&self.retained_focus))
                .child(
                    Dialog::new(cx)
                        .handle(self.handle.clone())
                        .focus_handle(self.dialog_focus.clone())
                        .on_cancel(move |_, _, _| {
                            cancel_count.set(cancel_count.get() + 1);
                            true
                        })
                        .popup(
                            DialogControl::new(
                                "cancel",
                                DialogControlAction::Cancel,
                                "dialog-control-test-anchor",
                            )
                            .size_full()
                            .label("Cancel"),
                        ),
                )
        }
    }

    #[gpui_kit::test]
    fn dialog_control_routes_from_its_own_anchor(cx: &mut TestAppContext) {
        cx.update(super::super::theme::init);
        let handle = DialogHandle::new(true);
        let cancel_count = Rc::new(Cell::new(0));
        let (view, cx) = cx.add_window_view({
            let handle = handle.clone();
            let cancel_count = cancel_count.clone();
            move |_, cx| DialogControlHarness {
                dialog_focus: cx.focus_handle(),
                retained_focus: cx.focus_handle(),
                handle,
                cancel_count,
            }
        });
        cx.update(|window, cx| {
            let retained_focus = view.read(cx).retained_focus.clone();
            retained_focus.focus(window, cx);
            window.draw(cx).clear(cx);
        });

        cx.simulate_click(point(px(20.), px(20.)), Default::default());
        cx.run_until_parked();

        assert_eq!(cancel_count.get(), 1);
        assert!(!handle.is_open());
    }

    struct TwoDialogControlHarness {
        first_focus: FocusHandle,
        second_focus: FocusHandle,
        first_handle: DialogHandle,
        second_handle: DialogHandle,
        first_count: Rc<Cell<usize>>,
        second_count: Rc<Cell<usize>>,
    }

    impl Render for TwoDialogControlHarness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let first_count = self.first_count.clone();
            let second_count = self.second_count.clone();
            div()
                .child(
                    Dialog::new(cx)
                        .handle(self.first_handle.clone())
                        .focus_handle(self.first_focus.clone())
                        .on_cancel(move |_, _, _| {
                            first_count.set(first_count.get() + 1);
                            true
                        })
                        .popup(
                            DialogControl::new(
                                "first-cancel",
                                DialogControlAction::Cancel,
                                "shared-dialog-control-anchor",
                            )
                            .absolute()
                            .left_0()
                            .top_0()
                            .w(px(40.))
                            .h(px(40.))
                            .label("First"),
                        ),
                )
                .child(
                    Dialog::new(cx)
                        .handle(self.second_handle.clone())
                        .focus_handle(self.second_focus.clone())
                        .on_cancel(move |_, _, _| {
                            second_count.set(second_count.get() + 1);
                            true
                        })
                        .popup(
                            DialogControl::new(
                                "second-cancel",
                                DialogControlAction::Cancel,
                                "shared-dialog-control-anchor",
                            )
                            .absolute()
                            .left(px(60.))
                            .top_0()
                            .w(px(40.))
                            .h(px(40.))
                            .label("Second"),
                        ),
                )
        }
    }

    #[gpui_kit::test]
    fn dialog_controls_keep_distinct_anchors_in_one_window(cx: &mut TestAppContext) {
        cx.update(super::super::theme::init);
        let first_handle = DialogHandle::new(true);
        let second_handle = DialogHandle::new(true);
        let first_count = Rc::new(Cell::new(0));
        let second_count = Rc::new(Cell::new(0));
        let (_, cx) = cx.add_window_view({
            let first_handle = first_handle.clone();
            let second_handle = second_handle.clone();
            let first_count = first_count.clone();
            let second_count = second_count.clone();
            move |_, cx| TwoDialogControlHarness {
                first_focus: cx.focus_handle(),
                second_focus: cx.focus_handle(),
                first_handle,
                second_handle,
                first_count,
                second_count,
            }
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));

        cx.simulate_click(point(px(80.), px(20.)), Default::default());
        cx.run_until_parked();

        assert_eq!(first_count.get(), 0);
        assert_eq!(second_count.get(), 1);
        assert!(first_handle.is_open());
        assert!(!second_handle.is_open());
    }
}
