#![allow(missing_docs)]
//! Nova-styled native text input with IME, field registration and standard editing shortcuts.

#[path = "input_editing.rs"]
mod editing;
#[path = "input_text.rs"]
mod text;

use std::rc::Rc;

use base_gpui::primitives::input::*;
use base_gpui::{
    field::{
        FieldControlRegistration, FieldValue, current_field_context, current_field_item_disabled,
    },
    fieldset::current_fieldset_disabled,
};
use editing::Editing;
use gpui::{
    App, Div, ElementId, Entity, Global, InteractiveElement as _, IntoElement, KeyBinding,
    MouseButton, ParentElement as _, RenderOnce, Role, SharedString,
    StatefulInteractiveElement as _, Styled, Window, div, prelude::FluentBuilder as _, px,
};
use text::InputTextElement;

use super::theme::{ThemeMode, UiTheme, input_text_layout};

type ValueChange = Rc<dyn Fn(SharedString) + 'static>;
type ChangeHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App) + 'static>;
type ContextHandler = Rc<dyn Fn(&SharedString, &mut Window, &mut App) + 'static>;
type FocusHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;
type StyleHandler = Rc<dyn Fn(InputStyleState, Div) -> Div + 'static>;
/// Shared wiring for editors hosted inside numeric or roving-focus controls.
#[derive(Clone, Default)]
pub(crate) struct CompositeInput {
    pub focus: Option<gpui::FocusHandle>,
    pub tab_stop: Option<bool>,
    pub select_all_on_focus: bool,
    pub register_field: bool,
    pub role: Option<Role>,
    pub on_home: Option<ChangeHandler>,
    pub on_end: Option<ChangeHandler>,
    pub on_edge_left: Option<ChangeHandler>,
    pub on_edge_right: Option<ChangeHandler>,
    pub on_focus_change: Option<FocusHandler>,
}

gpui::actions!(
    gpuicn_input,
    [
        InputUndo,
        InputRedo,
        InputWordLeft,
        InputWordRight,
        InputSelectWordLeft,
        InputSelectWordRight,
        InputSelectHome,
        InputSelectEnd,
        InputDeleteWordLeft,
        InputDeleteWordRight,
        InputDeleteToStart,
        InputDeleteToEnd
    ]
);
struct Initialized;
impl Global for Initialized {}

#[derive(IntoElement)]
pub struct Input {
    style: gpui::StyleRefinement,
    id: ElementId,
    value: Option<SharedString>,
    default_value: Option<SharedString>,
    placeholder: Option<SharedString>,
    aria_label: Option<SharedString>,
    name: Option<SharedString>,
    on_change: Option<ContextHandler>,
    on_submit: Option<ContextHandler>,
    style_with_state: Option<StyleHandler>,
    disabled: bool,
    read_only: bool,
    required: bool,
    on_value_change: Option<ValueChange>,
    on_enter: Option<ValueChange>,
    auto_focus: bool,
    focus_handle: Option<gpui::FocusHandle>,
    composite: Option<CompositeInput>,
}

