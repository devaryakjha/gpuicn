//! Shared visual recipes between the theme tokens and the components.
//!
//! shadcn expresses these as Tailwind class strings; here each one is a function
//! that applies the same styles to any GPUI element, so every component draws
//! surfaces, controls, focus, text and images the same way, and a change to a
//! recipe reaches the whole catalog. The registry lint fails components that
//! rewrite one of these by hand. Everything takes `&UiTheme` so it works in
//! `RenderOnce` and `Render` alike without re-reading the global.

use gpui_icons::{LucideIcon, lucide};
use gpui_kit::{
    AnyElement, Axis, BoxShadow, Div, FontWeight, ImageSource, IntoElement, MouseButton, ObjectFit,
    ParentElement as _, Pixels, Rgba, StatefulInteractiveElement, StyleRefinement, Styled,
    StyledImage as _, Window, div, img, prelude::FluentBuilder as _, px,
};

use super::{UiTextStyle, UiTheme};

/// Text sizes from the type scale, each with its line height and the body font.
pub mod text {
    use super::*;

    /// Applies one step of the type scale and the body font.
    pub fn style<T: Styled>(element: T, theme: &UiTheme, style: UiTextStyle) -> T {
        let style = theme.text_style(style);
        element
            .font_family(theme.fonts.body.clone())
            .text_size(style.size)
            .line_height(style.line_height)
    }
    /// Captions, key caps and timestamps.
    pub fn xs<T: Styled>(element: T, theme: &UiTheme) -> T {
        style(element, theme, theme.typography.xs)
    }
    /// Controls and body copy; the Nova default.
    pub fn sm<T: Styled>(element: T, theme: &UiTheme) -> T {
        style(element, theme, theme.typography.sm)
    }
    /// Prose and dialog titles.
    pub fn base<T: Styled>(element: T, theme: &UiTheme) -> T {
        style(element, theme, theme.typography.base)
    }
    /// Section headings.
    pub fn lg<T: Styled>(element: T, theme: &UiTheme) -> T {
        style(element, theme, theme.typography.lg)
    }
    /// Page headings.
    pub fn xl<T: Styled>(element: T, theme: &UiTheme) -> T {
        style(element, theme, theme.typography.xl)
    }
    /// A medium-weight heading in the heading font at the given step.
    pub fn heading<T: Styled>(element: T, theme: &UiTheme, size: UiTextStyle) -> T {
        style(element, theme, size)
            .font_family(theme.fonts.heading.clone())
            .font_weight(FontWeight::MEDIUM)
    }
    /// Switches to the monospace font, keeping the current size.
    pub fn mono<T: Styled>(element: T, theme: &UiTheme) -> T {
        element.font_family(theme.fonts.mono.clone())
    }
    /// Muted supporting text at the small size.
    pub fn muted<T: Styled>(element: T, theme: &UiTheme) -> T {
        sm(element, theme).text_color(theme.colors.muted_foreground)
    }
}

/// Icon sizes in spacing units, so icons scale with control density.
pub mod icon {
    use super::*;

    /// 12px: inside badges and tags.
    pub fn xs(theme: &UiTheme) -> Pixels {
        theme.space(3.)
    }
    /// 14px: inside small buttons and chips.
    pub fn sm(theme: &UiTheme) -> Pixels {
        theme.space(3.5)
    }
    /// 16px: the default inside controls and menu items.
    pub fn md(theme: &UiTheme) -> Pixels {
        theme.space(4.)
    }
    /// 20px: list leading icons.
    pub fn lg(theme: &UiTheme) -> Pixels {
        theme.space(5.)
    }
    /// 24px: empty states and headers.
    pub fn xl(theme: &UiTheme) -> Pixels {
        theme.space(6.)
    }
}

/// Keyboard focus treatment: shadcn Nova's `ring-3 ring-ring/50` outside the
/// border, with the border taking the ring color. Apply through
/// `focus_visible` so pointer presses leave it alone.
pub mod focus {
    use super::*;

    /// The ring as a box shadow, for elements that draw their own border.
    pub fn shadow(color: Rgba) -> Vec<BoxShadow> {
        vec![BoxShadow::new(px(0.), px(0.), color.into()).spread_radius(px(3.))]
    }
    /// The focus ring for an ordinary control.
    pub fn ring(style: StyleRefinement, theme: &UiTheme) -> StyleRefinement {
        let control = theme.control();
        style
            .border_color(theme.colors.ring)
            .shadow(shadow(control.ring))
    }
    /// The focus ring for an invalid control, in the destructive color.
    pub fn invalid_ring(style: StyleRefinement, theme: &UiTheme) -> StyleRefinement {
        let control = theme.control();
        style
            .border_color(theme.colors.destructive)
            .shadow(shadow(control.invalid_ring))
    }
    /// Picks the valid or invalid ring.
    pub fn ring_for(style: StyleRefinement, theme: &UiTheme, invalid: bool) -> StyleRefinement {
        if invalid {
            invalid_ring(style, theme)
        } else {
            ring(style, theme)
        }
    }
}

