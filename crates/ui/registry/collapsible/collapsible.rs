//! Nova presentation for GPUI Kit's controlled collapsible region.
use super::theme::UiTheme;
use gpui_kit::base::Button;
pub use gpui_kit::base::Collapsible;
use gpui_kit::{
    App, Div, ElementId, InteractiveElement as _, StatefulInteractiveElement as _, Styled, Window,
    div,
};

/// Creates a controlled region that reveals and removes its content with theme motion.
pub fn collapsible(
    id: impl Into<ElementId>,
    open: bool,
    window: &mut Window,
    cx: &mut App,
) -> Collapsible {
    let id = id.into();
    let presence = super::theme::presence(id.clone(), open, window, cx);
    let visible = presence.should_render() && (open || presence.progress > 0.);
    let mut region = Collapsible::new().open(visible);
    if visible {
        region = region.reveal(id, presence.progress);
    }
    region
        .flex()
        .flex_col()
        .font_family(UiTheme::read(cx).fonts.body.clone())
}
/// Creates an accessible trigger. Update the caller's open state in its click handler.
pub fn collapsible_trigger(id: impl Into<ElementId>, open: bool, cx: &App) -> Button {
    let theme = UiTheme::read(cx);
    let colors = theme.colors;
    Button::new(id)
        .aria_expanded(open)
        .px(theme.space(2.5))
        .py(theme.space(1.5))
        .rounded(theme.radius.lg)
        .border_1()
        .border_color(colors.background.opacity(0.))
        .text_size(theme.text(14.))
        .text_color(colors.foreground)
        .cursor_pointer()
        .hover(move |s| s.bg(colors.muted))
        .focus_visible(move |s| s.border_color(colors.ring))
}
/// Creates the disclosure content's text treatment.
pub fn collapsible_content(cx: &App) -> Div {
    let theme = UiTheme::read(cx);
    div()
        .w_full()
        .pt(theme.space(2.))
        .overflow_hidden()
        .font_family(theme.fonts.body.clone())
        .text_size(theme.text(14.))
        .text_color(theme.colors.foreground)
}

/// The default height above which [`ShowMore`] collapses its content, as in
/// Mattermost's long posts.
pub const SHOW_MORE_HEIGHT: gpui_kit::Pixels = gpui_kit::px(600.);

/// Whether the content is expanded, and whether it is taller than the limit.
#[derive(Default)]
struct ShowMoreState {
    expanded: bool,
    overflows: bool,
}

type ToggleHandler = std::rc::Rc<dyn Fn(&bool, &mut Window, &mut App)>;

/// Long content clipped to a height with a fade and a "Show more" link,
/// such as a pasted log in a chat message. Content that fits shows as is.
///
/// It remembers whether it is expanded while it stays on screen; pass
/// `expanded` and `on_toggle` to keep that state elsewhere, for example per
/// message while a channel is open.
#[derive(gpui_kit::IntoElement)]
pub struct ShowMore {
    id: ElementId,
    content: gpui_kit::AnyElement,
    max_height: gpui_kit::Pixels,
    expanded: Option<bool>,
    on_toggle: Option<ToggleHandler>,
    fade: Option<gpui_kit::Hsla>,
}

impl ShowMore {
    /// Wraps `content`. Keep `id` stable for the content.
    pub fn new(id: impl Into<ElementId>, content: impl gpui_kit::IntoElement) -> Self {
        Self {
            id: id.into(),
            content: content.into_any_element(),
            max_height: SHOW_MORE_HEIGHT,
            expanded: None,
            on_toggle: None,
            fade: None,
        }
    }
    /// The height above which the content collapses; 600 px by default.
    pub fn max_height(mut self, height: gpui_kit::Pixels) -> Self {
        self.max_height = height;
        self
    }
    /// Controls whether the content is expanded.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = Some(expanded);
        self
    }
    /// Runs with the new expanded state when Show more or Show less is
    /// pressed.
    pub fn on_toggle(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(std::rc::Rc::new(handler));
        self
    }
    /// The color the clipped content fades into: the surface behind it,
    /// the theme background by default.
    pub fn fade_color(mut self, color: impl Into<gpui_kit::Hsla>) -> Self {
        self.fade = Some(color.into());
        self
    }
}

