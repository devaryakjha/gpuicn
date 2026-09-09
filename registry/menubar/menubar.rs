//! Nova menu bar with one tab stop and keyboard switching between menus.
use super::{
    menu::{Menu, MenuState},
    theme::{UiTheme, apply_style},
};
use gpui_kit::{
    App, ElementId, Entity, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    Role, SharedString, StatefulInteractiveElement as _, StyleRefinement, Styled, Window, div,
};
#[derive(IntoElement)]
/// A horizontal menu group with one active Tab stop.
pub struct Menubar {
    id: ElementId,
    label: SharedString,
    items: Vec<(Entity<MenuState>, SharedString)>,
    style: StyleRefinement,
    pub(crate) role: Role,
}
impl Menubar {
    /// Creates a `Menubar` with a stable caller-owned ID.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            items: Vec::new(),
            style: Default::default(),
            role: Role::MenuBar,
        }
    }
    /// Appends a named trigger backed by its own retained menu state.
    pub fn menu(mut self, state: &Entity<MenuState>, label: impl Into<SharedString>) -> Self {
        self.items.push((state.clone(), label.into()));
        self
    }
}
impl Styled for Menubar {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
impl RenderOnce for Menubar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = UiTheme::read(cx).clone();
        let states: Vec<_> = self.items.iter().map(|(state, _)| state.clone()).collect();
        let active = window.use_keyed_state((self.id.clone(), "active"), cx, |_, _| 0_usize);
        let current = states
            .iter()
            .position(|s| s.read(cx).is_open() || s.read(cx).trigger_focus().is_focused(window))
            .unwrap_or(*active.read(cx))
            .min(states.len().saturating_sub(1));
        active.update(cx, |value, _| *value = current);
        for (i, state) in states.iter().enumerate() {
            state.read(cx).trigger_focus().tab_stop(i == current);
        }
        let mut root = div()
            .id(self.id)
            .role(self.role)
            .aria_label(self.label)
            .flex()
            .items_center()
            .gap(t.space(0.5))
            .p(t.space(0.75))
            .border_1()
            .border_color(t.colors.border)
            .rounded(t.radius.lg)
            .bg(t.colors.background);
        for (index, (state, label)) in self.items.into_iter().enumerate() {
            let siblings = states.clone();
            root = root.child(
                div()
                    .id(("menubar-entry", state.entity_id()))
                    .on_hover(move |hovered, window, cx| {
                        if !*hovered {
                            return;
                        }
                        let open = siblings.iter().position(|s| s.read(cx).is_open());
                        if let Some(open) = open.filter(|&i| i != index) {
                            siblings[open].update(cx, |s, cx| s.close(false, window, cx));
                            siblings[index].update(cx, |s, cx| s.open(window, cx));
                        }
                    })
                    .child(Menu::new(&state, label)),
            );
        }
        apply_style(
            root.capture_key_down(move |event, window, cx| {
                if event.keystroke.modifiers.modified() || states.is_empty() {
                    return;
                }
                let key = event.keystroke.key.as_str();
                let Some(current) = states.iter().position(|s| {
                    s.read(cx).is_open() || s.read(cx).trigger_focus().is_focused(window)
                }) else {
                    return;
                };
                let open = states[current].read(cx).is_open();
                if open
                    && ((key == "left" && states[current].read(cx).submenu_key(false))
                        || (key == "right" && states[current].read(cx).submenu_key(true)))
                {
                    return;
                }
                let next = match key {
                    "left" => (current + states.len() - 1) % states.len(),
                    "right" => (current + 1) % states.len(),
                    "home" if !open => 0,
                    "end" if !open => states.len() - 1,
                    _ => return,
                };
                states[current].read(cx).trigger_focus().tab_stop(false);
                let target = states[next].read(cx).trigger_focus().tab_stop(true);
                states[current].update(cx, |s, cx| s.close(false, window, cx));
                if open {
                    states[next].update(cx, |s, cx| s.open(window, cx));
                } else {
                    target.focus(window, cx);
                }
                active.update(cx, |value, _| *value = next);
                window.refresh();
                cx.stop_propagation();
            }),
            &self.style,
        )
    }
}