impl Input {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            style: gpui::StyleRefinement::default(),
            id: id.into(),
            value: None,
            default_value: None,
            placeholder: None,
            aria_label: None,
            name: None,
            on_change: None,
            on_submit: None,
            style_with_state: None,
            disabled: false,
            read_only: false,
            required: false,
            on_value_change: None,
            on_enter: None,
            auto_focus: false,
            focus_handle: None,
            composite: None,
        }
    }
    pub(crate) fn composite(mut self, composite: CompositeInput) -> Self {
        self.composite = Some(composite);
        self
    }
    pub(crate) fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }
    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default_value = Some(value.into());
        self
    }
    pub fn placeholder(mut self, value: impl Into<SharedString>) -> Self {
        self.placeholder = Some(value.into());
        self
    }
    pub fn aria_label(mut self, value: impl Into<SharedString>) -> Self {
        self.aria_label = Some(value.into());
        self
    }
    pub fn name(mut self, name: impl Into<SharedString>) -> Self {
        self.name = Some(name.into());
        self
    }
    /// Receive edits with GPUI context; accepts `cx.listener(...)` directly.
    pub fn on_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
    /// Submit the current value with GPUI context when Enter is pressed.
    pub fn on_submit(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_submit = Some(Rc::new(handler));
        self
    }
    /// Refine the default styles while retaining editing and focus behavior.
    pub fn style_with_state(
        mut self,
        style: impl Fn(InputStyleState, Div) -> Div + 'static,
    ) -> Self {
        self.style_with_state = Some(Rc::new(style));
        self
    }
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    pub fn read_only(mut self, value: bool) -> Self {
        self.read_only = value;
        self
    }
    pub fn required(mut self, value: bool) -> Self {
        self.required = value;
        self
    }
    pub fn on_value_change(mut self, handler: impl Fn(SharedString) + 'static) -> Self {
        self.on_value_change = Some(Rc::new(handler));
        self
    }
    /// Focus the input when it first mounts, for example an opened task composer.
    pub fn auto_focus(mut self, value: bool) -> Self {
        self.auto_focus = value;
        self
    }
    /// Use a stable handle so another control can focus this editor.
    pub fn focus_handle(mut self, focus: gpui::FocusHandle) -> Self {
        self.focus_handle = Some(focus);
        self
    }
    /// Submit the current native editing value when Enter is pressed.
    pub fn on_enter(mut self, handler: impl Fn(SharedString) + 'static) -> Self {
        self.on_enter = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Input {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        init(cx);
        let theme = UiTheme::read(cx).clone();
        let spacing = theme.spacing.unit;
        let text_scale = theme.text_scale;
        let colors = theme.colors;
        let field_context = current_field_context();
        let aria_label = self.aria_label.or_else(|| {
            field_context
                .as_ref()
                .and_then(|context| context.label_text(cx))
        });
        let field_state = field_context
            .as_ref()
            .map(|context| context.read(cx, |runtime, props| runtime.root_state(props)));
        let disabled = self.disabled
            || field_state.is_some_and(|state| state.disabled)
            || current_field_item_disabled()
            || current_fieldset_disabled();
        let valid = field_state.and_then(|state| state.valid);
        let field_context = field_context.filter(|_| {
            self.composite
                .as_ref()
                .is_none_or(|props| props.register_field)
        });
        let initial_value = self
            .value
            .clone()
            .or(self.default_value)
            .unwrap_or_default();
        let state: Entity<Editing> =
            window.use_keyed_state((self.id.clone(), "editing"), cx, |window, cx| {
                match self.focus_handle.clone().or_else(|| {
                    self.composite
                        .as_ref()
                        .and_then(|props| props.focus.clone())
                }) {
                    Some(focus) => Editing::new_with_focus_handle(initial_value, focus, window, cx),
                    None => Editing::new(initial_value, window, cx),
                }
            });
        let focus = state.read(cx).focus_handle();
        let mut registration = FieldControlRegistration::new(self.id.to_string())
            .disabled(disabled)
            .required(self.required)
            .focus_handle(focus.clone());
        if let Some(name) = self.name {
            registration = registration.name(name);
        }
        let changed_registration = registration.clone();
        let changed_field = field_context.clone();
        let changed_focus = focus.clone();
        let form_context = base_gpui::form::current_form_context();
        let change: ChangeHandler = Rc::new(move |value, window, cx| {
            // Keep form validation current even if Enter arrives before another frame.
            if let Some(context) = &changed_field {
                context.register_control(
                    changed_registration
                        .clone()
                        .value(FieldValue::Text(value.clone()))
                        .focused(changed_focus.is_focused(window)),
                    cx,
                );
            }
            if let Some(handler) = &self.on_value_change {
                handler(value.clone());
            }
            if let Some(handler) = &self.on_change {
                handler(&value, window, cx);
            }
        });
        let submit: ChangeHandler = Rc::new(move |value, window, cx| {
            if self.on_enter.is_none()
                && self.on_submit.is_none()
                && let Some(context) = &form_context
            {
                context.submit(base_gpui::form::FormSubmitReason::Action, window, cx);
                return;
            }
            if let Some(handler) = &self.on_enter {
                handler(value.clone());
            }
            if let Some(handler) = &self.on_submit {
                handler(&value, window, cx);
            }
        });
        state.update(cx, |editing, cx| {
            editing.composite = self.composite.clone().unwrap_or_default();
            editing.sync_props(
                self.value,
                disabled,
                self.read_only,
                self.required,
                Some(change),
                Some(submit),
                cx,
            )
        });
        let auto_focused: Entity<bool> =
            window.use_keyed_state((self.id.clone(), "auto-focus"), cx, |_, _| false);
        if self.auto_focus && !disabled && !*auto_focused.read(cx) {
            focus.focus(window, cx);
            *auto_focused.as_mut(cx) = true;
        }
        let style_state = state.read(cx).style_state(window, valid);
        if let Some(context) = field_context {
            context.register_control(
                registration
                    .value(FieldValue::Text(style_state.value.clone()))
                    .focused(style_state.focused),
                cx,
            );
        }
        let focus_ring = if style_state.invalid {
            theme.destructive_focus_ring()
        } else {
            theme.focus_ring()
        };
        let border = if style_state.invalid {
            colors
                .destructive
                .opacity(if theme.mode == ThemeMode::Dark {
                    0.5
                } else {
                    1.0
                })
        } else {
            colors.input
        };
        let background = match theme.mode {
            ThemeMode::Light => colors.background,
            ThemeMode::Dark => colors.background.blend(colors.input.opacity(0.30)),
        };
        let base = input_text_layout(div(), text_scale)
            .w_full()
            .min_w_0()
            .h(spacing * 8_f32)
            .px(spacing * 2.5_f32)
            .rounded(theme.radius.lg)
            .border_1()
            .border_color(border)
            .bg(if disabled {
                colors.input.opacity(if theme.mode == ThemeMode::Dark {
                    0.8
                } else {
                    0.5
                })
            } else {
                background
            })
            .font_family(theme.fonts.body)
            .text_color(colors.foreground)
            .text_size(px(14.) * text_scale)
            .focus_visible(move |style| {
                style
                    .border_color(if style_state.invalid {
                        border
                    } else {
                        colors.ring
                    })
                    .shadow(focus_ring.clone())
            })
            .when(disabled, |base| base.cursor_not_allowed());
        let value = style_state.value.clone();
        let base = if self.composite.is_some() {
            input_text_layout(div(), text_scale)
        } else {
            base
        };
        let base = if let Some(style) = self.style_with_state {
            style(style_state, base)
        } else {
            base
        };
        let placeholder = self.placeholder.unwrap_or_default();
        super::theme::apply_style(base, &self.style)
            .id(self.id.clone())
            .accessibility_id(self.id.to_string())
            .role(
                self.composite
                    .as_ref()
                    .and_then(|props| props.role)
                    .unwrap_or(Role::TextInput),
            )
            .aria_value(value)
            .aria_placeholder(placeholder.clone())
            .when_some(aria_label, |base, label| base.aria_label(label))
            .track_focus(
                &focus
                    .tab_stop(
                        self.composite
                            .as_ref()
                            .and_then(|props| props.tab_stop)
                            .unwrap_or(!disabled),
                    )
                    .tab_index(if disabled { -1 } else { 0 }),
            )
            .key_context("Input gpuicn-input")
            .focusable()
            .on_action(window.listener_for(&state, Editing::left))
            .on_action(window.listener_for(&state, Editing::right))
            .on_action(window.listener_for(&state, Editing::select_left))
            .on_action(window.listener_for(&state, Editing::select_right))
            .on_action(window.listener_for(&state, Editing::select_all))
            .on_action(window.listener_for(&state, Editing::home))
            .on_action(window.listener_for(&state, Editing::end))
            .on_action(window.listener_for(&state, Editing::word_left))
            .on_action(window.listener_for(&state, Editing::word_right))
            .on_action(window.listener_for(&state, Editing::select_word_left))
            .on_action(window.listener_for(&state, Editing::select_word_right))
            .on_action(window.listener_for(&state, Editing::select_home))
            .on_action(window.listener_for(&state, Editing::select_end))
            .on_action(window.listener_for(&state, Editing::copy))
            .on_action(window.listener_for(&state, Editing::enter))
            .on_action(window.listener_for(&state, Editing::undo))
            .on_action(window.listener_for(&state, Editing::redo))
            .on_action(window.listener_for(&state, Editing::backspace))
            .on_action(window.listener_for(&state, Editing::delete))
            .on_action(window.listener_for(&state, Editing::delete_word_left))
            .on_action(window.listener_for(&state, Editing::delete_word_right))
            .on_action(window.listener_for(&state, Editing::delete_to_start))
            .on_action(window.listener_for(&state, Editing::delete_to_end))
            .on_action(window.listener_for(&state, Editing::paste))
            .on_action(window.listener_for(&state, Editing::cut))
            .when(!disabled, |base| {
                base.on_mouse_down(
                    MouseButton::Left,
                    window.listener_for(&state, Editing::on_mouse_down),
                )
                .on_mouse_up(
                    MouseButton::Left,
                    window.listener_for(&state, Editing::on_mouse_up),
                )
                .on_mouse_up_out(
                    MouseButton::Left,
                    window.listener_for(&state, Editing::on_mouse_up),
                )
                .on_mouse_move(window.listener_for(&state, Editing::on_mouse_move))
            })
            .child(InputTextElement::new(state, placeholder))
    }
}

