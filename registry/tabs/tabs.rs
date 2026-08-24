//! The shadcn Nova Tabs visual port.
//!
//! Visual source: shadcn/ui 4.19.0 `tabs.tsx` and `style-nova.css` at
//! `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`. Selection and keyboard
//! navigation come from the pinned Base GPUI Tabs primitives.

pub use base_gpui::tabs::{TabsList, TabsPanel, TabsRoot, TabsTab};
use gpui::{
    App, BoxShadow, FontWeight, InteractiveElement as _, Styled, prelude::FluentBuilder as _, px,
};

use super::theme::UiTheme;

/// The pinned Tabs list visual treatment.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TabsVariant {
    /// A compact muted surface with the active tab raised above it.
    #[default]
    Default,
    /// A bare list with an underline below the active tab.
    Line,
}

/// Creates a vertical stack for a horizontal Tabs control.
pub fn tabs<T: Clone + Eq + 'static>(cx: &App) -> TabsRoot<T> {
    let theme = UiTheme::read(cx).clone();
    TabsRoot::new()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .font_family(theme.fonts.body)
}

/// Creates a Nova Tabs list using the default compact surface.
pub fn tabs_list<T: Clone + Eq + 'static>(cx: &App) -> TabsList<T> {
    tabs_list_with_variant(TabsVariant::Default, cx)
}

/// Creates a Nova Tabs list with an explicit visual treatment.
pub fn tabs_list_with_variant<T: Clone + Eq + 'static>(
    variant: TabsVariant,
    cx: &App,
) -> TabsList<T> {
    let theme = UiTheme::read(cx).clone();
    TabsList::new().style_with_state(move |_state, base| match variant {
        TabsVariant::Default => base
            .flex()
            .items_center()
            .justify_center()
            .h(px(32.0))
            .rounded(theme.radius.base)
            .p(px(3.0))
            .bg(theme.colors.muted)
            .text_color(theme.colors.muted_foreground),
        TabsVariant::Line => base
            .flex()
            .items_center()
            .gap(px(4.0))
            .border_b_1()
            .border_color(theme.colors.border)
            .text_color(theme.colors.muted_foreground),
    })
}

/// Creates a Nova tab trigger. `variant` must match its containing list.
pub fn tabs_trigger<T: Clone + Eq + 'static>(variant: TabsVariant, cx: &App) -> TabsTab<T> {
    let theme = UiTheme::read(cx).clone();
    TabsTab::new().style_with_state(move |state, base| {
        let colors = theme.colors;
        let selected = state.active;
        let base = base
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .h(px(26.0))
            .gap(px(6.0))
            .rounded(px(6.0))
            .border_1()
            .border_color(colors.background.alpha(0.0))
            .px(px(6.0))
            .py(px(2.0))
            .font_family(theme.fonts.body.clone())
            .font_weight(FontWeight::MEDIUM)
            .text_size(px(14.0))
            .text_color(if selected {
                colors.foreground
            } else {
                colors.muted_foreground
            })
            .when(!state.disabled, |base| {
                base.cursor_pointer()
                    .hover(move |style| style.text_color(colors.foreground))
            })
            .when(state.disabled, |base| {
                base.opacity(0.50).cursor_not_allowed()
            })
            .focus_visible(move |style| {
                style
                    .bg(match (variant, selected) {
                        (TabsVariant::Default, true) => colors.background,
                        (TabsVariant::Default, false) => colors.muted,
                        (TabsVariant::Line, _) => colors.background,
                    })
                    .border_color(colors.ring)
                    .shadow(vec![
                        BoxShadow::new(px(0.0), px(0.0), colors.ring.alpha(0.50).into())
                            .spread_radius(px(3.0)),
                    ])
            });

        match variant {
            TabsVariant::Default => base.when(selected, move |base| {
                base.bg(colors.background).shadow(vec![BoxShadow::new(
                    px(0.0),
                    px(1.0),
                    colors.foreground.alpha(0.08).into(),
                )])
            }),
            TabsVariant::Line => base
                .rounded(px(0.0))
                .border_b_1()
                .border_color(if selected {
                    colors.foreground
                } else {
                    colors.background.alpha(0.0)
                })
                .mb(px(-1.0)),
        }
    })
}

/// Creates the standard Nova Tabs content panel.
pub fn tabs_content<T: Clone + Eq + 'static>(cx: &App) -> TabsPanel<T> {
    let theme = UiTheme::read(cx).clone();
    TabsPanel::new().style_with_state(move |_state, base| {
        base.flex_1()
            .font_family(theme.fonts.body.clone())
            .text_size(px(14.0))
            .text_color(theme.colors.foreground)
    })
}
