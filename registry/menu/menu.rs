//! Nova menus composed from Kit buttons and popup positioning.
use super::{
    button::{ButtonSize, ButtonVariant, style_button},
    theme::{UiTheme, apply_style},
};
use gpui_icons::{LucideIcon, lucide};
use gpui_kit::base::{Align, Button, ElementExt as _, Placement, Positioner};
use gpui_kit::{
    AnyElement, App, Bounds, ClickEvent, Context, ElementId, Entity, FocusHandle, Focusable,
    InteractiveElement as _, IntoElement, MouseButton, ParentElement as _, Pixels, Point, Render,
    RenderOnce, Role, SharedString, StatefulInteractiveElement as _, StyleRefinement, Styled,
    Window, div, prelude::FluentBuilder as _,
};
use std::{rc::Rc, time::Duration};

type Handler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
#[derive(Clone)]
/// An action, check item, radio item, separator or submenu entry.
pub struct MenuItem {
    id: ElementId,
    label: SharedString,
    disabled: bool,
    checked: Option<bool>,
    radio: bool,
    link: bool,
    handler: Option<Handler>,
    children: Vec<MenuItem>,
    separator: bool,
}
impl MenuItem {
    /// Creates an action with a caller-owned ID, unique within this menu tree.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            disabled: false,
            checked: None,
            radio: false,
            link: false,
            handler: None,
            children: Vec::new(),
            separator: false,
        }
    }
    /// Disables interaction and applies the disabled appearance.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    /// Makes this a checkbox item with the supplied initial state.
    pub fn checked(mut self, value: bool) -> Self {
        self.checked = Some(value);
        self
    }
    /// Makes this a radio item; activating it clears radio siblings at this menu level.
    pub fn radio(mut self, value: bool) -> Self {
        self.checked = Some(value);
        self.radio = true;
        self
    }
    /// Handles activation by pointer, keyboard or an accessibility action.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.handler = Some(Rc::new(handler));
        self
    }
    /// Marks a link; the caller handles navigation through `on_click`.
    pub fn link(mut self) -> Self {
        self.link = true;
        self
    }
    /// Creates an item containing another level of menu entries.
    pub fn submenu(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        items: impl IntoIterator<Item = MenuItem>,
    ) -> Self {
        let mut item = Self::new(id, label);
        item.children = items.into_iter().collect();
        item
    }
    /// Creates a non-interactive separator between menu entries.
    pub fn separator() -> Self {
        let mut item = Self::new("separator", "");
        item.separator = true;
        item
    }
    /// Creates a non-interactive menu heading.
    pub fn label(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id, label).disabled(true)
    }
    fn enabled(&self) -> bool {
        !self.disabled && !self.separator
    }
}

/// A committed leaf action, including its new checked state when applicable.
#[derive(Clone, Debug)]
pub struct MenuEvent {
    /// Indices from the root menu to the activated leaf.
    pub path: Vec<usize>,
    /// The committed checked state, or `None` for a plain action.
    pub checked: Option<bool>,
}
impl gpui_kit::EventEmitter<MenuEvent> for MenuState {}

