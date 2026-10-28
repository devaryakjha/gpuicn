//! Nova toolbar with roving focus and GPUI Kit buttons and editors.
use super::{
    input::{Input, InputState},
    theme::UiTheme,
};
use gpui_kit::base::Button as BaseButton;
use gpui_kit::{
    AnyElement, App, Axis, ClickEvent, Div, ElementId, Entity, FocusHandle, Focusable as _,
    InteractiveElement as _, IntoElement, ParentElement, RenderOnce, Role, SharedString,
    StatefulInteractiveElement as _, StyleRefinement, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};

#[derive(IntoElement)]
/// A named group of buttons and editors with roving keyboard focus.
pub struct Toolbar {
    id: ElementId,
    base: Div,
    label: SharedString,
    axis: Axis,
    children: Vec<ToolbarItem>,
}
enum ToolbarItem {
    Button(Box<ToolbarButton>),
    Input(Entity<InputState>, SharedString, bool),
    Separator,
}
impl Toolbar {
    /// Creates a `Toolbar` with a stable caller-owned ID.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, cx: &App) -> Self {
        let theme = UiTheme::read(cx);
        Self {
            id: id.into(),
            label: label.into(),
            axis: Axis::Horizontal,
            children: Vec::new(),
            base: div()
                .flex()
                .items_center()
                .gap(theme.space(1.))
                .p(theme.space(1.))
                .border_1()
                .border_color(theme.colors.border)
                .rounded(theme.radius.lg)
                .bg(theme.colors.background)
                .font_family(theme.fonts.body.clone()),
        }
    }
    /// Selects horizontal or vertical layout and keyboard navigation.
    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }
    /// Appends a toolbar action.
    pub fn button(mut self, button: ToolbarButton) -> Self {
        self.children.push(ToolbarItem::Button(Box::new(button)));
        self
    }
    /// Appends a retained input with its accessible label and disabled state.
    pub fn input(
        mut self,
        state: &Entity<InputState>,
        label: impl Into<SharedString>,
        disabled: bool,
    ) -> Self {
        self.children
            .push(ToolbarItem::Input(state.clone(), label.into(), disabled));
        self
    }
    /// Appends a divider between toolbar controls.
    pub fn separator(mut self) -> Self {
        self.children.push(ToolbarItem::Separator);
        self
    }
}
impl Styled for Toolbar {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

#[derive(IntoElement)]
/// A Nova toolbar action backed by a Kit button.
pub struct ToolbarButton {
    id: ElementId,
    button: BaseButton,
    disabled: bool,
}
impl ToolbarButton {
    /// Creates a `ToolbarButton` with a stable caller-owned ID.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, cx: &App) -> Self {
        let id = id.into();
        let theme = UiTheme::read(cx);
        let colors = theme.colors;
        Self {
            id: id.clone(),
            disabled: false,
            button: BaseButton::new(id)
                .accessibility_label(label)
                .h(theme.space(7.))
                .px(theme.space(2.))
                .rounded(theme.radius.sm)
                .border_1()
                .border_color(colors.background.opacity(0.))
                .text_size(theme.text(14.))
                .text_color(colors.foreground)
                .cursor_pointer()
                .hover(move |s| s.bg(colors.muted))
                .focus_visible(move |s| s.border_color(colors.ring)),
        }
    }
    /// Disables interaction and applies the disabled appearance.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    /// Handles activation by pointer, keyboard or an accessibility action.
    pub fn on_click(
        mut self,
        callback: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.button = self.button.on_click(callback);
        self
    }
}
impl Styled for ToolbarButton {
    fn style(&mut self) -> &mut StyleRefinement {
        self.button.style()
    }
}
impl ParentElement for ToolbarButton {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.button.extend(elements);
    }
}
impl RenderOnce for ToolbarButton {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.button
            .disabled(self.disabled)
            .when(self.disabled, |b| b.opacity(0.5).cursor_not_allowed())
    }
}

