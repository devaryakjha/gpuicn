//! Nova-styled composition around GPUI Kit's native carousel engine.

use super::theme::{UiTheme, apply_style};
use gpui_icons::{LucideIcon, lucide};
use gpui_kit::base::AxisExt as _;
use gpui_kit::component::carousel::{
    Carousel as KitCarousel, CarouselNext as KitCarouselNext,
    CarouselPaginationItem as KitCarouselPaginationItem, CarouselPrevious as KitCarouselPrevious,
};
use gpui_kit::{
    AnyElement, App, ElementId, Entity, IntoElement, ParentElement, RenderOnce, SharedString,
    StyleRefinement, Styled, Window, prelude::FluentBuilder as _,
};

pub use gpui_kit::component::carousel::{
    CarouselContent, CarouselEvent, CarouselItem, CarouselPagination, CarouselState,
};

/// A themed carousel region backed by GPUI Kit's snapping and gesture engine.
#[derive(IntoElement)]
pub struct Carousel {
    id: ElementId,
    state: Entity<CarouselState>,
    label: SharedString,
    style: StyleRefinement,
    children: Vec<AnyElement>,
}

impl Carousel {
    /// Creates a carousel bound to retained caller-owned state.
    pub fn new(id: impl Into<ElementId>, state: &Entity<CarouselState>) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            label: "Carousel".into(),
            style: StyleRefinement::default(),
            children: Vec::new(),
        }
    }

    /// Sets the name announced for the carousel region.
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }
}

impl ParentElement for Carousel {
    fn extend(&mut self, children: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(children);
    }
}

impl Styled for Carousel {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Carousel {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx);
        let root = KitCarousel::new(self.id, &self.state)
            .accessibility_label(self.label)
            .gap(theme.space(3.))
            .font_family(theme.fonts.body.clone())
            .text_color(theme.colors.foreground)
            .children(self.children);
        apply_style(root, &self.style)
    }
}

/// Nova-styled previous-slide control.
#[derive(IntoElement)]
pub struct CarouselPrevious {
    state: Entity<CarouselState>,
    label: SharedString,
    style: StyleRefinement,
}

impl CarouselPrevious {
    /// Creates a previous-slide control bound to the carousel state.
    pub fn new(state: &Entity<CarouselState>) -> Self {
        Self {
            state: state.clone(),
            label: "Previous slide".into(),
            style: StyleRefinement::default(),
        }
    }

    /// Sets the accessible control name.
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }
}

impl Styled for CarouselPrevious {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for CarouselPrevious {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx);
        let state = self.state;
        let snapshot = state.read(cx);
        let axis = snapshot.axis();
        let icon = if axis.is_horizontal() {
            LucideIcon::ChevronLeft
        } else {
            LucideIcon::ChevronUp
        };
        let control = KitCarouselPrevious::new(&state)
            .accessibility_label(self.label)
            .child(
                lucide(icon)
                    .size(theme.space(4.))
                    .text_color(theme.colors.foreground),
            );
        apply_style(control, &self.style)
    }
}

/// Nova-styled next-slide control.
#[derive(IntoElement)]
pub struct CarouselNext {
    state: Entity<CarouselState>,
    label: SharedString,
    style: StyleRefinement,
}

impl CarouselNext {
    /// Creates a next-slide control bound to the carousel state.
    pub fn new(state: &Entity<CarouselState>) -> Self {
        Self {
            state: state.clone(),
            label: "Next slide".into(),
            style: StyleRefinement::default(),
        }
    }

    /// Sets the accessible control name.
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }
}

impl Styled for CarouselNext {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for CarouselNext {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx);
        let state = self.state;
        let snapshot = state.read(cx);
        let axis = snapshot.axis();
        let icon = if axis.is_horizontal() {
            LucideIcon::ChevronRight
        } else {
            LucideIcon::ChevronDown
        };
        let control = KitCarouselNext::new(&state)
            .accessibility_label(self.label)
            .child(
                lucide(icon)
                    .size(theme.space(4.))
                    .text_color(theme.colors.foreground),
            );
        apply_style(control, &self.style)
    }
}

/// Nova dot control for one carousel item.
#[derive(IntoElement)]
pub struct CarouselPaginationItem {
    id: ElementId,
    index: usize,
    state: Entity<CarouselState>,
    label: Option<SharedString>,
    style: StyleRefinement,
}

impl CarouselPaginationItem {
    /// Creates a pagination control for the zero-based item index.
    pub fn new(id: impl Into<ElementId>, index: usize, state: &Entity<CarouselState>) -> Self {
        Self {
            id: id.into(),
            index,
            state: state.clone(),
            label: None,
            style: StyleRefinement::default(),
        }
    }

    /// Replaces the generated “Go to slide N” name.
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl Styled for CarouselPaginationItem {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for CarouselPaginationItem {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx);
        let state = self.state;
        let snapshot = state.read(cx);
        let selected = snapshot.selected_index() == Some(self.index);
        let index = self.index;
        let dot = gpui_kit::div()
            .size(theme.space(2.))
            .rounded(theme.radius.lg)
            .bg(if selected {
                theme.colors.primary
            } else {
                theme.colors.muted_foreground.opacity(0.45)
            });
        let control = KitCarouselPaginationItem::new(self.id, index, &state)
            .when_some(self.label, |control, label| {
                control.accessibility_label(label)
            });
        apply_style(control.child(dot), &self.style)
    }
}