/// Retained popup visibility, item state and keyboard navigation. Emits `MenuEvent` on leaf activation.
pub struct MenuState {
    items: Vec<MenuItem>,
    open: bool,
    focus: FocusHandle,
    trigger: FocusHandle,
    return_focus: Option<FocusHandle>,
    anchor: Bounds<Pixels>,
    pointer: bool,
    path: Vec<usize>,
    highlighted: Vec<Option<usize>>,
    row_bounds: Vec<Vec<Bounds<Pixels>>>,
    popup_bounds: Vec<Bounds<Pixels>>,
    query: String,
    typed_at: Option<web_time::Instant>,
}
impl MenuState {
    /// Creates a closed menu with the supplied entries and new focus handles.
    pub fn new(items: impl IntoIterator<Item = MenuItem>, cx: &mut Context<Self>) -> Self {
        Self {
            items: items.into_iter().collect(),
            open: false,
            focus: cx.focus_handle().tab_stop(false),
            trigger: cx.focus_handle().tab_stop(true),
            return_focus: None,
            anchor: Default::default(),
            pointer: false,
            path: Vec::new(),
            highlighted: Vec::new(),
            row_bounds: Vec::new(),
            popup_bounds: Vec::new(),
            query: String::new(),
            typed_at: None,
        }
    }
    /// Returns whether the popup is open.
    pub fn is_open(&self) -> bool {
        self.open
    }
    /// Replaces the menu entries and resets submenu navigation.
    pub fn set_items(&mut self, items: impl IntoIterator<Item = MenuItem>, cx: &mut Context<Self>) {
        self.items = items.into_iter().collect();
        self.path.clear();
        self.highlighted = vec![self.items.iter().position(MenuItem::enabled)];
        cx.notify();
    }
    /// Returns the handle used by the menu trigger.
    pub fn trigger_focus(&self) -> FocusHandle {
        self.trigger.clone()
    }
    /// Opens the menu at its trigger and transfers keyboard focus into it.
    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open_at(None, window, cx);
    }
    /// Opens at a pointer position, or at the trigger when the position is absent.
    pub fn open_at(
        &mut self,
        point: Option<Point<Pixels>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.pointer = point.is_some();
        self.return_focus = if self.pointer {
            window.focused(cx)
        } else {
            None
        };
        if let Some(point) = point {
            self.anchor = Bounds::new(point, gpui_kit::size(gpui_kit::px(0.), gpui_kit::px(0.)));
        }
        self.open = true;
        self.path.clear();
        self.query.clear();
        self.highlighted = vec![self.items.iter().position(MenuItem::enabled)];
        self.focus.focus(window, cx);
        cx.notify();
    }
    /// Closes and optionally restores the prior context focus or menu trigger.
    pub fn close(&mut self, restore: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.open = false;
        self.path.clear();
        if restore {
            self.return_focus
                .take()
                .unwrap_or_else(|| self.trigger.clone())
                .focus(window, cx);
        }
        cx.notify();
    }
    pub(crate) fn submenu_key(&self, right: bool) -> bool {
        if !right {
            return !self.path.is_empty();
        }
        self.highlighted
            .last()
            .copied()
            .flatten()
            .is_some_and(|i| !self.level(self.path.len())[i].children.is_empty())
    }
    fn level(&self, depth: usize) -> &[MenuItem] {
        let mut items = self.items.as_slice();
        for &index in self.path.iter().take(depth) {
            items = &items[index].children;
        }
        items
    }
    fn highlight(&mut self, depth: usize, index: usize, cx: &mut Context<Self>) {
        if !self.level(depth)[index].enabled() {
            return;
        }
        if self.highlighted.get(depth).copied().flatten() == Some(index) {
            return;
        }
        self.path.truncate(depth);
        self.highlighted.truncate(depth + 1);
        self.highlighted[depth] = Some(index);
        cx.notify();
    }
    fn activate(
        &mut self,
        depth: usize,
        index: usize,
        event: &ClickEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let item = self.level(depth)[index].clone();
        if !item.enabled() {
            return;
        }
        if !item.children.is_empty() {
            self.path.truncate(depth);
            self.path.push(index);
            self.highlighted.truncate(depth + 1);
            self.highlighted
                .push(item.children.iter().position(MenuItem::enabled));
            cx.notify();
        } else {
            let mut path = self.path[..depth].to_vec();
            path.push(index);
            let checked = item.checked.map(|value| item.radio || !value);
            if let Some(checked) = checked {
                let mut items = &mut self.items;
                for &parent in &path[..depth] {
                    items = &mut items[parent].children;
                }
                if item.radio {
                    for sibling in items.iter_mut().filter(|item| item.radio) {
                        sibling.checked = Some(false);
                    }
                }
                items[index].checked = Some(checked);
            }
            self.close(true, window, cx);
            cx.emit(MenuEvent { path, checked });
            if let Some(handler) = item.handler {
                handler(event, window, cx);
            }
        }
    }
    fn key(&mut self, event: &gpui_kit::KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.open {
            return;
        }
        let depth = self.path.len();
        let key = event.keystroke.key.as_str();
        if key == "escape" {
            self.close(true, window, cx);
        } else if key == "tab" {
            self.close(true, window, cx);
            return;
        } else if event.keystroke.modifiers.modified() {
            return;
        } else if key == "left" && depth > 0 {
            self.path.pop();
            self.highlighted.pop();
            cx.notify();
        } else if matches!(key, "enter" | "space" | "right") {
            if let Some(index) = self.highlighted[depth]
                && (key != "right" || !self.level(depth)[index].children.is_empty())
            {
                self.activate(
                    depth,
                    index,
                    &ClickEvent::Keyboard(Default::default()),
                    window,
                    cx,
                );
            }
        } else {
            let items = self.level(depth);
            let enabled: Vec<_> = items
                .iter()
                .enumerate()
                .filter_map(|(i, item)| item.enabled().then_some(i))
                .collect();
            if enabled.is_empty() {
                return;
            }
            let current = enabled
                .iter()
                .position(|i| Some(*i) == self.highlighted[depth])
                .unwrap_or(0);
            let next = match key {
                "up" => Some(enabled[(current + enabled.len() - 1) % enabled.len()]),
                "down" => Some(enabled[(current + 1) % enabled.len()]),
                "home" => enabled.first().copied(),
                "end" => enabled.last().copied(),
                _ if key.chars().count() == 1 => {
                    let now = web_time::Instant::now();
                    if self
                        .typed_at
                        .is_none_or(|time| now.duration_since(time) > Duration::from_millis(700))
                    {
                        self.query.clear();
                    }
                    self.typed_at = Some(now);
                    self.query.push_str(key);
                    let query = self.query.to_lowercase();
                    enabled.into_iter().find(|&i| {
                        self.level(depth)[i]
                            .label
                            .to_lowercase()
                            .starts_with(&query)
                    })
                }
                _ => return,
            };
            if let Some(index) = next {
                self.highlight(depth, index, cx);
            }
        }
        cx.stop_propagation();
    }
}
impl Focusable for MenuState {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
impl Render for MenuState {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = UiTheme::read(cx).clone();
        let mut root = div()
            .id(("menu", cx.entity_id()))
            .track_focus(&self.focus)
            .capture_key_down(cx.listener(Self::key));
        if !self.open {
            return root;
        }
        self.row_bounds.resize_with(self.path.len() + 1, Vec::new);
        self.popup_bounds
            .resize(self.path.len() + 1, Default::default());
        for depth in 0..=self.path.len() {
            let items = self.level(depth).to_vec();
            self.row_bounds[depth].resize(items.len(), Default::default());
            let entity = cx.entity();
            let popup_id = if depth == 0 {
                ElementId::from(("menu-popup", cx.entity_id()))
            } else {
                (
                    self.level(depth - 1)[self.path[depth - 1]].id.clone(),
                    "popup",
                )
                    .into()
            };
            let mut popup = div()
                .id(popup_id)
                .relative()
                .role(Role::Menu)
                .min_w(t.space(40.))
                .max_h(t.space(72.))
                .overflow_y_scroll()
                .p(t.space(1.))
                .rounded(t.radius.lg)
                .border_1()
                .border_color(t.colors.border)
                .bg(t.colors.popover)
                .text_color(t.colors.popover_foreground)
                .font_family(t.fonts.body.clone())
                .text_size(t.text(14.))
                .shadow(t.shadows.md.clone())
                .on_prepaint(move |bounds, _, cx| {
                    entity.update(cx, |this, _| this.popup_bounds[depth] = bounds)
                })
                .on_mouse_down_out(cx.listener(
                    |this, event: &gpui_kit::MouseDownEvent, window, cx| {
                        if !this.anchor.contains(&event.position)
                            && !this
                                .popup_bounds
                                .iter()
                                .any(|b| b.contains(&event.position))
                        {
                            this.close(false, window, cx);
                        }
                    },
                ));
            for (index, item) in items.into_iter().enumerate() {
                if item.separator {
                    popup = popup.child(
                        div()
                            .h(gpui_kit::px(1.))
                            .my(t.space(1.))
                            .bg(t.colors.border),
                    );
                    continue;
                }
                let entity = cx.entity();
                let selected = self.highlighted[depth] == Some(index);
                let submenu = !item.children.is_empty();
                let role = if submenu {
                    Role::MenuItem
                } else if item.link {
                    Role::Link
                } else if item.radio {
                    Role::MenuItemRadio
                } else if item.checked.is_some() {
                    Role::MenuItemCheckBox
                } else {
                    Role::MenuItem
                };
                let row = Button::new(item.id.clone())
                    .role(role)
                    .focusable(false)
                    .disabled(item.disabled)
                    .accessibility_label(item.label.clone())
                    .relative()
                    .w_full()
                    .min_h(t.space(8.))
                    .justify_start()
                    .gap(t.space(2.))
                    .px(t.space(1.5))
                    .rounded(t.radius.sm)
                    .when(selected, |b| b.bg(t.colors.accent).aria_active_descendant())
                    .when(item.disabled, |b| b.opacity(0.5))
                    .when_some(item.checked, |b, checked| {
                        b.aria_toggled(if checked {
                            gpui_kit::Toggled::True
                        } else {
                            gpui_kit::Toggled::False
                        })
                    })
                    .when(submenu, |b| {
                        b.aria_expanded(self.path.get(depth) == Some(&index))
                    })
                    .on_prepaint(move |bounds, window, cx| {
                        entity.update(cx, |this, _| {
                            if this.row_bounds[depth][index] != bounds {
                                this.row_bounds[depth][index] = bounds;
                                window.request_animation_frame();
                            }
                        })
                    })
                    .on_hover(cx.listener(move |this, hovered, _, cx| {
                        if *hovered {
                            this.highlight(depth, index, cx);
                        }
                    }))
                    .on_click(cx.listener(move |this, event, window, cx| {
                        this.activate(depth, index, event, window, cx)
                    }))
                    .child(div().flex_1().child(item.label))
                    .when(item.checked == Some(true), |b| {
                        b.child(
                            lucide(LucideIcon::Check)
                                .size(t.space(4.))
                                .text_color(t.colors.popover_foreground),
                        )
                    })
                    .when(submenu, |b| {
                        b.child(
                            lucide(LucideIcon::ChevronRight)
                                .size(t.space(4.))
                                .text_color(t.colors.popover_foreground),
                        )
                    });
                popup = popup.child(row);
            }
            let anchor = if depth == 0 {
                self.anchor
            } else {
                self.row_bounds[depth - 1][self.path[depth - 1]]
            };
            root = root.child(
                gpui_kit::deferred(
                    Positioner::side(anchor)
                        .align(Align::Start)
                        .placement(if depth > 0 || self.pointer {
                            Placement::Right
                        } else {
                            Placement::Bottom
                        })
                        .offset(t.space(1.))
                        .occlude()
                        .child(popup),
                )
                .with_priority(gpui_kit::base::POPUP_PRIORITY + depth),
            );
        }
        root
    }
}

