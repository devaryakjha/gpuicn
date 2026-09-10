//! Nova selection controls with Kit editor state, semantics and popup positioning.
use super::{
    input::{Input, InputEvent, InputState},
    theme::{UiTheme, apply_style},
};
use gpui_icons::{LucideIcon, lucide};
use gpui_kit::base::{
    Button, ElementExt as _, Placement, Positioner, Select as BaseSelect,
    actions::{SelectDown, SelectUp},
};
use gpui_kit::{
    App, AppContext as _, Context, ElementId, Entity, EventEmitter, FocusHandle, Focusable,
    InteractiveElement as _, IntoElement, ParentElement as _, Render, RenderOnce, Role,
    ScrollHandle, SharedString, StatefulInteractiveElement as _, StyleRefinement, Styled,
    Subscription, Window, div, prelude::FluentBuilder as _,
};

#[derive(Clone)]
/// One selectable value and its visible label.
pub struct SelectItem {
    /// The stable value emitted when selected.
    pub value: SharedString,
    /// The visible option label.
    pub label: SharedString,
    /// Whether this option is excluded from selection.
    pub disabled: bool,
}
impl SelectItem {
    /// Creates an enabled option from its stable value and visible label.
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
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
#[derive(Clone, Debug)]
/// Notifications emitted when a selector commits a value.
pub enum SelectEvent {
    /// The newly committed value, or `None` after clearing the selection.
    Change(Option<SharedString>),
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    Select,
    Combobox,
    Autocomplete,
}
/// Own the state and subscribe to `SelectEvent::Change` for committed values.
pub struct SelectState {
    items: Vec<SelectItem>,
    syncing_value: Option<SharedString>,
    value: Option<SharedString>,
    input: Entity<InputState>,
    focus: FocusHandle,
    bounds: gpui_kit::Bounds<gpui_kit::Pixels>,
    open: bool,
    highlighted: Option<usize>,
    scroll: ScrollHandle,
    mode: Mode,
    disabled: bool,
    label: SharedString,
    placeholder: SharedString,
    style: StyleRefinement,
    _subscription: Subscription,
}
impl SelectState {
    /// Creates selector state and its Kit editor in the current window.
    pub fn new(
        items: impl IntoIterator<Item = SelectItem>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let input = cx.new(|cx| InputState::new(window, cx));
        let subscription = cx.subscribe_in(&input, window, |this, _, event, _, cx| match event {
            InputEvent::Change if !this.disabled => {
                if this.syncing_value.take().as_ref() == Some(&this.input.read(cx).value()) {
                    return;
                }
                this.open = true;
                this.highlighted = this
                    .visible(cx)
                    .into_iter()
                    .find(|&i| !this.items[i].disabled);
                cx.notify();
            }
            InputEvent::Blur => {
                this.open = false;
                cx.notify();
            }
            _ => {}
        });
        Self {
            items: items.into_iter().collect(),
            syncing_value: None,
            value: None,
            input,
            focus: cx.focus_handle(),
            bounds: Default::default(),
            open: false,
            highlighted: None,
            scroll: ScrollHandle::new(),
            mode: Mode::Select,
            disabled: false,
            label: "Selection".into(),
            placeholder: "Select…".into(),
            style: Default::default(),
            _subscription: subscription,
        }
    }
    /// Returns the currently committed value.
    pub fn value(&self) -> Option<&SharedString> {
        self.value.as_ref()
    }
    /// Returns the retained Kit text editor used by searchable presentations.
    pub fn input(&self) -> &Entity<InputState> {
        &self.input
    }
    /// Returns whether the popup is open.
    pub fn is_open(&self) -> bool {
        self.open
    }
    /// Updates the committed selection and synchronizes the editor text.
    pub fn set_value(
        &mut self,
        value: Option<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let text = value
            .as_ref()
            .and_then(|value| self.items.iter().find(|item| &item.value == value))
            .map(|item| item.label.clone())
            .or_else(|| {
                (self.mode == Mode::Autocomplete)
                    .then(|| value.clone())
                    .flatten()
            });
        self.value = text.as_ref().and(value);
        let text = text.unwrap_or_default();
        // Plain selects have no rendered editor or initialized editor font.
        if self.mode != Mode::Select {
            self.syncing_value = Some(text.clone());
            self.input
                .update(cx, |input, cx| input.set_value(text, window, cx));
        }
        cx.notify();
    }
    fn visible(&self, cx: &App) -> Vec<usize> {
        let query = if self.mode == Mode::Select {
            String::new()
        } else {
            self.input.read(cx).value().to_lowercase()
        };
        self.items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| item.label.to_lowercase().contains(&query).then_some(i))
            .collect()
    }
    fn set_open(&mut self, open: bool, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        self.open = open;
        if open {
            let visible = self.visible(cx);
            self.highlighted = visible
                .iter()
                .copied()
                .find(|&i| {
                    !self.items[i].disabled && Some(&self.items[i].value) == self.value.as_ref()
                })
                .or_else(|| visible.into_iter().find(|&i| !self.items[i].disabled));
        }
        cx.notify();
    }
    fn navigate(&mut self, direction: isize, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        if !self.open {
            self.set_open(true, cx);
            return;
        }
        let visible = self.visible(cx);
        let enabled: Vec<_> = visible
            .iter()
            .copied()
            .filter(|&i| !self.items[i].disabled)
            .collect();
        if enabled.is_empty() {
            return;
        }
        let current = enabled
            .iter()
            .position(|i| Some(*i) == self.highlighted)
            .unwrap_or(0);
        let next = match direction {
            -2 => 0,
            2 => enabled.len() - 1,
            _ => (current as isize + direction).rem_euclid(enabled.len() as isize) as usize,
        };
        self.highlighted = Some(enabled[next]);
        if let Some(row) = visible.iter().position(|i| *i == enabled[next]) {
            self.scroll.scroll_to_item(row);
        }
        cx.notify();
    }
    fn choose(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || self.items[index].disabled {
            return;
        }
        self.set_value(Some(self.items[index].value.clone()), window, cx);
        self.open = false;
        self.focus_handle(cx).focus(window, cx);
        cx.emit(SelectEvent::Change(self.value.clone()));
    }
    fn confirm(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.open {
            self.set_open(true, cx);
            return;
        }
        if let Some(index) = self.highlighted {
            self.choose(index, window, cx);
        } else if self.mode == Mode::Autocomplete {
            self.value = Some(self.input.read(cx).value());
            self.open = false;
            cx.emit(SelectEvent::Change(self.value.clone()));
            cx.notify();
        }
    }
    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_value(None, window, cx);
        self.set_open(true, cx);
        self.input.read(cx).focus_handle(cx).focus(window, cx);
        cx.emit(SelectEvent::Change(None));
    }
}
impl EventEmitter<SelectEvent> for SelectState {}

