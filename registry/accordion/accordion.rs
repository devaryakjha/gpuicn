//! The shadcn Nova Accordion visual port.
//!
//! Visual source: shadcn/ui 4.19.0 `accordion.tsx` and `style-nova.css` at
//! `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`. Interaction, roving focus,
//! and disclosure state come from the pinned Base GPUI Accordion primitives.

pub use base_gpui::accordion::{
    AccordionHeader, AccordionItem, AccordionOrientation, AccordionPanel, AccordionRoot,
    AccordionTrigger,
};
use gpui::{
    App, BoxShadow, FontWeight, InteractiveElement as _, ParentElement as _, Styled,
    prelude::FluentBuilder as _, px,
};
use gpui_icons::{LucideIcon, lucide};

use super::theme::UiTheme;

/// Creates a full-width, vertical Accordion root.
pub fn accordion<T: Clone + Eq + 'static>(cx: &App) -> AccordionRoot<T> {
    let theme = UiTheme::read(cx).clone();
    AccordionRoot::new()
        .flex()
        .flex_col()
        .w_full()
        .font_family(theme.fonts.body)
}

/// Creates an Accordion item with the pinned Neutral divider.
pub fn accordion_item<T: Clone + Eq + 'static>(value: T, cx: &App) -> AccordionItem<T> {
    let theme = UiTheme::read(cx).clone();
    AccordionItem::new(value)
        .style_with_state(move |_state, base| base.border_b_1().border_color(theme.colors.border))
}

/// Creates the layout-only Accordion header.
pub fn accordion_header<T: Clone + Eq + 'static>() -> AccordionHeader<T> {
    AccordionHeader::new().flex()
}

/// Creates a Nova Accordion trigger with its chevron. Add the caller's label as a child.
pub fn accordion_trigger<T: Clone + Eq + 'static>(cx: &App) -> AccordionTrigger<T> {
    let theme = UiTheme::read(cx).clone();
    let icon_color = theme.colors.muted_foreground;
    AccordionTrigger::new()
        .style_with_state(move |state, base| {
            let colors = theme.colors;
            base.w_full()
                .flex()
                .items_start()
                .justify_between()
                .rounded(theme.radius.base)
                .border_1()
                .border_color(colors.background.alpha(0.0))
                .py(px(10.0))
                .text_left()
                .font_family(theme.fonts.body.clone())
                .font_weight(FontWeight::MEDIUM)
                .text_size(px(14.0))
                .text_color(colors.foreground)
                .when(!state.item.disabled, |base| {
                    base.cursor_pointer().hover(|style| style.underline())
                })
                .when(state.item.disabled, |base| {
                    base.opacity(0.50).cursor_not_allowed()
                })
                .focus_visible(move |style| {
                    style
                        .bg(colors.background)
                        .border_color(colors.ring)
                        .shadow(vec![
                            BoxShadow::new(px(0.0), px(0.0), colors.ring.alpha(0.50).into())
                                .spread_radius(px(3.0)),
                        ])
                })
        })
        .child(
            lucide(LucideIcon::ChevronDown)
                .size(px(16.0))
                .text_color(icon_color),
        )
}

/// Creates an Accordion panel with the pinned content inset.
pub fn accordion_content<T: Clone + Eq + 'static>(cx: &App) -> AccordionPanel<T> {
    let theme = UiTheme::read(cx).clone();
    AccordionPanel::new().style_with_state(move |_state, base| {
        base.overflow_hidden()
            .pb(px(10.0))
            .font_family(theme.fonts.body.clone())
            .text_size(px(14.0))
            .text_color(theme.colors.foreground)
    })
}