#[derive(IntoElement)]
/// A named menu trigger or context area backed by retained menu state.
pub struct Menu {
    state: Entity<MenuState>,
    label: SharedString,
    disabled: bool,
    context: Option<AnyElement>,
    trigger: Option<AnyElement>,
    trigger_style: Option<Box<dyn FnOnce(Button) -> Button>>,
    style: StyleRefinement,
}
impl Menu {
    /// Creates a named trigger for the caller's retained menu state.
    pub fn new(state: &Entity<MenuState>, label: impl Into<SharedString>) -> Self {
        Self {
            state: state.clone(),
            label: label.into(),
            disabled: false,
            context: None,
            trigger: None,
            trigger_style: None,
            style: Default::default(),
        }
    }
    /// Supplies custom content for the named trigger button.
    pub fn trigger(mut self, child: impl IntoElement) -> Self {
        self.trigger = Some(child.into_any_element());
        self
    }
    /// Replaces trigger styling while retaining menu behavior. Include a visible keyboard focus treatment.
    pub fn trigger_style_with(mut self, style: impl FnOnce(Button) -> Button + 'static) -> Self {
        self.trigger_style = Some(Box::new(style));
        self
    }
    /// Disables interaction and applies the disabled appearance.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    /// Replaces the button trigger with content that opens a menu on secondary click or Shift+F10.
    pub fn context_area(mut self, child: impl IntoElement) -> Self {
        self.context = Some(child.into_any_element());
        self
    }
}
impl Styled for Menu {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
impl RenderOnce for Menu {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = self.state.clone();
        let keyboard = state.clone();
        let measured = state.clone();
        let context = self.context.is_some();
        let disabled = self.disabled;
        let focus = state.read(cx).trigger.clone();
        let open = state.read(cx).open;
        let mut base = div()
            .id(("menu-trigger", state.entity_id()))
            .relative()
            .on_prepaint(move |bounds, _, cx| {
                measured.update(cx, |this, _| {
                    if !this.pointer || !this.open {
                        this.anchor = bounds;
                    }
                })
            })
            .capture_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                if !disabled
                    && keyboard.read(cx).trigger.is_focused(window)
                    && ((!context && (key == "down" || key == "up"))
                        || (context && key == "f10" && event.keystroke.modifiers.shift))
                {
                    keyboard.update(cx, |this, cx| this.open(window, cx));
                    cx.stop_propagation();
                }
            });
        if let Some(area) = self.context {
            base = base
                .track_focus(&focus.tab_stop(!disabled))
                .aria_label(self.label)
                // Keep the content's focus for selection and context-menu restoration.
                .capture_any_mouse_down(|event, window, _| {
                    if matches!(event.button, MouseButton::Left | MouseButton::Right) {
                        window.prevent_default();
                    }
                })
                .on_mouse_down(MouseButton::Right, move |event, window, cx| {
                    if !disabled {
                        state.update(cx, |this, cx| {
                            this.open_at(Some(event.position), window, cx)
                        });
                        cx.stop_propagation();
                    }
                })
                .child(area);
        } else {
            base = base.child(
                Button::new(("menu-button", state.entity_id()))
                    .track_focus(&focus)
                    .tab_stop(focus.tab_stop)
                    .disabled(disabled)
                    .accessibility_label(self.label.clone())
                    .aria_expanded(open)
                    .map(|button| match self.trigger_style {
                        Some(style) => style(button),
                        None => style_button(
                            button,
                            disabled,
                            ButtonVariant::Outline,
                            ButtonSize::Default,
                            UiTheme::read(cx),
                        ),
                    })
                    .on_click(move |_, window, cx| {
                        state.update(cx, |this, cx| {
                            if this.open {
                                this.close(true, window, cx);
                            } else {
                                this.open(window, cx);
                            }
                        })
                    })
                    .child(
                        self.trigger
                            .unwrap_or_else(|| self.label.into_any_element()),
                    ),
            );
        }
        apply_style(base.child(self.state), &self.style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[gpui_kit::test]
    fn check_actions_toggle_and_radio_actions_select_one(cx: &mut gpui_kit::TestAppContext) {
        cx.update(super::super::theme::init);
        let window = cx.add_window(|_, cx| {
            MenuState::new(
                [
                    MenuItem::new("notifications", "Notifications").checked(true),
                    MenuItem::new("compact", "Compact").radio(true),
                    MenuItem::new("comfortable", "Comfortable").radio(false),
                    MenuItem::new("locked", "Locked")
                        .checked(true)
                        .disabled(true),
                ],
                cx,
            )
        });
        window
            .update(cx, |menu, window, cx| {
                menu.activate(0, 0, &ClickEvent::default(), window, cx);
                assert_eq!(menu.items[0].checked, Some(false));
                menu.activate(0, 2, &ClickEvent::default(), window, cx);
                assert_eq!(menu.items[1].checked, Some(false));
                assert_eq!(menu.items[2].checked, Some(true));
                menu.activate(0, 3, &ClickEvent::default(), window, cx);
                assert_eq!(menu.items[3].checked, Some(true));
            })
            .unwrap();
    }
}