impl RenderOnce for Toolbar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // Keep the last active item as the one tab stop when focus leaves the toolbar.
        let active =
            window.use_keyed_state((self.id.clone(), "active"), cx, |_, _| None::<FocusHandle>);
        let mut handles = Vec::new();
        let mut inputs = Vec::new();
        let mut item_focus = Vec::new();
        for item in &self.children {
            let (focus, disabled, input) = match item {
                ToolbarItem::Button(button) => (
                    window
                        .use_keyed_state((button.id.clone(), "focus"), cx, |_, cx| {
                            cx.focus_handle()
                        })
                        .read(cx)
                        .clone(),
                    button.disabled,
                    None,
                ),
                ToolbarItem::Input(state, _, disabled) => (
                    state.read(cx).focus_handle(cx),
                    *disabled,
                    Some(state.clone()),
                ),
                ToolbarItem::Separator => {
                    item_focus.push(None);
                    continue;
                }
            };
            item_focus.push(Some(focus.clone()));
            if !disabled {
                handles.push(focus);
                inputs.push(input);
            } else {
                focus.tab_stop(false);
            }
        }
        let selected = handles
            .iter()
            .position(|h| h.is_focused(window))
            .or_else(|| {
                handles
                    .iter()
                    .position(|h| Some(h) == active.read(cx).as_ref())
            })
            .unwrap_or(0);
        if let Some(focus) = handles.get(selected) {
            active.update(cx, |value, _| *value = Some(focus.clone()));
        }
        for (index, focus) in handles.iter().enumerate() {
            focus.clone().tab_stop(index == selected);
        }
        let theme = UiTheme::read(cx);
        let color = theme.colors.border;
        let axis = self.axis;
        let children = self
            .children
            .into_iter()
            .zip(item_focus)
            .map(|(item, focus)| match item {
                ToolbarItem::Button(mut button) => {
                    let focus = focus.unwrap().clone();
                    button.button = button.button.track_focus(&focus).tab_stop(focus.tab_stop);
                    (*button).into_any_element()
                }
                ToolbarItem::Input(state, label, disabled) => Input::new(&state)
                    .aria_label(label)
                    .disabled(disabled)
                    .w(theme.space(30.))
                    .h(theme.space(7.))
                    .into_any_element(),
                ToolbarItem::Separator => div()
                    .bg(color)
                    .when(axis == Axis::Horizontal, |d| d.w(px(1.)).h(theme.space(4.)))
                    .when(axis == Axis::Vertical, |d| d.h(px(1.)).w_full())
                    .into_any_element(),
            });
        self.base
            .id(self.id)
            .role(Role::Toolbar)
            .aria_label(self.label)
            .aria_orientation(if axis == Axis::Horizontal {
                gpui_kit::Orientation::Horizontal
            } else {
                gpui_kit::Orientation::Vertical
            })
            .when(axis == Axis::Vertical, |d| d.flex_col())
            .capture_key_down(move |event, window, cx| {
                if event.keystroke.modifiers.modified() {
                    return;
                }
                let key = event.keystroke.key.as_str();
                let Some(current) = handles.iter().position(|h| h.is_focused(window)) else {
                    return;
                };
                let movement: isize = match (axis, key) {
                    (Axis::Horizontal, "left") | (Axis::Vertical, "up") => -1,
                    (Axis::Horizontal, "right") | (Axis::Vertical, "down") => 1,
                    (_, "home") => -2,
                    (_, "end") => 2,
                    _ => return,
                };
                if let Some(state) = &inputs[current] {
                    let state = state.read(cx);
                    if movement.abs() == 2 {
                        return;
                    }
                    if axis == Axis::Horizontal
                        && (!state.selected_range().is_empty()
                            || (movement < 0 && state.cursor() != 0)
                            || (movement > 0 && state.cursor() != state.text().len()))
                    {
                        return;
                    }
                }
                let next = match movement {
                    -2 => 0,
                    2 => handles.len() - 1,
                    _ => (current as isize + movement).rem_euclid(handles.len() as isize) as usize,
                };
                handles[current].clone().tab_stop(false);
                let target = handles[next].clone().tab_stop(true);
                target.focus(window, cx);
                if let Some(state) = &inputs[next] {
                    state.update(cx, |state, cx| state.select_all(window, cx));
                }
                active.update(cx, |value, _| *value = Some(target));
                window.refresh();
                cx.stop_propagation();
            })
            .children(children)
    }
}