/// Disabled and interactive cursor treatment.
pub mod disabled {
    use super::*;

    /// Nova's `disabled:opacity-50` plus the blocked cursor.
    pub fn style<T: Styled>(element: T) -> T {
        element.opacity(0.5).cursor_not_allowed()
    }
    /// A pointer cursor when enabled, the disabled treatment otherwise.
    pub fn interactive<T: Styled>(element: T, disabled: bool) -> T {
        if disabled {
            style(element)
        } else {
            element.cursor_pointer()
        }
    }
}

/// Elevated surfaces. One rule per kind, so every popover, menu, dialog, card
/// and tooltip in the catalog shares border, radius, shadow and text defaults.
pub mod surface {
    use super::*;

    fn popover_base<T: Styled>(element: T, theme: &UiTheme) -> T {
        text::sm(element, theme)
            .border_1()
            .border_color(theme.control().surface_border)
            .bg(theme.colors.popover)
            .text_color(theme.colors.popover_foreground)
    }
    /// Popovers, hover cards, select and combobox lists: `rounded-lg shadow-md`.
    pub fn popover<T: Styled>(element: T, theme: &UiTheme) -> T {
        popover_base(element, theme)
            .rounded(theme.radius.lg)
            .shadow(theme.shadows.md.to_vec())
    }
    /// Menus and context menus: a popover with Nova's 4px inset.
    pub fn menu<T: Styled>(element: T, theme: &UiTheme) -> T {
        popover(element, theme).p(theme.space(1.))
    }
    /// Dialogs and command palettes: `rounded-xl shadow-lg`.
    pub fn modal<T: Styled>(element: T, theme: &UiTheme) -> T {
        popover_base(element, theme)
            .rounded(theme.radius.xl)
            .shadow(theme.shadows.lg.to_vec())
    }
    /// A sheet or drawer panel: the modal surface without a radius, which the
    /// caller rounds on the open edge.
    pub fn sheet<T: Styled>(element: T, theme: &UiTheme) -> T {
        popover_base(element, theme).border_color(theme.colors.border)
    }
    /// Toasts: `rounded-2xl` on the popover surface with the plain border.
    pub fn toast<T: Styled>(element: T, theme: &UiTheme) -> T {
        popover_base(element, theme)
            .rounded(theme.radius.two_xl)
            .border_color(theme.colors.border)
            .shadow(theme.shadows.lg.to_vec())
    }
    /// Cards and empty states: `rounded-xl border shadow-sm` on the card color.
    pub fn card<T: Styled>(element: T, theme: &UiTheme) -> T {
        text::sm(element, theme)
            .rounded(theme.radius.xl)
            .border_1()
            .border_color(theme.colors.border)
            .bg(theme.colors.card)
            .text_color(theme.colors.card_foreground)
            .shadow(theme.shadows.sm.to_vec())
    }
    /// A floating action bar over content, as on a hovered message.
    pub fn floating<T: Styled>(element: T, theme: &UiTheme) -> T {
        element
            .rounded(theme.radius.lg)
            .border_1()
            .border_color(theme.colors.border)
            .bg(theme.colors.background)
            .shadow(theme.shadows.sm.to_vec())
    }
    /// Tooltips: inverted colors at the small radius and caption size.
    pub fn tooltip<T: Styled>(element: T, theme: &UiTheme) -> T {
        text::xs(element, theme)
            .rounded(theme.radius.sm)
            .px(theme.space(3.))
            .py(theme.space(1.5))
            .bg(theme.colors.foreground)
            .text_color(theme.colors.background)
    }
    /// A one-pixel rule in the border color, across or down.
    pub fn divider<T: Styled>(element: T, theme: &UiTheme, axis: Axis) -> T {
        let element = match axis {
            Axis::Horizontal => element.w_full().h(px(1.)),
            Axis::Vertical => element.h_full().w(px(1.)),
        };
        element.bg(theme.colors.border)
    }
}

/// Form control pieces shared by single controls and their grouped variants.
pub mod control {
    use super::*;

    /// Corner radius of a 16px check box: Nova's `rounded-[4px]` at the default radius.
    pub fn box_radius(theme: &UiTheme) -> Pixels {
        theme.radius.sm * (2. / 3.)
    }

