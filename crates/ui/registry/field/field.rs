//! Nova field layout with explicit input state, label focus and validation messages.

use super::{
    input::{Input, InputState, Textarea},
    theme::UiTheme,
};
use gpui_kit::{
    AnyElement, App, ClickEvent, Div, ElementId, Entity, FocusHandle, FontWeight,
    InteractiveElement as _, IntoElement, MouseButton, ParentElement as _, RenderOnce, Role,
    SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// What a click on a field's label does for controls that are not focused
/// by it, such as a checkbox toggling.
pub type FieldLabelClick = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// A control that can receive a field's shared name, disabled and invalid state.
///
/// Text controls return their focus handle so a click on the label focuses
/// them. Controls without one focus target, such as a checkbox or a radio
/// group, return `None` from `field_label_focus` and may handle the click in
/// `field_label_click` instead.
pub trait FieldControl: 'static {
    /// Returns the control's real keyboard focus target.
    fn field_focus_handle(&self, cx: &App) -> FocusHandle {
        cx.focus_handle()
    }
    /// The handle a press on the label focuses; the control's focus handle
    /// unless overridden.
    fn field_label_focus(&self, cx: &App) -> Option<FocusHandle> {
        Some(self.field_focus_handle(cx))
    }
    /// What a click on the label does besides focusing; nothing by default.
    fn field_label_click(&self) -> Option<FieldLabelClick> {
        None
    }
    /// Whether the control sits before its label on one row, as a checkbox
    /// does; the description and error then line up under the label.
    fn field_inline(&self) -> bool {
        false
    }
    /// Applies the field state and erases the rendered control type.
    fn into_field_control(self, label: SharedString, disabled: bool, invalid: bool) -> AnyElement;
}

impl FieldControl for Input {
    fn field_focus_handle(&self, cx: &App) -> FocusHandle {
        Input::field_focus_handle(self, cx)
    }

    fn into_field_control(self, label: SharedString, disabled: bool, invalid: bool) -> AnyElement {
        self.aria_label(label)
            .disabled(disabled)
            .invalid(invalid)
            .into_any_element()
    }
}

impl FieldControl for Textarea {
    fn field_focus_handle(&self, cx: &App) -> FocusHandle {
        Textarea::field_focus_handle(self, cx)
    }

    fn into_field_control(self, label: SharedString, disabled: bool, invalid: bool) -> AnyElement {
        self.aria_label(label)
            .disabled(disabled)
            .invalid(invalid)
            .into_any_element()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
/// The placement of a field label relative to its editor.
pub enum FieldOrientation {
    #[default]
    /// Place the label above the editor.
    Vertical,
    /// Place the label beside the editor.
    Horizontal,
}

#[derive(IntoElement)]
/// A labeled control with application-owned validation feedback.
pub struct Field<C: FieldControl = Input> {
    id: ElementId,
    control: C,
    label: SharedString,
    description: Option<SharedString>,
    error: Option<SharedString>,
    required: bool,
    disabled: bool,
    orientation: FieldOrientation,
    style: gpui_kit::StyleRefinement,
}
impl Field<Input> {
    /// Creates a field around the caller's retained editing state.
    pub fn new(id: impl Into<ElementId>, state: &Entity<InputState>) -> Self {
        Field::from_control(id, Input::new(state))
    }
}
impl<C: FieldControl> Field<C> {
    /// Creates a field around any supported text, choice or number control.
    pub fn from_control(id: impl Into<ElementId>, control: C) -> Self {
        Self {
            id: id.into(),
            control,
            label: SharedString::default(),
            description: None,
            error: None,
            required: false,
            disabled: false,
            orientation: FieldOrientation::Vertical,
            style: Default::default(),
        }
    }
    /// Sets the visible label and the control's accessible name.
    pub fn label(mut self, value: impl Into<SharedString>) -> Self {
        self.label = value.into();
        self
    }
    /// Sets supporting text below the editor.
    pub fn description(mut self, value: impl Into<SharedString>) -> Self {
        self.description = Some(value.into());
        self
    }
    /// Shows validation feedback and marks the editor invalid.
    pub fn error(mut self, value: impl Into<SharedString>) -> Self {
        self.error = Some(value.into());
        self
    }
    /// Marks the field as required in its visible and accessible label.
    pub fn required(mut self, value: bool) -> Self {
        self.required = value;
        self
    }
    /// Disables interaction and applies the disabled appearance.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    /// Places the label above or beside the editor.
    pub fn orientation(mut self, value: FieldOrientation) -> Self {
        self.orientation = value;
        self
    }
}
impl<C: FieldControl> Styled for Field<C> {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}
impl<C: FieldControl> RenderOnce for Field<C> {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = UiTheme::read(cx).clone();
        let focus = self.control.field_label_focus(cx);
        let click = self.control.field_label_click();
        let inline = self.control.field_inline();
        let label = if self.required {
            format!("{} (required)", self.label).into()
        } else {
            self.label.clone()
        };
        let interactive = !self.disabled && (focus.is_some() || click.is_some());
        let label_view = div()
            .id((self.id.clone(), "label"))
            .flex()
            .items_center()
            .gap(t.space(1.))
            .font_weight(FontWeight::MEDIUM)
            .text_size(t.text(14.))
            .line_height(t.text(20.))
            .child(self.label)
            .when(self.required, |label| label.child(" *"))
            .when(interactive, |label| label.cursor_pointer())
            .when(!self.disabled, |label| {
                label
                    .when_some(focus, |label, focus| {
                        label.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            focus.focus(window, cx)
                        })
                    })
                    .when_some(click, |label, click| {
                        label.on_click(move |event, window, cx| click(event, window, cx))
                    })
            });
        let description = self.error.clone().or(self.description.clone());
        let invalid = self.error.is_some();
        let description = description.map(|description| {
            div()
                .id("description")
                .when(invalid, |description| description.role(Role::Alert))
                .text_size(t.text(14.))
                .line_height(t.text(20.))
                .text_color(if invalid {
                    t.colors.destructive
                } else {
                    t.colors.muted_foreground
                })
                .child(description)
        });
        let control = self
            .control
            .into_field_control(label, self.disabled, invalid);
        let root = div()
            .id(self.id)
            .role(Role::Group)
            .w_full()
            .flex()
            .gap(t.space(2.))
            .font_family(t.fonts.body.clone())
            .text_color(t.colors.foreground);
        let root = if inline {
            // Checkbox first, then the label with its note beneath it.
            root.items_start().gap(t.space(3.)).child(control).child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(t.space(1.5))
                    .child(label_view)
                    .children(description),
            )
        } else {
            root.when(self.orientation == FieldOrientation::Vertical, |root| {
                root.flex_col()
            })
            .when(self.orientation == FieldOrientation::Horizontal, |root| {
                root.items_center()
            })
            .child(label_view)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(t.space(1.5))
                    .child(control)
                    .children(description),
            )
        };
        super::theme::apply_style(root, &self.style)
    }
}

