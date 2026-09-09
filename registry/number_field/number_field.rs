#![allow(missing_docs)]
//! Nova-styled Number Field backed by Base GPUI parsing, steppers, and keyboard behavior.

use std::{rc::Rc, sync::Arc};

use super::input::{CompositeInput, Input};
use base_gpui::field::{
    FieldControlRegistration, FieldValue, current_field_context, current_field_item_disabled,
};
use base_gpui::fieldset::current_fieldset_disabled;
use base_gpui::number_field::*;
use gpui::{
    App, ElementId, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce, Role,
    SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use gpui_icons::{LucideIcon, lucide};

use super::theme::{InputSemantics, ThemeMode, UiTheme, focus_outline, input_text_layout};

type ChangeHandler = Rc<dyn Fn(Option<f64>, NumberFieldChangeDetails, &mut Window, &mut App)>;
type CommitHandler = Rc<dyn Fn(Option<f64>, NumberFieldCommitDetails, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct NumberField {
    style: gpui::StyleRefinement,
    id: ElementId,
    default_value: Option<f64>,
    value: Option<Option<f64>>,
    min: Option<f64>,
    max: Option<f64>,
    step: f64,
    placeholder: Option<SharedString>,
    aria_label: Option<SharedString>,
    disabled: bool,
    read_only: bool,
    on_value_change: Option<ChangeHandler>,
    on_value_committed: Option<CommitHandler>,
}
impl NumberField {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            style: gpui::StyleRefinement::default(),
            id: id.into(),
            default_value: None,
            value: None,
            min: None,
            max: None,
            step: 1.,
            placeholder: None,
            aria_label: None,
            disabled: false,
            read_only: false,
            on_value_change: None,
            on_value_committed: None,
        }
    }
    pub fn default_value(mut self, value: f64) -> Self {
        self.default_value = Some(value);
        self
    }
    pub fn value(mut self, value: Option<f64>) -> Self {
        self.value = Some(value);
        self
    }
    pub fn range(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    pub fn step(mut self, value: f64) -> Self {
        self.step = value;
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
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    pub fn read_only(mut self, value: bool) -> Self {
        self.read_only = value;
        self
    }
    pub fn on_value_change(
        mut self,
        handler: impl Fn(Option<f64>, NumberFieldChangeDetails, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(handler));
        self
    }
    pub fn on_value_committed(
        mut self,
        handler: impl Fn(Option<f64>, NumberFieldCommitDetails, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_committed = Some(Rc::new(handler));
        self
    }
}
impl RenderOnce for NumberField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let field = current_field_context();
        let field_state = field
            .as_ref()
            .map(|field| field.read(cx, |runtime, props| runtime.root_state(props)));
        let disabled = self.disabled
            || field_state.is_some_and(|state| state.disabled)
            || current_field_item_disabled()
            || current_fieldset_disabled();
        let focus = window
            .use_keyed_state((self.id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let registration = FieldControlRegistration::new(self.id.to_string())
            .disabled(disabled)
            .focus_handle(focus.clone());
        let change_field = field.clone();
        let change_registration = registration.clone();
        let change_focus = focus.clone();
        let handler = self.on_value_change.clone();
        let change: NumberFieldValueChangeHandler = Rc::new(move |value, details, window, cx| {
            if let Some(field) = &change_field {
                field.register_control(
                    change_registration
                        .clone()
                        .value(number_field_value(value))
                        .focused(change_focus.is_focused(window)),
                    cx,
                );
            }
            if let Some(handler) = &handler {
                handler(value, details, window, cx);
            }
        });
        let context = NumberFieldContext::new(
            self.id.clone(),
            cx,
            window,
            self.value,
            self.default_value,
            NumberFieldProps::new(
                None,
                None,
                self.min,
                self.max,
                NumberFieldStep::amount(self.step),
                0.1,
                10.,
                false,
                false,
                false,
                disabled,
                self.read_only,
                false,
                Some(change),
                self.on_value_committed.clone(),
            ),
            focus.clone(),
        );
        context.sync_focus(focus.is_focused(window), window, cx);
        let mut root_state = context.read(cx, |runtime, props| runtime.root_state(props));
        if let Some(valid) = field_state.and_then(|state| state.valid) {
            root_state.valid = Some(valid);
            root_state.invalid = !valid;
        }
        if let Some(field) = &field {
            field.register_control(
                registration
                    .value(number_field_value(root_state.value))
                    .focused(root_state.focused),
                cx,
            );
        }
        let theme = UiTheme::read(cx).clone();
        let spacing = theme.spacing.unit;
        let text_scale = theme.text_scale;
        let colors = theme.colors;
        let background = if theme.mode == ThemeMode::Dark {
            colors.background.blend(colors.input.opacity(0.30))
        } else {
            colors.background
        };
        let label = self.aria_label.or_else(|| {
            base_gpui::field::current_field_context().and_then(|field| field.label_text(cx))
        });
        let changed = context.clone();
        let focused = context.clone();
        let home = context.clone();
        let end = context.clone();
        let input = Input::new(ElementId::NamedChild(
            Arc::new(self.id.clone()),
            "input".into(),
        ))
        .value(root_state.input_value.clone())
        .disabled(disabled)
        .read_only(self.read_only)
        .when_some(label, |input, label| input.aria_label(label))
        .composite(CompositeInput {
            focus: Some(focus),
            role: Some(Role::SpinButton),
            on_focus_change: Some(Rc::new(move |focused_now, window, cx| {
                focused.sync_focus(focused_now, window, cx)
            })),
            on_home: Some(Rc::new(move |_, window, cx| {
                home.move_to_boundary(NumberFieldStepDirection::Down, window, cx)
            })),
            on_end: Some(Rc::new(move |_, window, cx| {
                end.move_to_boundary(NumberFieldStepDirection::Up, window, cx)
            })),
            ..Default::default()
        })
        .on_change(move |value, window, cx| changed.input_changed(value.clone(), window, cx))
        .style_with_state(move |_, base| {
            let base = InputSemantics(base)
                .aria_numeric_value_step(self.step)
                .when_some(root_state.value, |base, value| {
                    base.aria_numeric_value(value)
                })
                .when_some(self.min, |base, value| base.aria_min_numeric_value(value))
                .when_some(self.max, |base, value| base.aria_max_numeric_value(value))
                .0;
            input_text_layout(base, text_scale)
                .flex_1()
                .min_w_0()
                .h(spacing * 8. - px(2.))
                .px(spacing * 2.5_f32)
                .text_color(colors.foreground)
                .text_size(px(14.) * text_scale)
                .when(disabled, |base| base.opacity(0.50))
        });
        let input = if let Some(placeholder) = self.placeholder {
            input.placeholder(placeholder)
        } else {
            input
        };
        let root = {
            let base = {
                let ring = if root_state.invalid {
                    theme.destructive_focus_ring()[0].color
                } else {
                    theme.focus_ring()[0].color
                };
                div()
                    .w_full()
                    .h(spacing * 8_f32)
                    .rounded(theme.radius.lg)
                    .border_1()
                    .border_color(if root_state.invalid {
                        colors.destructive
                    } else {
                        colors.input
                    })
                    .bg(background)
                    .when(root_state.focused, |base| {
                        focus_outline(
                            base.border_color(if root_state.invalid {
                                colors.destructive
                            } else {
                                colors.ring
                            }),
                            ring.into(),
                            gpui::Corners::all(theme.radius.lg),
                        )
                    })
                    .when(root_state.disabled, |base| base.cursor_not_allowed())
            };
            super::theme::apply_style(base, &self.style)
        };
        let root = root.child(
            NumberFieldGroup::new()
                .with_number_field_context(context.clone())
                .w_full()
                .h_full()
                .flex()
                .items_center()
                .child_any(input)
                .child(
                    NumberFieldDecrement::new()
                        .with_number_field_context(context.clone())
                        .style_with_state(move |state, base| {
                            base.when(state.can_decrement, |base| {
                                base.cursor_pointer()
                                    .hover(move |style| style.bg(colors.muted))
                            })
                            .when(!state.can_decrement, |base| {
                                base.opacity(0.50).cursor_not_allowed()
                            })
                        })
                        .flex()
                        .size(spacing * 8. - px(2.))
                        .items_center()
                        .justify_center()
                        .border_l_1()
                        .border_color(colors.input)
                        .text_color(colors.muted_foreground)
                        .child(
                            lucide(LucideIcon::Minus)
                                .size(spacing * 3.5_f32)
                                .text_color(colors.muted_foreground),
                        ),
                )
                .child(
                    NumberFieldIncrement::new()
                        .with_number_field_context(context.clone())
                        .style_with_state(move |state, base| {
                            base.when(state.can_increment, |base| {
                                base.cursor_pointer()
                                    .hover(move |style| style.bg(colors.muted))
                            })
                            .when(!state.can_increment, |base| {
                                base.opacity(0.50).cursor_not_allowed()
                            })
                        })
                        .flex()
                        .size(spacing * 8. - px(2.))
                        .items_center()
                        .justify_center()
                        .border_l_1()
                        .border_color(colors.input)
                        .text_color(colors.muted_foreground)
                        .child(
                            lucide(LucideIcon::Plus)
                                .size(spacing * 3.5_f32)
                                .text_color(colors.muted_foreground),
                        ),
                ),
        );
        number_field_actions(root.id(self.id), context)
    }
}

impl gpui::Styled for NumberField {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

fn number_field_value(value: Option<f64>) -> FieldValue {
    value.map_or(FieldValue::Empty, |value| {
        FieldValue::Text(format_number(Some(value)))
    })
}

fn number_field_actions(
    mut base: gpui::Stateful<gpui::Div>,
    context: NumberFieldContext,
) -> gpui::Stateful<gpui::Div> {
    macro_rules! step {
        ($action:ty, $direction:ident, $amount:ident) => {{
            let context = context.clone();
            base = base.on_action(move |_: &$action, window, cx| {
                context.step(
                    NumberFieldStepDirection::$direction,
                    NumberFieldStepAmount::$amount,
                    NumberFieldChangeReason::Keyboard,
                    NumberFieldCommitReason::Keyboard,
                    window,
                    cx,
                );
            });
        }};
    }
    step!(NumberFieldStepUp, Up, Normal);
    step!(NumberFieldStepDown, Down, Normal);
    step!(NumberFieldStepUpSmall, Up, Small);
    step!(NumberFieldStepDownSmall, Down, Small);
    step!(NumberFieldStepUpLarge, Up, Large);
    step!(NumberFieldStepDownLarge, Down, Large);
    let min = context.clone();
    base.key_context(NUMBER_FIELD_KEY_CONTEXT)
        .on_action(move |_: &NumberFieldMin, window, cx| {
            min.move_to_boundary(NumberFieldStepDirection::Down, window, cx)
        })
        .on_action(move |_: &NumberFieldMax, window, cx| {
            context.move_to_boundary(NumberFieldStepDirection::Up, window, cx)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Context, Render, TestAppContext, VisualTestContext};
    use std::cell::RefCell;

    struct View {
        value: Rc<RefCell<Option<f64>>>,
        commits: Rc<RefCell<Vec<Option<f64>>>>,
        disabled: bool,
        read_only: bool,
    }
    impl Render for View {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let value = self.value.clone();
            let commits = self.commits.clone();
            div()
                .w(px(300.))
                .child(
                    NumberField::new("quantity")
                        .default_value(12.)
                        .range(Some(-20.), Some(200.))
                        .disabled(self.disabled)
                        .read_only(self.read_only)
                        .on_value_change(move |next, _, _, _| *value.borrow_mut() = next)
                        .on_value_committed(move |next, _, _, _| commits.borrow_mut().push(next)),
                )
                .child(super::super::input::Input::new("after"))
        }
    }

    #[gpui::test]
    fn editing_history_preserves_numeric_steps_bounds_and_inert_states(cx: &mut TestAppContext) {
        cx.update(super::super::theme::init);
        let value = Rc::new(RefCell::new(Some(12.)));
        let commits = Rc::new(RefCell::new(Vec::new()));
        let window = cx.add_window({
            let value = value.clone();
            let commits = commits.clone();
            move |_, _| View {
                value,
                commits,
                disabled: false,
                read_only: false,
            }
        });
        let mut visual = VisualTestContext::from_window(window.into(), cx);
        let draw =
            |visual: &mut VisualTestContext| visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, _| window.activate_window());
        draw(&mut visual);
        visual.simulate_keystrokes("tab cmd-a");
        visual.simulate_input("34");
        draw(&mut visual);
        assert_eq!(*value.borrow(), Some(34.));
        for (keys, expected) in [
            ("cmd-z", Some(12.)),
            ("cmd-shift-z", Some(34.)),
            ("right alt-backspace", None),
            ("cmd-z", Some(34.)),
            ("up", Some(35.)),
            ("shift-up", Some(45.)),
            ("home", Some(-20.)),
            ("end", Some(200.)),
            ("up", Some(200.)),
        ] {
            visual.simulate_keystrokes(keys);
            draw(&mut visual);
            assert_eq!(*value.borrow(), expected, "{keys}");
        }
        visual.simulate_keystrokes("cmd-a");
        visual.simulate_input("-");
        draw(&mut visual);
        // An incomplete number stays editable, then undo restores the last parsed value.
        visual.simulate_keystrokes("cmd-z");
        draw(&mut visual);
        assert_eq!(*value.borrow(), Some(200.));
        visual.simulate_keystrokes("cmd-a");
        visual.simulate_input("57");
        draw(&mut visual);
        visual.simulate_keystrokes("tab");
        draw(&mut visual);
        assert_eq!(commits.borrow().last(), Some(&Some(57.)));
        visual.simulate_keystrokes("shift-tab");
        draw(&mut visual);
        window
            .update(cx, |view, _, cx| {
                view.read_only = true;
                cx.notify();
            })
            .unwrap();
        draw(&mut visual);
        visual.simulate_keystrokes("cmd-z up alt-backspace cmd-a");
        visual.simulate_input("99");
        draw(&mut visual);
        assert_eq!(*value.borrow(), Some(57.));
        window
            .update(cx, |view, _, cx| {
                view.disabled = true;
                view.read_only = false;
                cx.notify();
            })
            .unwrap();
        draw(&mut visual);
        visual.simulate_keystrokes("cmd-z up alt-backspace");
        visual.simulate_input("99");
        draw(&mut visual);
        assert_eq!(*value.borrow(), Some(57.));
    }
}