    /// The frame every text-like control shares: inputs, selects, number and
    /// OTP cells. Height 32px, 10px inset, large radius, input border.
    pub fn frame<T: Styled>(element: T, theme: &UiTheme, invalid: bool) -> T {
        let control = theme.control();
        text::sm(element, theme)
            .h(theme.space(8.))
            .px(theme.space(2.5))
            .rounded(theme.radius.lg)
            .border_1()
            .border_color(if invalid {
                theme.colors.destructive
            } else {
                control.border
            })
            .bg(control.background)
            .text_color(theme.colors.foreground)
    }

    /// A 16px check box with its mark, for checkboxes and grouped checkboxes.
    pub fn checkbox_box(theme: &UiTheme, checked: bool, indeterminate: bool) -> Div {
        let colors = theme.colors;
        let control = theme.control();
        let selected = checked || indeterminate;
        div()
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .size(theme.space(4.))
            .rounded(box_radius(theme))
            .border_1()
            .border_color(if selected {
                colors.primary
            } else {
                control.border
            })
            .bg(if selected {
                colors.primary
            } else {
                control.background
            })
            .when(selected, |root| {
                root.child(
                    lucide(if indeterminate {
                        LucideIcon::Minus
                    } else {
                        LucideIcon::Check
                    })
                    .size(icon::sm(theme))
                    .text_color(colors.primary_foreground),
                )
            })
    }

    /// A 16px radio circle with its dot.
    pub fn radio_dot(theme: &UiTheme, checked: bool) -> Div {
        let colors = theme.colors;
        let control = theme.control();
        div()
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .size(theme.space(4.))
            .rounded_full()
            .border_1()
            .border_color(if checked {
                colors.primary
            } else {
                control.border
            })
            .bg(if checked {
                colors.primary
            } else {
                control.background
            })
            .when(checked, |root| {
                root.child(
                    div()
                        .size(theme.space(2.))
                        .rounded_full()
                        .bg(colors.primary_foreground),
                )
            })
    }

    /// The toggle look shared by Toggle and Toggle Group items: transparent at
    /// rest, muted when hovered or pressed, an input border when outlined.
    pub fn toggle<T: Styled>(
        element: T,
        theme: &UiTheme,
        pressed: bool,
        disabled: bool,
        outline: bool,
    ) -> T {
        let colors = theme.colors;
        let control = theme.control();
        let element = text::sm(element, theme)
            .border_1()
            .border_color(if outline {
                control.border
            } else {
                colors.background.opacity(0.)
            })
            .text_color(colors.foreground)
            .bg(if pressed {
                control.pressed
            } else {
                colors.background.opacity(0.)
            });
        disabled::interactive(element, disabled)
    }
}

/// An image inside a rounded frame. GPUI clips children to rectangles, so the
/// frame rounds itself and the image rounds to the frame's inner radius; the
/// image fills the frame's box and never sizes the frame from its aspect ratio.
pub fn clipped_image(
    source: impl Into<ImageSource>,
    radius: Pixels,
    fit: ObjectFit,
    theme: &UiTheme,
) -> Div {
    div()
        .relative()
        .flex_shrink_0()
        .overflow_hidden()
        .rounded(radius)
        .bg(theme.colors.muted)
        .child(
            img(source)
                .absolute()
                .inset_0()
                .size_full()
                .rounded(radius)
                .object_fit(fit),
        )
}

/// A circular image, as in avatars.
pub fn round_image(source: impl Into<ImageSource>, size: Pixels, theme: &UiTheme) -> Div {
    clipped_image(source, size / 2., ObjectFit::Cover, theme).size(size)
}

/// Dismisses an overlay from a press outside it or the Escape key.
pub mod dismiss {
    use super::*;

    /// Runs `handler` when the pointer presses outside `element` or Escape is
    /// pressed while focus is inside it.
    pub fn on_outside<E: StatefulInteractiveElement>(
        element: E,
        handler: impl Fn(&mut Window, &mut gpui_kit::App) + 'static,
    ) -> E {
        let handler = std::rc::Rc::new(handler);
        let on_press = handler.clone();
        element
            .on_mouse_down_out(move |event, window, cx| {
                if event.button == MouseButton::Left {
                    on_press(window, cx)
                }
            })
            .on_key_down(move |event, window, cx| {
                if event.keystroke.key == "escape" {
                    handler(window, cx);
                    cx.stop_propagation();
                }
            })
    }
}

/// Hover-revealed content that never changes layout: the element keeps its
/// size and only its opacity follows `visible`.
pub fn hover_reveal<T: Styled>(element: T, visible: bool) -> T {
    element.opacity(if visible { 1. } else { 0. })
}

/// Erases an element's type after styling, for callers that store children.
pub fn erase(element: impl IntoElement) -> AnyElement {
    element.into_any_element()
}