impl gpui_kit::RenderOnce for ShowMore {
    fn render(self, window: &mut Window, cx: &mut App) -> impl gpui_kit::IntoElement {
        use gpui_icons::{LucideIcon, lucide};
        use gpui_kit::base::ElementExt as _;
        use gpui_kit::{ParentElement as _, prelude::FluentBuilder as _};

        use super::theme::{UiText, locale};

        let theme = UiTheme::read(cx).clone();
        let state = window.use_keyed_state((self.id.clone(), "show-more"), cx, |_, _| {
            ShowMoreState::default()
        });
        let (stored, overflows) = {
            let state = state.read(cx);
            (state.expanded, state.overflows)
        };
        let expanded = self.expanded.unwrap_or(stored);
        let limit = self.max_height;
        let fade = self.fade.unwrap_or_else(|| theme.colors.background.into());
        let measured = state.clone();
        let clip = div()
            .relative()
            .when(!expanded, |clip| clip.max_h(limit).overflow_hidden())
            .child(
                div()
                    .on_prepaint(move |bounds, window, cx| {
                        // The content's natural height, measured inside the
                        // clip, decides whether there is more to show.
                        let overflows = bounds.size.height > limit + gpui_kit::px(0.5);
                        if measured.read(cx).overflows != overflows {
                            measured.update(cx, |state, _| state.overflows = overflows);
                            window.refresh();
                        }
                    })
                    .child(self.content),
            )
            .when(overflows && !expanded, |clip| {
                clip.child(
                    div()
                        .absolute()
                        .bottom_0()
                        .left_0()
                        .right_0()
                        .h(theme.space(16.))
                        .bg(gpui_kit::linear_gradient(
                            180.,
                            gpui_kit::linear_color_stop(fade.opacity(0.), 0.),
                            gpui_kit::linear_color_stop(fade, 1.),
                        )),
                )
            });
        let label = locale::text(
            cx,
            if expanded {
                UiText::ShowLess
            } else {
                UiText::ShowMore
            },
        );
        let toggle_state = state.clone();
        let on_toggle = self.on_toggle;
        let colors = theme.colors;
        let focus_theme = theme.clone();
        div()
            .id(self.id.clone())
            .flex()
            .flex_col()
            .child(clip)
            .when(overflows, |column| {
                column.child(
                    div().flex().pt(theme.space(1.)).child(
                        gpui_kit::base::Button::new((self.id.clone(), "toggle"))
                            .aria_label(label.clone())
                            .aria_expanded(expanded)
                            .flex()
                            .items_center()
                            .gap(theme.space(1.))
                            .rounded(theme.radius.sm)
                            .text_size(theme.text(13.))
                            .font_weight(gpui_kit::FontWeight::MEDIUM)
                            .text_color(colors.primary)
                            .cursor_pointer()
                            .hover(|style| style.underline())
                            .focus_visible(move |style| {
                                super::theme::primitives::focus::ring(style, &focus_theme)
                            })
                            .child(
                                lucide(if expanded {
                                    LucideIcon::ChevronUp
                                } else {
                                    LucideIcon::ChevronDown
                                })
                                .size(theme.space(3.5))
                                .text_color(colors.primary),
                            )
                            .child(label)
                            .on_click(move |_, window, cx| {
                                let next = !expanded;
                                toggle_state.update(cx, |state, _| state.expanded = next);
                                if let Some(handler) = &on_toggle {
                                    handler(&next, window, cx);
                                }
                                window.refresh();
                            }),
                    ),
                )
            })
    }
}

#[cfg(test)]
mod show_more_tests {
    use super::ShowMore;
    use gpui_kit::{
        Context, InteractiveElement as _, IntoElement, ParentElement as _, Render, Styled as _,
        TestAppContext, VisualTestContext, Window, div, px, test::TestQueryExt as _,
    };

    struct Post(f32);

    impl Render for Post {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().flex().flex_col().items_start().child(
                div().w(px(400.)).debug_selector(|| "post".into()).child(
                    ShowMore::new("post", div().h(px(self.0)).w_full()).max_height(px(200.)),
                ),
            )
        }
    }

    fn post(height: f32, cx: &mut TestAppContext) -> &mut VisualTestContext {
        cx.update(super::super::theme::init);
        let (_, cx) = cx.add_window_view(move |_, _| Post(height));
        // The first frame measures, the second shows the toggle.
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx
    }

    #[gpui_kit::test]
    fn long_content_collapses_until_show_more(cx: &mut TestAppContext) {
        let cx = post(900., cx);
        let collapsed = cx.debug_bounds("post").unwrap().size.height;
        assert!(
            collapsed < px(300.),
            "clipped near 200 px, got {collapsed:?}"
        );
        let button = cx.update(|window, _| window.find_by_label("Show more"));
        cx.simulate_click(button.bounds().center(), gpui_kit::Modifiers::default());
        cx.update(|window, cx| window.draw(cx).clear(cx));
        assert!(cx.debug_bounds("post").unwrap().size.height > px(900.));
        cx.update(|window, _| {
            window.find_by_label("Show less");
        });
    }

    #[gpui_kit::test]
    fn short_content_has_no_toggle(cx: &mut TestAppContext) {
        let cx = post(120., cx);
        cx.update(|window, _| {
            assert!(window.find_all_by_label("Show more").is_empty());
        });
    }
}