fn init(cx: &mut App) {
    if cx.has_global::<Initialized>() {
        return;
    }
    cx.set_global(Initialized);
    let context = Some("gpuicn-input");
    let command = if cfg!(any(target_os = "macos", target_family = "wasm")) {
        "cmd"
    } else {
        "ctrl"
    };
    let word = if cfg!(any(target_os = "macos", target_family = "wasm")) {
        "alt"
    } else {
        "ctrl"
    };
    #[cfg(target_family = "wasm")]
    cx.bind_keys([
        KeyBinding::new("ctrl-z", InputUndo, context),
        KeyBinding::new("ctrl-shift-z", InputRedo, context),
        KeyBinding::new("ctrl-left", InputWordLeft, context),
        KeyBinding::new("ctrl-right", InputWordRight, context),
        KeyBinding::new("ctrl-shift-left", InputSelectWordLeft, context),
        KeyBinding::new("ctrl-shift-right", InputSelectWordRight, context),
        KeyBinding::new("ctrl-backspace", InputDeleteWordLeft, context),
        KeyBinding::new("ctrl-delete", InputDeleteWordRight, context),
    ]);
    cx.bind_keys([
        KeyBinding::new(&format!("{command}-z"), InputUndo, context),
        KeyBinding::new(&format!("{command}-shift-z"), InputRedo, context),
        KeyBinding::new(&format!("{word}-left"), InputWordLeft, context),
        KeyBinding::new(&format!("{word}-right"), InputWordRight, context),
        KeyBinding::new(&format!("{word}-shift-left"), InputSelectWordLeft, context),
        KeyBinding::new(
            &format!("{word}-shift-right"),
            InputSelectWordRight,
            context,
        ),
        KeyBinding::new(&format!("{word}-backspace"), InputDeleteWordLeft, context),
        KeyBinding::new(&format!("{word}-delete"), InputDeleteWordRight, context),
        KeyBinding::new("shift-home", InputSelectHome, context),
        KeyBinding::new("shift-end", InputSelectEnd, context),
        #[cfg(any(target_os = "macos", target_family = "wasm"))]
        KeyBinding::new("cmd-shift-left", InputSelectHome, context),
        #[cfg(any(target_os = "macos", target_family = "wasm"))]
        KeyBinding::new("cmd-shift-right", InputSelectEnd, context),
        #[cfg(any(target_os = "macos", target_family = "wasm"))]
        KeyBinding::new("cmd-backspace", InputDeleteToStart, context),
        #[cfg(any(target_os = "macos", target_family = "wasm"))]
        KeyBinding::new("cmd-delete", InputDeleteToEnd, context),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-y", InputRedo, context),
    ]);
}

