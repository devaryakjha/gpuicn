//! Nova tabs with controlled selection and automatic keyboard activation.
use super::theme::{RovingFocus, UiTheme, apply_style};
use gpui_kit::{
    App, Div, ElementId, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    Role, SharedString, Stateful, StatefulInteractiveElement as _, StyleRefinement, Styled, Window,
    div, prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
/// The visual presentation of the tab list.
pub enum TabsVariant {
    #[default]
    /// The default Nova presentation.
    Default,
    /// Underline the selected tab.
    Line,
}

/// One tab with a stable ID, selection value and visible label.
pub struct Tab {
    id: ElementId,
    value: SharedString,
    label: SharedString,
    disabled: bool,
}
impl Tab {
    /// Creates a `Tab` with a stable caller-owned ID.
    pub fn new(
        id: impl Into<ElementId>,
        value: impl Into<SharedString>,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }
    /// Disables interaction and applies the disabled appearance.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}
type ChangeHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;
#[derive(IntoElement)]
/// A controlled tab list with automatic keyboard selection.
pub struct Tabs {
    id: ElementId,
    selected: Option<SharedString>,
    variant: TabsVariant,
    label: Option<SharedString>,
    items: Vec<Tab>,
    on_change: Option<ChangeHandler>,
    style: StyleRefinement,
}
impl Tabs {
    /// Creates a `Tabs` with a stable caller-owned ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            selected: None,
            variant: Default::default(),
            label: None,
            items: Vec::new(),
            on_change: None,
            style: Default::default(),
        }
    }
    /// Sets the caller-owned selected tab value.
    pub fn selected(mut self, value: impl Into<SharedString>) -> Self {
        self.selected = Some(value.into());
        self
    }
    /// Selects the Nova surface style.
    pub fn variant(mut self, variant: TabsVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Sets the accessible name of the control.
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
    /// Appends an option to the group.
    pub fn item(mut self, item: Tab) -> Self {
        self.items.push(item);
        self
    }
    /// Reports a requested value change; retain the next value in the owning view.
    pub fn on_change(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}
impl Styled for Tabs {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
impl RenderOnce for Tabs {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let selected = self
            .items
            .iter()
            .position(|i| Some(&i.value) == self.selected.as_ref());
        let focus = RovingFocus::new(
            self.id.clone(),
            &self
                .items
                .iter()
                .map(|i| (i.id.clone(), i.disabled))
                .collect::<Vec<_>>(),
            selected,
            window,
            cx,
        );
        let values: Vec<_> = self.items.iter().map(|i| i.value.clone()).collect();
        let count = self.items.len();
        let children = self
            .items
            .into_iter()
            .enumerate()
            .map(|(index, item)| {
                let active = Some(index) == selected;
                let change = self.on_change.clone();
                let ring = theme.focus_ring();
                let handle = focus.handles[index].as_ref();
                gpui_kit::base::Tab::new(item.id)
                    .accessibility_label(item.label.clone())
                    .child(item.label)
                    .selected(active)
                    .disabled(item.disabled)
                    .set_position(index + 1, count)
                    .when_some(handle, |tab, h| tab.track_focus(h))
                    .h(theme.space(6.5))
                    .px(theme.space(1.5))
                    .py(theme.space(0.5))
                    .text_size(theme.text(14.))
                    .text_color(if active {
                        colors.foreground
                    } else {
                        colors.muted_foreground
                    })
                    .when(self.variant == TabsVariant::Default, |tab| {
                        tab.rounded(theme.radius.sm).when(active, |tab| {
                            tab.bg(colors.background).shadow(theme.shadows.sm.clone())
                        })
                    })
                    .when(self.variant == TabsVariant::Line, |tab| {
                        tab.border_b_2().border_color(if active {
                            colors.foreground
                        } else {
                            colors.background.opacity(0.)
                        })
                    })
                    .when(item.disabled, |tab| tab.opacity(0.5).cursor_not_allowed())
                    .when(!item.disabled, |tab| tab.cursor_pointer())
                    .focus_visible(move |s| s.shadow(ring.clone()))
                    .when_some(change, |tab, handler| {
                        tab.on_click(move |_, window, cx| handler(item.value.clone(), window, cx))
                    })
            })
            .collect::<Vec<_>>();
        let on_change = self.on_change;
        let root = gpui_kit::base::Tabs::new(self.id)
            .flex()
            .items_center()
            .when_some(self.label, |root, label| root.aria_label(label))
            .when(self.variant == TabsVariant::Default, |root| {
                root.h(theme.space(8.))
                    .rounded(theme.radius.lg)
                    .p(theme.space(0.75))
                    .bg(colors.muted)
            })
            .when(self.variant == TabsVariant::Line, |root| {
                root.gap(theme.space(1.))
                    .border_b_1()
                    .border_color(colors.border)
            })
            .on_key_down(move |event, window, cx| {
                if let Some(index) = focus.key(event, Some(gpui_kit::Axis::Horizontal), window, cx)
                    && let Some(change) = &on_change
                {
                    change(values[index].clone(), window, cx);
                }
            })
            .children(children);
        apply_style(root, &self.style)
    }
}
/// Mount the selected content explicitly in the caller's render function.
pub fn tabs_content(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    cx: &App,
) -> Stateful<Div> {
    let theme = UiTheme::read(cx);
    div()
        .id(id)
        .role(Role::TabPanel)
        .aria_label(label)
        .tab_index(0)
        .flex_1()
        .font_family(theme.fonts.body.clone())
        .text_size(px(14.) * theme.text_scale)
        .text_color(theme.colors.foreground)
}
