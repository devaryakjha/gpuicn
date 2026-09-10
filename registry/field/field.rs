//! Nova field layout with explicit input state, label focus and validation messages.

use super::{
    input::{Input, InputState},
    theme::UiTheme,
};
use gpui_kit::{
    App, Div, ElementId, Entity, Focusable as _, FontWeight, InteractiveElement as _, IntoElement,
    MouseButton, ParentElement as _, RenderOnce, Role, SharedString,
    StatefulInteractiveElement as _, Styled, Window, div, prelude::FluentBuilder as _, px,
};

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
/// A labeled Kit input with application-owned validation feedback.
pub struct Field {
    id: ElementId,
    state: Entity<InputState>,
    label: SharedString,
    description: Option<SharedString>,
    error: Option<SharedString>,
    required: bool,
    disabled: bool,
    orientation: FieldOrientation,
    style: gpui_kit::StyleRefinement,
}
impl Field {
    /// Creates a field around the caller's retained editing state.
    pub fn new(id: impl Into<ElementId>, state: &Entity<InputState>) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            label: SharedString::default(),
            description: None,
            error: None,
            required: false,
            disabled: false,
            orientation: FieldOrientation::Vertical,
            style: Default::default(),
        }
    }
    /// Sets the visible label.
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
impl Styled for Field {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}
impl RenderOnce for Field {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = UiTheme::read(cx).clone();
        let focus = self.state.focus_handle(cx);
        let label = if self.required {
            format!("{} (required)", self.label).into()
        } else {
            self.label.clone()
        };
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
            .when(!self.disabled, |label| {
                label
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                        focus.focus(window, cx)
                    })
            });
        let description = self.error.clone().or(self.description.clone());
        let input = Input::new(&self.state)
            .aria_label(label)
            .disabled(self.disabled)
            .invalid(self.error.is_some());
        let root = div()
            .id(self.id)
            .role(Role::Group)
            .w_full()
            .flex()
            .gap(t.space(2.))
            .font_family(t.fonts.body.clone())
            .text_color(t.colors.foreground)
            .when(self.orientation == FieldOrientation::Vertical, |root| {
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
                    .child(input)
                    .when_some(description, |root, description| {
                        root.child(
                            div()
                                .id("description")
                                .when(self.error.is_some(), |description| {
                                    description.role(Role::Alert)
                                })
                                .text_size(t.text(14.))
                                .line_height(t.text(20.))
                                .text_color(if self.error.is_some() {
                                    t.colors.destructive
                                } else {
                                    t.colors.muted_foreground
                                })
                                .child(description),
                        )
                    }),
            );
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
