//! Nova-styled Avatar with native image loading and error fallback.

use gpui_kit::{
    App, ElementId, ImageSource, InteractiveElement as _, IntoElement, ObjectFit, ParentElement,
    RenderOnce, Rgba, SharedString, StatefulInteractiveElement as _, Styled, StyledImage, Window,
    div, px,
};

use gpui_kit::base::TestSupportExt as _;

use super::theme::{UiTheme, primitives::clipped_image_with};

/// A compact, circular Avatar with an optional image and fallback content.
#[derive(IntoElement)]
pub struct Avatar {
    style: gpui_kit::StyleRefinement,
    id: ElementId,
    image: Option<ImageSource>,
    fallback: SharedString,
    aria_label: Option<SharedString>,
    size: AvatarSize,
    status: Option<AvatarStatus>,
    ring_color: Option<Rgba>,
}

/// A presence state drawn as a dot over the avatar's bottom-right corner.
///
/// The colors come from the theme: `success` for online, `warning` for away,
/// `destructive` for do not disturb, and a hollow `muted_foreground` ring for
/// offline. Set those tokens to change every presence dot at once.
#[derive(Clone, Debug, PartialEq)]
pub enum AvatarStatus {
    /// Active now: a filled `success` dot.
    Online,
    /// Idle or away: a filled `warning` dot.
    Away,
    /// Do not disturb: a filled `destructive` dot.
    Dnd,
    /// Not connected: a hollow ring, so it reads without color.
    Offline,
    /// An app-defined state with its own color and the words read after the name.
    Custom {
        /// The dot fill.
        color: Rgba,
        /// The status read after the accessible name, such as "in a meeting".
        label: SharedString,
    },
}

impl AvatarStatus {
    /// The words appended to the avatar's accessible name.
    pub fn label(&self) -> SharedString {
        match self {
            Self::Online => "online".into(),
            Self::Away => "away".into(),
            Self::Dnd => "do not disturb".into(),
            Self::Offline => "offline".into(),
            Self::Custom { label, .. } => label.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
/// Nova avatar dimensions.
pub enum AvatarSize {
    /// Compact dimensions.
    Sm,
    #[default]
    /// The default Nova presentation.
    Default,
    /// Large dimensions.
    Lg,
}

impl Avatar {
    /// Creates a `Avatar` with a stable caller-owned ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            style: gpui_kit::StyleRefinement::default(),
            id: id.into(),
            image: None,
            fallback: SharedString::default(),
            aria_label: None,
            size: AvatarSize::Default,
            status: None,
            ring_color: None,
        }
    }

    /// Sets the text shown while the image is unavailable.
    pub fn fallback(mut self, text: impl Into<SharedString>) -> Self {
        self.fallback = text.into();
        self
    }
    /// Sets the avatar image source.
    pub fn image(mut self, source: impl Into<ImageSource>) -> Self {
        self.image = Some(source.into());
        self
    }
    /// Sets the accessible name of the control.
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }
    /// Selects the Nova control dimensions.
    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = size;
        self
    }
    /// Draws a presence dot at the bottom-right corner, sized to the avatar,
    /// and appends the status to the accessible name ("Ana Lopez, away").
    pub fn status(mut self, status: AvatarStatus) -> Self {
        self.status = Some(status);
        self
    }
    /// Sets the ring drawn behind the presence dot. It should match the
    /// surface under the avatar; the default is the `background` token, so
    /// pass `colors.sidebar` for avatars in a sidebar.
    pub fn ring_color(mut self, color: impl Into<Rgba>) -> Self {
        self.ring_color = Some(color.into());
        self
    }
}

/// The presence dot and its surface ring, sized for an avatar of `size` pixels.
fn status_dot(status: &AvatarStatus, size: f32, ring: Rgba, theme: &UiTheme) -> gpui_kit::Div {
    // 8, 10 and 12px dots for the 24, 32 and 40px avatars.
    let diameter = theme.space(size / 16. + 0.5);
    let ring_width = px(if size < 32. { 1.5 } else { 2. });
    let dot = div().size_full().rounded_full();
    let dot = match status {
        AvatarStatus::Online => dot.bg(theme.colors.success),
        AvatarStatus::Away => dot.bg(theme.colors.warning),
        AvatarStatus::Dnd => dot.bg(theme.colors.destructive),
        AvatarStatus::Custom { color, .. } => dot.bg(*color),
        AvatarStatus::Offline => dot
            .bg(ring)
            .border(ring_width)
            .border_color(theme.colors.muted_foreground),
    };
    div()
        .debug_selector(|| "avatar-status".into())
        .absolute()
        .right(px(-1.))
        .bottom(px(-1.))
        .size(diameter + ring_width * 2.)
        .p(ring_width)
        .rounded_full()
        .bg(ring)
        .child(dot)
}

