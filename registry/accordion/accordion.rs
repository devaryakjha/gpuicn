//! Nova accordion presentation with caller-owned disclosure state.
use super::theme::UiTheme;
use gpui_icons::{LucideIcon, lucide};
pub use gpui_kit::base::{
    Accordion, AccordionHeader, AccordionItem, AccordionPanel, AccordionTrigger,
};
use gpui_kit::{
    App, ElementId, FontWeight, InteractiveElement as _, ParentElement as _, Styled,
    prelude::FluentBuilder as _,
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
        .when(disabled, |b| b.opacity(0.5).cursor_not_allowed())
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
/// Creates the panel; attach it with `AccordionItem::panel` to share visibility.
pub fn accordion_content(cx: &App) -> AccordionPanel {
    let theme = UiTheme::read(cx);
    AccordionPanel::new()
        .overflow_hidden()
        .pb(theme.space(2.5))
        .font_family(theme.fonts.body.clone())
        .text_size(theme.text(14.))
        .text_color(theme.colors.foreground)
}
