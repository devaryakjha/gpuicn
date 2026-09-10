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
    App, Div, ElementId, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, Stateful, StatefulInteractiveElement as _, Styled, Window, div, px,
};

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
/// Icon-only close control; Kit routes Cancel through the host's veto callback.
pub fn dialog_close(id: impl Into<ElementId>, cx: &App) -> Button {
    let t = UiTheme::read(cx);
    Button::new(id)
        .aria_label("Close")
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::IconSm)
        .absolute()
        .top(t.space(2.))
        .right(t.space(2.))
        .on_click(|_, window, cx| {
            window.dispatch_action(Box::new(gpui_kit::base::actions::Cancel), cx)
        })
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