impl RenderOnce for Avatar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let size = match self.size {
            AvatarSize::Sm => 24.,
            AvatarSize::Default => 32.,
            AvatarSize::Lg => 40.,
        };
        let image_size = theme.space(size / 4.);
        let dot = self.status.as_ref().map(|status| {
            status_dot(
                status,
                size,
                self.ring_color.unwrap_or(theme.colors.background),
                &theme,
            )
        });
        let label = match (self.aria_label, &self.status) {
            (Some(label), Some(status)) => Some(format!("{label}, {}", status.label()).into()),
            (label, _) => label,
        };
        let frame_theme = theme.clone();
        let fallback = move || {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .rounded_full()
                .bg(theme.colors.muted)
                .text_color(theme.colors.muted_foreground)
                .text_size(theme.text(if self.size == AvatarSize::Sm {
                    12.
                } else {
                    14.
                }))
                .child(self.fallback.clone())
                .into_any_element()
        };
        let root = match self.image {
            Some(source) => {
                let loading = fallback.clone();
                clipped_image_with(
                    source,
                    image_size / 2.,
                    ObjectFit::Cover,
                    &frame_theme,
                    |image| image.with_loading(loading).with_fallback(fallback),
                )
            }
            None => div().rounded_full().overflow_hidden().child(fallback()),
        }
        .size_full();
        // The dot sits outside the circular clip, so the frame is a plain box.
        let root = div()
            .id(self.id)
            .test_support()
            .relative()
            .size(image_size)
            .flex_shrink_0()
            .child(root)
            .children(dot);
        let root = if let Some(label) = label {
            root.aria_label(label)
        } else {
            root
        };
        super::theme::apply_style(root, &self.style)
    }
}

impl gpui_kit::Styled for Avatar {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{
        Context, Render, TestAppContext, VisualTestContext, prelude::FluentBuilder as _,
        test::TestQueryExt as _,
    };

    struct People {
        status: Option<AvatarStatus>,
        size: AvatarSize,
    }

    impl Render for People {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().p(px(8.)).child(
                Avatar::new("ana")
                    .aria_label("Ana Lopez")
                    .fallback("AL")
                    .size(self.size)
                    .when_some(self.status.clone(), |avatar, status| avatar.status(status)),
            )
        }
    }

    fn draw(
        status: Option<AvatarStatus>,
        size: AvatarSize,
        cx: &mut TestAppContext,
    ) -> VisualTestContext {
        cx.update(super::super::theme::init);
        let window = cx.add_window(move |_, _| People { status, size });
        let mut cx = VisualTestContext::from_window(window.into(), cx);
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx
    }

    #[gpui_kit::test]
    fn the_status_joins_the_accessible_name(cx: &mut TestAppContext) {
        let mut cx = draw(Some(AvatarStatus::Away), AvatarSize::Default, cx);
        cx.update(|window, _| {
            window.find_by_label("Ana Lopez, away");
        });
        assert!(cx.debug_bounds("avatar-status").is_some());
    }

    #[gpui_kit::test]
    fn no_status_draws_no_dot(cx: &mut TestAppContext) {
        let mut cx = draw(None, AvatarSize::Default, cx);
        cx.update(|window, _| {
            window.find_by_label("Ana Lopez");
        });
        assert!(cx.debug_bounds("avatar-status").is_none());
    }

    #[gpui_kit::test]
    fn the_dot_scales_with_the_avatar_and_sits_on_its_corner(cx: &mut TestAppContext) {
        for (size, avatar, dot) in [
            (AvatarSize::Sm, 24., 8. + 3.),
            (AvatarSize::Default, 32., 10. + 4.),
            (AvatarSize::Lg, 40., 12. + 4.),
        ] {
            let mut cx = draw(Some(AvatarStatus::Offline), size, cx);
            let bounds = cx.debug_bounds("avatar-status").expect("the dot is drawn");
            assert_eq!(bounds.size.width, px(dot), "{size:?}");
            // Bottom-right corner of the avatar at (8, 8), overhanging by 1px.
            assert_eq!(bounds.right(), px(8. + avatar + 1.), "{size:?}");
            assert_eq!(bounds.bottom(), px(8. + avatar + 1.), "{size:?}");
        }
    }

    #[test]
    fn statuses_read_as_words() {
        assert_eq!(AvatarStatus::Dnd.label(), "do not disturb");
        let custom = AvatarStatus::Custom {
            color: gpui_kit::Rgba::default(),
            label: "in a meeting".into(),
        };
        assert_eq!(custom.label(), "in a meeting");
    }
}
