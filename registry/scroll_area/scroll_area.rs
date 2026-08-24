//! The shadcn Nova Scroll Area visual port.
//!
//! Visual source: shadcn/ui 4.19.0 `scroll-area.tsx` and `style-nova.css` at
//! `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`. Scrolling, pointer dragging,
//! track clicks, and keyboard support come from the pinned Base GPUI parts.

use base_gpui::primitives::ScrollbarStyle;
pub use base_gpui::scroll_area::{
    ScrollAreaContent, ScrollAreaCorner, ScrollAreaOrientation, ScrollAreaRoot,
    ScrollAreaScrollbar, ScrollAreaThumb, ScrollAreaViewport,
};
use gpui::{App, BoxShadow, InteractiveElement as _, Styled, prelude::FluentBuilder as _, px};

use super::theme::UiTheme;

/// Creates the relative Scroll Area root.
pub fn scroll_area(cx: &App) -> ScrollAreaRoot {
    let theme = UiTheme::read(cx).clone();
    ScrollAreaRoot::new()
        .font_family(theme.fonts.body)
        .text_color(theme.colors.foreground)
}

/// Creates the focusable scroll viewport. Add `scroll_area_content()` as its child.
pub fn scroll_area_viewport(cx: &App) -> ScrollAreaViewport {
    let theme = UiTheme::read(cx).clone();
    ScrollAreaViewport::new().style_with_state(move |_state, base| {
        let colors = theme.colors;
        base.size_full()
            .rounded(theme.radius.base)
            .bg(colors.background)
            .focus_visible(move |style| {
                style.border_color(colors.ring).shadow(vec![
                    BoxShadow::new(px(0.0), px(0.0), colors.ring.alpha(0.50).into())
                        .spread_radius(px(3.0)),
                ])
            })
    })
}

/// Creates the intrinsic-size Scroll Area content layer.
pub fn scroll_area_content(cx: &App) -> ScrollAreaContent {
    let theme = UiTheme::read(cx).clone();
    ScrollAreaContent::new().style_with_state(move |_state, base| {
        base.font_family(theme.fonts.body.clone())
            .text_size(px(14.0))
            .text_color(theme.colors.foreground)
    })
}

/// Creates a Nova scrollbar for the requested axis. Add `scroll_area_thumb()` as its child.
pub fn scroll_area_scrollbar(orientation: ScrollAreaOrientation, cx: &App) -> ScrollAreaScrollbar {
    let _ = cx;
    ScrollAreaScrollbar::new()
        .orientation(orientation)
        .style_with_state(move |state, base| {
            base.p(px(1.0))
                .when(!state.has_overflow(), |base| base.hidden())
        })
}

/// Creates the pinned rounded scrollbar thumb.
pub fn scroll_area_thumb(cx: &App) -> ScrollAreaThumb {
    let theme = UiTheme::read(cx).clone();
    ScrollAreaThumb::new().style_with_state(move |state, mut style: ScrollbarStyle| {
        style.track_color = theme.colors.background.alpha(0.0).into();
        style.thumb_color = if state.scrolling {
            theme.colors.muted_foreground.alpha(0.72).into()
        } else {
            theme.colors.border.into()
        };
        style.thickness = px(10.0);
        style.inset = px(1.0);
        style.corner_radius = px(99.0);
        style
    })
}

/// Creates the styled corner shared by the two scrollbars.
pub fn scroll_area_corner(cx: &App) -> ScrollAreaCorner {
    let theme = UiTheme::read(cx).clone();
    ScrollAreaCorner::new().style_with_state(move |_state, base| base.bg(theme.colors.background))
}
