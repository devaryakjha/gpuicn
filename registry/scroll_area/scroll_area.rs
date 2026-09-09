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
use gpui::{App, InteractiveElement as _, Styled, prelude::FluentBuilder as _, px};

use super::theme::UiTheme;

fn scrollbar_style(theme: &UiTheme, scrolling: bool, mut style: ScrollbarStyle) -> ScrollbarStyle {
    style.track_color = theme.colors.background.opacity(0.0).into();
    style.thumb_color = if scrolling {
        theme.colors.muted_foreground.opacity(0.72).into()
    } else {
        theme.colors.border.into()
    };
    style.thickness = theme.space(2.5);
    style.inset = px(1.0);
    style.corner_radius = (style.thickness - style.inset * 2.0) / 2.0;
    style
}

/// Overlay the same styled vertical scrollbar on a GPUI list or scroll handle.
/// Place beside the scrollable child in a relative container. Base GPUI owns
/// thumb geometry, dragging and track clicks; the supplied target owns scrolling.
pub fn scroll_area_scrollbar_for<H: base_gpui::primitives::ScrollTarget + Clone>(
    id: impl Into<gpui::ElementId>,
    target: &H,
    cx: &App,
) -> base_gpui::primitives::Scrollbar {
    let theme = UiTheme::read(cx).clone();
    base_gpui::primitives::scrollbar_vertical(target)
        .id(id)
        .visibility(base_gpui::primitives::ScrollbarVisibility::Scrolling)
        .style_with_state(move |state, style| scrollbar_style(&theme, state.scrolling, style))
}

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
    let focus_ring = theme.focus_ring();
    ScrollAreaViewport::new().style_with_state(move |_state, base| {
        let colors = theme.colors;
        let focus_ring = focus_ring.clone();
        base.size_full()
            .rounded(theme.radius.lg)
            .focus_visible(move |style| style.border_color(colors.ring).shadow(focus_ring.clone()))
    })
}

/// Creates the intrinsic-size Scroll Area content layer.
pub fn scroll_area_content(cx: &App) -> ScrollAreaContent {
    let theme = UiTheme::read(cx).clone();
    let text_scale = theme.text_scale;
    ScrollAreaContent::new().style_with_state(move |_state, base| {
        base.font_family(theme.fonts.body.clone())
            .text_size(px(14.0) * text_scale)
            .text_color(theme.colors.foreground)
    })
}

/// Creates an idle-hidden scrollbar. Scrolling or hovering the track reveals it.
/// Add `scroll_area_thumb()` as its child.
pub fn scroll_area_scrollbar(orientation: ScrollAreaOrientation, cx: &App) -> ScrollAreaScrollbar {
    let _ = cx;
    ScrollAreaScrollbar::new()
        .orientation(orientation)
        .style_with_state(move |state, base| {
            base.p(px(1.0))
                .opacity(if state.scrolling { 1.0 } else { 0.0 })
                .hover(|style| style.opacity(1.0))
                .when(!state.has_overflow(), |base| base.hidden())
        })
}

/// Creates the pinned rounded scrollbar thumb.
pub fn scroll_area_thumb(cx: &App) -> ScrollAreaThumb {
    let theme = UiTheme::read(cx).clone();
    ScrollAreaThumb::new()
        .style_with_state(move |state, style| scrollbar_style(&theme, state.scrolling, style))
}

/// Creates the styled corner shared by the two scrollbars.
pub fn scroll_area_corner(cx: &App) -> ScrollAreaCorner {
    let theme = UiTheme::read(cx).clone();
    ScrollAreaCorner::new().style_with_state(move |_state, base| base.bg(theme.colors.background))
}
