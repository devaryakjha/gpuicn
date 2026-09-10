//! Nova accordion presentation with caller-owned disclosure state.
use super::theme::UiTheme;
use gpui_icons::{LucideIcon, lucide};
pub use gpui_kit::base::{
    Accordion, AccordionHeader, AccordionItem, AccordionPanel, AccordionTrigger,
};
use gpui_kit::{
    App, ElementId, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _, Styled,
    Window, div, prelude::FluentBuilder as _,
};

/// Creates the accordion collection. Each item receives its expanded state from the caller.
pub fn accordion(id: impl Into<ElementId>, cx: &App) -> Accordion {
    Accordion::new(id)
        .flex()
        .flex_col()
        .w_full()
        .font_family(UiTheme::read(cx).fonts.body.clone())
}
/// Creates an item with the Nova divider.
pub fn accordion_item(cx: &App) -> AccordionItem {
    AccordionItem::new()
        .border_b_1()
        .border_color(UiTheme::read(cx).colors.border)
}
/// Creates a heading around a trigger.
pub fn accordion_header(trigger: AccordionTrigger) -> AccordionHeader {
    AccordionHeader::new(trigger).flex()
}
/// Creates a keyboard-focusable disclosure trigger with a chevron.
pub fn accordion_trigger(
    id: impl Into<ElementId>,
    open: bool,
    disabled: bool,
    cx: &App,
) -> AccordionTrigger {
    let theme = UiTheme::read(cx);
    let colors = theme.colors;
    let ring = theme.focus_ring();
    AccordionTrigger::new(id)
        .open(open)
        .disabled(disabled)
        .when(!disabled, |b| {
            b.tab_index(0).cursor_pointer().hover(|s| s.underline())
        })
        .when(disabled, |b| {
            b.opacity(0.5)
                .cursor_not_allowed()
                .capture_any_mouse_down(|_, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                })
                .capture_any_mouse_up(|_, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                })
                .capture_key_down(|event, window, cx| {
                    if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                        window.prevent_default();
                        cx.stop_propagation();
                    }
                })
        })
        .w_full()
        .flex()
        .flex_row_reverse()
        .items_start()
        .justify_between()
        .rounded(theme.radius.lg)
        .border_1()
        .border_color(colors.background.opacity(0.))
        .py(theme.space(2.5))
        .text_left()
        .font_family(theme.fonts.body.clone())
        .font_weight(FontWeight::MEDIUM)
        .text_size(theme.text(14.))
        .text_color(colors.foreground)
        .focus_visible(move |s| {
            s.bg(colors.background)
                .border_color(colors.ring)
                .shadow(ring.clone())
        })
        .child(super::theme::disclosure_icon(
            lucide(LucideIcon::ChevronDown)
                .size(theme.space(4.))
                .text_color(colors.muted_foreground),
            open,
        ))
}
/// Creates a controlled panel, retaining its content only while the exit motion runs.
pub fn accordion_content(
    id: impl Into<ElementId>,
    open: bool,
    content: impl IntoElement,
    window: &mut Window,
    cx: &mut App,
) -> AccordionPanel {
    let id = id.into();
    let presence = super::theme::presence(id.clone(), open, window, cx);
    let theme = UiTheme::read(cx);
    AccordionPanel::new()
        .open(open)
        .keep_mounted(presence.should_render() && (open || presence.progress > 0.))
        .child(gpui_kit::base::motion::MotionReveal::new(
            id,
            presence.progress,
            div()
                .w_full()
                .pb(theme.space(2.5))
                .font_family(theme.fonts.body.clone())
                .text_size(theme.text(14.))
                .text_color(theme.colors.foreground)
                .child(content)
                .into_any_element(),
        ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{
        Context, Modifiers, Render, StatefulInteractiveElement as _, TestAppContext, point, px,
    };
    use std::{cell::Cell, rc::Rc};

    #[gpui_kit::test]
    fn disabled_trigger_blocks_caller_click_handlers(cx: &mut TestAppContext) {
        struct Probe {
            disabled: bool,
            clicks: Rc<Cell<usize>>,
        }
        impl Render for Probe {
            fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                let clicks = self.clicks.clone();
                accordion_trigger("trigger", false, self.disabled, cx)
                    .size(px(100.))
                    .on_click(move |_, _, _| clicks.set(clicks.get() + 1))
            }
        }
        cx.update(|cx| UiTheme::set(cx, UiTheme::neutral_light()));
        for disabled in [false, true] {
            let clicks = Rc::new(Cell::new(0));
            let (_, visual) = cx.add_window_view({
                let clicks = clicks.clone();
                move |_, _| Probe { disabled, clicks }
            });
            visual.update(|window, cx| window.draw(cx).clear(cx));
            visual.simulate_click(point(px(10.), px(10.)), Modifiers::default());
            assert_eq!(clicks.get(), usize::from(!disabled));
        }
    }
}