#[cfg(test)]
mod tests {
    use super::*;
    #[gpui_kit::test]
    fn choosing_an_option_without_a_rendered_editor(cx: &mut gpui_kit::TestAppContext) {
        let window = cx.add_empty_window();
        window.update(|window, cx| {
            crate::init(cx);
            let state =
                cx.new(|cx| SelectState::new([SelectItem::new("pear", "Pear")], window, cx));
            state.update(cx, |state, cx| state.choose(0, window, cx));
            assert_eq!(state.read(cx).value().map(|v| v.as_ref()), Some("pear"));
            assert!(state.read(cx).input.read(cx).value().is_empty());
        });
    }
}
impl Focusable for SelectState {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        if self.mode == Mode::Select {
            self.focus.clone()
        } else {
            self.input.read(cx).focus_handle(cx)
        }
    }
}
impl Render for SelectState {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let id = ElementId::from(("select", cx.entity_id()));
        let editing = self.mode != Mode::Select;
        let state = cx.entity();
        let open_state = state.clone();
        let measured = state.clone();
        let trigger = if editing {
            let clear = Button::new((id.clone(), "clear"))
                .focusable(false)
                .disabled(self.disabled)
                .accessibility_label("Clear selection")
                .size(theme.space(6.))
                .on_click(cx.listener(|this, _, window, cx| this.clear(window, cx)))
                .child(
                    lucide(LucideIcon::X)
                        .size(theme.space(3.5))
                        .text_color(colors.muted_foreground),
                );
            let toggle = Button::new((id.clone(), "toggle"))
                .focusable(false)
                .disabled(self.disabled)
                .accessibility_label("Show options")
                .size(theme.space(6.))
                .on_click(cx.listener(|this, _, _, cx| this.set_open(!this.open, cx)))
                .child(
                    lucide(LucideIcon::ChevronDown)
                        .size(theme.space(4.))
                        .text_color(colors.muted_foreground),
                );
            div()
                .flex()
                .items_center()
                .w_full()
                .rounded(theme.radius.lg)
                .border_1()
                .border_color(colors.input)
                .bg(colors.background)
                .pr(theme.space(1.))
                .child(
                    Input::new(&self.input)
                        .aria_label(self.label.clone())
                        .disabled(self.disabled)
                        .bordered(false),
                )
                .child(clear)
                .child(toggle)
                .into_any_element()
        } else {
            let text = self
                .value
                .as_ref()
                .and_then(|value| self.items.iter().find(|i| &i.value == value))
                .map(|i| i.label.clone())
                .unwrap_or_else(|| self.placeholder.clone());
            div()
                .id((id.clone(), "trigger"))
                .flex()
                .items_center()
                .justify_between()
                .w_full()
                .h(theme.space(8.))
                .rounded(theme.radius.lg)
                .border_1()
                .border_color(colors.input)
                .px(theme.space(2.5))
                .bg(colors.background)
                .text_color(if self.value.is_some() {
                    colors.foreground
                } else {
                    colors.muted_foreground
                })
                .when(!self.disabled, |d| {
                    d.cursor_pointer()
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.focus.focus(window, cx);
                            this.set_open(!this.open, cx);
                        }))
                })
                .child(text)
                .child(
                    lucide(LucideIcon::ChevronDown)
                        .size(theme.space(4.))
                        .text_color(colors.muted_foreground),
                )
                .into_any_element()
        };
        let mut content = div().w_full().child(trigger);
        if self.open && !self.disabled {
            let mut list = div()
                .id((id.clone(), "list"))
                .role(Role::ListBox)
                .aria_label(self.label.clone())
                .w(self.bounds.size.width.max(theme.space(48.)))
                .max_h(theme.space(60.))
                .overflow_y_scroll()
                .track_scroll(&self.scroll)
                .rounded(theme.radius.lg)
                .border_1()
                .border_color(colors.border)
                .bg(colors.popover)
                .text_color(colors.popover_foreground)
                .p(theme.space(1.))
                .shadow(theme.shadows.md.clone())
                .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                    this.open = false;
                    cx.notify();
                }));
            let visible = self.visible(cx);
            if visible.is_empty() {
                list = list.child(
                    div()
                        .p(theme.space(3.))
                        .text_color(colors.muted_foreground)
                        .child("No results found."),
                );
            }
            for i in visible {
                let item = &self.items[i];
                list = list.child(
                    Button::new((id.clone(), item.value.clone()))
                        .role(Role::ListBoxOption)
                        .focusable(false)
                        .disabled(item.disabled)
                        .accessibility_label(item.label.clone())
                        .aria_selected(Some(&item.value) == self.value.as_ref())
                        .w_full()
                        .justify_start()
                        .h(theme.space(8.))
                        .px(theme.space(1.5))
                        .rounded(theme.radius.sm)
                        .when(Some(i) == self.highlighted, |b| b.bg(colors.muted))
                        .when(item.disabled, |b| b.opacity(0.5).cursor_not_allowed())
                        .on_hover(cx.listener(move |this, hovered, _, cx| {
                            if *hovered && !this.items[i].disabled {
                                this.highlighted = Some(i);
                                cx.notify();
                            }
                        }))
                        .on_click(
                            cx.listener(move |this, _, window, cx| this.choose(i, window, cx)),
                        )
                        .child(item.label.clone()),
                );
            }
            content = content.child(
                gpui_kit::deferred(
                    Positioner::side(self.bounds)
                        .placement(Placement::Bottom)
                        .offset(theme.space(1.))
                        .occlude()
                        .child(list),
                )
                .with_priority(gpui_kit::base::POPUP_PRIORITY),
            );
        }
        let root = BaseSelect::new(id.clone())
            .key_context(if editing { "Combobox" } else { "Select" })
            .open(self.open)
            .disabled(self.disabled)
            .accessibility_label(self.label.clone())
            .accessibility_value(
                self.value
                    .as_ref()
                    .and_then(|value| self.items.iter().find(|i| &i.value == value))
                    .map(|i| i.label.clone())
                    .unwrap_or_default(),
            )
            .when(!editing, |s| s.focus_handle(&self.focus))
            .on_open_change(move |open, _, cx| {
                open_state.update(cx, |state, cx| state.set_open(open, cx))
            })
            .on_confirm(move |window, cx| {
                state.update(cx, |this, cx| this.confirm(window, cx));
                cx.stop_propagation();
            })
            .child(content);
        apply_style(
            div()
                .id((id, "keyboard"))
                .relative()
                .font_family(theme.fonts.body.clone())
                .text_size(theme.text(14.))
                .when(self.disabled, |d| d.opacity(0.5))
                .on_prepaint(move |bounds, window, cx| {
                    measured.update(cx, |this, _| {
                        if this.bounds != bounds {
                            this.bounds = bounds;
                            window.request_animation_frame();
                        }
                    })
                })
                .capture_action(cx.listener(|this, _: &SelectUp, _, cx| {
                    this.navigate(-1, cx);
                    cx.stop_propagation();
                }))
                .capture_action(cx.listener(|this, _: &SelectDown, _, cx| {
                    this.navigate(1, cx);
                    cx.stop_propagation();
                }))
                .on_key_down(
                    cx.listener(move |this, event: &gpui_kit::KeyDownEvent, _, cx| {
                        if !editing && !event.keystroke.modifiers.modified() {
                            match event.keystroke.key.as_str() {
                                "home" => this.navigate(-2, cx),
                                "end" => this.navigate(2, cx),
                                _ => return,
                            }
                            cx.stop_propagation();
                        }
                    }),
                )
                .child(root),
            &self.style,
        )
    }
}