/// Stacks related fields with Nova spacing.
pub fn field_group(cx: &App) -> Div {
    div()
        .flex()
        .flex_col()
        .w_full()
        .gap(UiTheme::read(cx).space(7.))
}
/// Stacks a field editor and supporting content.
pub fn field_content(cx: &App) -> Div {
    div()
        .flex()
        .flex_col()
        .flex_1()
        .gap(UiTheme::read(cx).space(1.5))
}
/// Creates a themed field heading.
pub fn field_title(cx: &App) -> Div {
    let t = UiTheme::read(cx);
    div()
        .flex()
        .items_center()
        .gap(t.space(2.))
        .font_family(t.fonts.body.clone())
        .font_weight(FontWeight::MEDIUM)
        .text_size(t.text(14.))
        .line_height(t.text(20.))
        .text_color(t.colors.foreground)
}
/// Creates a horizontal divider between field sections.
pub fn field_separator(cx: &App) -> Div {
    div().w_full().h(px(1.)).bg(UiTheme::read(cx).colors.border)
}

#[cfg(test)]
mod choice_tests {
    use super::super::checkbox::Checkbox;
    use super::super::radio_group::{RadioGroup, RadioItem};
    use super::*;
    use gpui_kit::{
        Context, Entity, Render, TestAppContext, VisualTestContext, test::TestQueryExt as _,
    };

    struct Form {
        checked: bool,
    }

    impl Render for Form {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let view = cx.entity();
            div()
                .flex()
                .flex_col()
                .child(
                    Field::from_control(
                        "notify",
                        Checkbox::new("notify-box").checked(self.checked).on_change(
                            move |next, _, _, cx| {
                                view.update(cx, |form, cx| {
                                    form.checked = next;
                                    cx.notify()
                                })
                            },
                        ),
                    )
                    .label("Notify the channel")
                    .error("Pick at least one option."),
                )
                .child(
                    Field::from_control(
                        "size",
                        RadioGroup::new("size-group")
                            .item(RadioItem::new("s", "s").label("Small"))
                            .item(RadioItem::new("l", "l").label("Large")),
                    )
                    .label("Size")
                    .description("Applies to new posts."),
                )
        }
    }

    #[gpui_kit::test]
    fn checkboxes_and_radio_groups_take_the_field_label_and_note(cx: &mut TestAppContext) {
        cx.update(super::super::theme::init);
        let window = cx.add_window(|_, _| Form { checked: false });
        let view: Entity<Form> = window.root(cx).unwrap();
        let mut cx = VisualTestContext::from_window(window.into(), cx);
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let checkbox = cx.update(|window, _| {
            window.find_by_label("Small");
            window.find_by_label("Notify the channel")
        });
        let bounds = checkbox.bounds();
        // The label sits on the checkbox's row, after it.
        let on_label = gpui_kit::point(bounds.right() + px(40.), bounds.center().y);
        cx.simulate_click(on_label, gpui_kit::Modifiers::default());
        cx.update(|_, cx| {
            assert!(
                view.read(cx).checked,
                "a click on the label toggles the box"
            )
        });
    }
}
