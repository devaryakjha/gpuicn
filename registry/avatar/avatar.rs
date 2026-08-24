#![allow(missing_docs)]
//! Nova-styled Avatar primitives backed by Base GPUI.

use base_gpui::avatar::{AvatarFallback, AvatarImage, AvatarRoot};
use gpui::{
    AnyElement, App, ElementId, ImageSource, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, px,
};

use super::theme::UiTheme;

/// A compact, circular Avatar with an optional image and fallback content.
#[derive(IntoElement)]
pub struct Avatar {
    id: ElementId,
    image: Option<ImageSource>,
    fallback: Vec<AnyElement>,
    aria_label: Option<SharedString>,
    size: AvatarSize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AvatarSize {
    Sm,
    #[default]
    Default,
    Lg,
}

impl Avatar {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            image: None,
            fallback: Vec::new(),
            aria_label: None,
            size: AvatarSize::Default,
        }
    }

    pub fn image(mut self, source: impl Into<ImageSource>) -> Self {
        self.image = Some(source.into());
        self
    }
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }
    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = size;
        self
    }
}

impl ParentElement for Avatar {
    fn extend(&mut self, children: impl IntoIterator<Item = AnyElement>) {
        self.fallback.extend(children);
    }
}

impl RenderOnce for Avatar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let size = match self.size {
            AvatarSize::Sm => 24.,
            AvatarSize::Default => 32.,
            AvatarSize::Lg => 40.,
        };
        let mut root = AvatarRoot::new()
            .id(self.id)
            .relative()
            .flex_shrink_0()
            .size(px(size))
            .rounded_full()
            .overflow_hidden();
        if let Some(label) = self.aria_label {
            root = root.aria_label(label);
        }
        if let Some(image) = self.image {
            root = root.child(AvatarImage::new(image).size_full().rounded_full());
        }
        root.child(
            AvatarFallback::new()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .rounded_full()
                .bg(theme.colors.muted)
                .text_color(theme.colors.muted_foreground)
                .text_size(px(if self.size == AvatarSize::Sm {
                    12.
                } else {
                    14.
                }))
                .children(self.fallback),
        )
    }
}
