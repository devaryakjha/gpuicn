//! The shadcn Nova Collapsible visual port.
//!
//! Visual source: shadcn/ui 4.19.0 `collapsible.tsx` and `style-nova.css` at
//! `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`. Interaction and disclosure
//! state come from the pinned Base GPUI Collapsible primitives.

pub use base_gpui::collapsible::{CollapsiblePanel, CollapsibleRoot, CollapsibleTrigger};
use gpui::{
    App, BoxShadow, FontWeight, InteractiveElement as _, Styled, prelude::FluentBuilder as _, px,
};

use super::theme::UiTheme;

/// Creates a Collapsible root that keeps its caller-owned layout.
pub fn collapsible(cx: &App) -> CollapsibleRoot {
    let theme = UiTheme::read(cx).clone();
    CollapsibleRoot::new()
        .flex()
        .flex_col()
        .font_family(theme.fonts.body)
}

/// Creates a Nova Collapsible trigger. Add its visible content as children.
pub fn collapsible_trigger(cx: &App) -> CollapsibleTrigger {
    let theme = UiTheme::read(cx).clone();
    CollapsibleTrigger::new().style_with_state(move |state, base| {
        let colors = theme.colors;
        base.flex()
            .items_center()
            .justify_center()
            .rounded(theme.radius.base)
            .border_1()
            .border_color(colors.background.alpha(0.0))
            .px(px(10.0))
            .py(px(6.0))
            .font_family(theme.fonts.body.clone())
            .font_weight(FontWeight::MEDIUM)
            .text_size(px(14.0))
            .text_color(colors.foreground)
            .when(!state.disabled, |base| {
                base.cursor_pointer()
                    .hover(move |style| style.bg(colors.muted))
            })
            .when(state.disabled, |base| {
                base.opacity(0.50).cursor_not_allowed()
            })
            .focus_visible(move |style| {
                style.border_color(colors.ring).shadow(vec![
                    BoxShadow::new(px(0.0), px(0.0), colors.ring.alpha(0.50).into())
                        .spread_radius(px(3.0)),
                ])
            })
    })
}

/// Creates a Collapsible content panel with the standard Nova text treatment.
pub fn collapsible_content(cx: &App) -> CollapsiblePanel {
    let theme = UiTheme::read(cx).clone();
    CollapsiblePanel::new().style_with_state(move |_state, base| {
        base.overflow_hidden()
            .pt(px(8.0))
            .font_family(theme.fonts.body.clone())
            .text_size(px(14.0))
            .text_color(theme.colors.foreground)
    })
}