#[derive(IntoElement)]
/// A single-choice popup backed by retained selector state.
pub struct Select {
    state: Entity<SelectState>,
    pub(crate) mode: Mode,
    label: SharedString,
    placeholder: SharedString,
    disabled: bool,
    style: StyleRefinement,
}
impl Select {
    /// Creates a single-choice trigger for retained selector state.
    pub fn new(state: &Entity<SelectState>) -> Self {
        Self {
            state: state.clone(),
            mode: Mode::Select,
            label: "Selection".into(),
            placeholder: "Select…".into(),
            disabled: false,
            style: Default::default(),
        }
    }
    /// Sets the accessible name of the control.
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }
    /// Sets the text shown when no value is selected.
    pub fn placeholder(mut self, value: impl Into<SharedString>) -> Self {
        self.placeholder = value.into();
        self
    }
    /// Disables interaction and applies the disabled appearance.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}
impl Styled for Select {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
impl RenderOnce for Select {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.state.update(cx, |state, cx| {
            state.mode = self.mode;
            state.disabled = self.disabled;
            state.label = self.label;
            state.placeholder = self.placeholder;
            state.style = self.style;
            if state.input.read(cx).presentation().placeholder() != &state.placeholder {
                state.input.update(cx, |input, cx| {
                    input.set_placeholder(state.placeholder.clone(), window, cx)
                });
            }
        });
        self.state
    }
}