impl From<Input> for base_gpui::field::FieldChild {
    fn from(input: Input) -> Self {
        Self::Any(input.into_any_element())
    }
}
impl From<Input> for base_gpui::field::FieldItemChild {
    fn from(input: Input) -> Self {
        Self::Any(input.into_any_element())
    }
}

impl gpui::Styled for Input {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{AppContext as _, Context, Render, TestAppContext, VisualTestContext};

    struct View {
        value: String,
        disabled: bool,
        submitted: usize,
    }
    impl Render for View {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div().w(px(300.)).child(
                Input::new("editor")
                    .aria_label("Task title")
                    .auto_focus(true)
                    .value(self.value.clone())
                    .disabled(self.disabled)
                    .on_change(cx.listener(|this, value: &SharedString, _, cx| {
                        this.value = value.to_string();
                        cx.notify();
                    }))
                    .on_submit(cx.listener(|this, _, _, _| this.submitted += 1)),
            )
        }
    }

    #[gpui::test]
    fn native_editing_shortcuts_keep_controlled_value_and_selection(cx: &mut TestAppContext) {
        cx.update(super::super::theme::init);
        let window = cx.add_window(|_, _| View {
            value: "hello café".into(),
            disabled: false,
            submitted: 0,
        });
        let mut visual = VisualTestContext::from_window(window.into(), cx);
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_keystrokes("alt-shift-left");
        visual.simulate_input("世界");
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(
            cx.read_window(&window, |view, cx| view.read(cx).value.clone())
                .unwrap(),
            "hello 世界"
        );
        for (key, expected) in [("cmd-z", "hello café"), ("cmd-shift-z", "hello 世界")] {
            visual.simulate_keystrokes(key);
            visual.update(|window, cx| window.draw(cx).clear(cx));
            assert_eq!(
                cx.read_window(&window, |view, cx| view.read(cx).value.clone())
                    .unwrap(),
                expected
            );
        }
        visual.simulate_keystrokes("cmd-shift-left");
        visual.simulate_input("🦀");
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(
            cx.read_window(&window, |view, cx| view.read(cx).value.clone())
                .unwrap(),
            "🦀"
        );
        visual.simulate_keystrokes("backspace");
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(
            cx.read_window(&window, |view, cx| view.read(cx).value.clone())
                .unwrap(),
            ""
        );
        visual.simulate_keystrokes("cmd-z");
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(
            cx.read_window(&window, |view, cx| view.read(cx).value.clone())
                .unwrap(),
            "🦀"
        );
        visual.simulate_keystrokes("enter");
        window
            .update(cx, |this, _, cx| {
                this.disabled = true;
                cx.notify();
            })
            .unwrap();
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_input("ignored");
        visual.simulate_keystrokes("cmd-z enter");
        assert_eq!(
            cx.read_window(&window, |view, cx| (
                view.read(cx).value.clone(),
                view.read(cx).submitted
            ))
            .unwrap(),
            ("🦀".into(), 1)
        );
    }
    #[gpui::test]
    fn field_label_names_input_unless_explicitly_overridden(cx: &mut TestAppContext) {
        use base_gpui::field::{FieldContext, FieldLabel, FieldProps, context::with_field_context};
        use gpui::Element as _;
        struct NamedField;
        impl Render for NamedField {
            fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                let field =
                    FieldContext::new("named-field", cx, window, FieldProps::default(), None);
                with_field_context(field, || {
                    let _ = FieldLabel::new().text("Email address").render(window, cx);
                    for explicit in [None, Some("Work email")] {
                        let input = Input::new("named-input")
                            .when_some(explicit, |input, name| input.aria_label(name))
                            .render(window, cx)
                            .into_element();
                        assert_eq!(input.a11y_role(), Some(Role::TextInput));
                        let mut node = gpui::accesskit::Node::new(Role::TextInput);
                        input.write_a11y_info(&mut node);
                        assert_eq!(node.label(), Some(explicit.unwrap_or("Email address")));
                    }
                });
                div()
            }
        }
        cx.update(super::super::theme::init);
        let window = cx.add_window(|_, _| NamedField);
        VisualTestContext::from_window(window.into(), cx)
            .update(|window, cx| window.draw(cx).clear(cx));
    }

    struct FormView {
        values: Vec<base_gpui::form::FormValues>,
    }
    impl Render for FormView {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let view = cx.entity().downgrade();
            base_gpui::form::Form::new()
                .id("form")
                .on_form_submit(move |values, _, _, cx| {
                    let _ = view.update(cx, |this, _| this.values.push(values));
                })
                .child_any(
                    base_gpui::field::FieldRoot::new()
                        .id("email-field")
                        .name("email")
                        .child(Input::new("email").required(true).auto_focus(true)),
                )
        }
    }

    #[gpui::test]
    fn enter_submits_current_form_value_and_rejects_empty_required_input(cx: &mut TestAppContext) {
        cx.update(super::super::theme::init);
        let window = cx.add_window(|_, _| FormView { values: Vec::new() });
        let mut visual = VisualTestContext::from_window(window.into(), cx);
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_keystrokes("enter");
        assert!(
            cx.read_window(&window, |view, cx| view.read(cx).values.is_empty())
                .unwrap()
        );
        visual.simulate_input("ada@example.com");
        // No draw between typing and Enter: registration cannot depend on a rerender.
        visual.simulate_keystrokes("enter");
        assert_eq!(
            cx.read_window(&window, |view, cx| view.read(cx).values.clone())
                .unwrap(),
            vec![std::collections::BTreeMap::from([(
                "email".into(),
                base_gpui::form::FormValue::Text("ada@example.com".into())
            )])]
        );
    }
}
