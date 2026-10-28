//! Nova input-group composition around the existing Kit-backed text controls.

use super::{
    button::{Button, ButtonSize, ButtonVariant},
    field::FieldControl,
    input::{Input, Textarea},
    theme::{ThemeMode, UiTheme},
};
use gpui_kit::{
    AnyElement, App, ClickEvent, ElementId, FocusHandle, InteractiveElement as _, IntoElement,
    MouseButton, ParentElement, RenderOnce, Role, SharedString, StatefulInteractiveElement as _,
    StyleRefinement, Styled, Window, div, prelude::FluentBuilder as _,
};

/// The logical side of an addon relative to the text control.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum InputGroupAddonAlignment {
    /// Places the addon before a single-line control.
    #[default]
    InlineStart,
    /// Places the addon after a single-line control.
    InlineEnd,
    /// Places the addon above the control.
    BlockStart,
    /// Places the addon below the control.
    BlockEnd,
}

/// A single-line input or multi-line textarea accepted by [`InputGroup`].
pub enum InputGroupControl {
    /// A single-line input.
    Input(Input),
    /// A multi-line textarea.
    Textarea(Textarea),
}

impl From<Input> for InputGroupControl {
    fn from(value: Input) -> Self {
        Self::Input(value)
    }
}

impl From<Textarea> for InputGroupControl {
    fn from(value: Textarea) -> Self {
        Self::Textarea(value)
    }
}

impl InputGroupControl {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        match self {
            Self::Input(input) => input.field_focus_handle(cx),
            Self::Textarea(textarea) => textarea.field_focus_handle(cx),
        }
    }

    fn is_multiline(&self) -> bool {
        matches!(self, Self::Textarea(_))
    }

    fn is_disabled(&self) -> bool {
        match self {
            Self::Input(input) => input.is_disabled(),
            Self::Textarea(textarea) => textarea.is_disabled(),
        }
    }

    fn is_read_only(&self) -> bool {
        match self {
            Self::Input(input) => input.is_read_only(),
            Self::Textarea(textarea) => textarea.is_read_only(),
        }
    }

    fn is_invalid(&self) -> bool {
        match self {
            Self::Input(input) => input.is_invalid(),
            Self::Textarea(textarea) => textarea.is_invalid(),
        }
    }

    fn render(
        self,
        label: Option<SharedString>,
        disabled: bool,
        read_only: bool,
        invalid: bool,
    ) -> AnyElement {
        match self {
            Self::Input(input) => input
                .when_some(label, |input, label| input.aria_label(label))
                .when(disabled, |input| input.disabled(true))
                .when(read_only, |input| input.read_only(true))
                .when(invalid, |input| input.invalid(true))
                .bordered(false)
                .into_any_element(),
            Self::Textarea(textarea) => textarea
                .when_some(label, |textarea, label| textarea.aria_label(label))
                .when(disabled, |textarea| textarea.disabled(true))
                .when(read_only, |textarea| textarea.read_only(true))
                .when(invalid, |textarea| textarea.invalid(true))
                .bordered(false)
                .into_any_element(),
        }
    }
}

/// A shared Nova frame around one text control and its addons.
#[derive(IntoElement)]
pub struct InputGroup {
    id: ElementId,
    control: InputGroupControl,
    addons: Vec<InputGroupAddon>,
    style: StyleRefinement,
    label: Option<SharedString>,
    disabled: bool,
    read_only: bool,
    invalid: bool,
}

impl InputGroup {
    /// Creates a group around its required text control.
    pub fn new(id: impl Into<ElementId>, control: impl Into<InputGroupControl>) -> Self {
        Self {
            id: id.into(),
            control: control.into(),
            addons: Vec::new(),
            style: Default::default(),
            label: None,
            disabled: false,
            read_only: false,
            invalid: false,
        }
    }

    /// Replaces the group's text control.
    pub fn input(mut self, control: impl Into<InputGroupControl>) -> Self {
        self.control = control.into();
        self
    }

    /// Appends an explicitly aligned addon.
    pub fn addon(mut self, addon: InputGroupAddon) -> Self {
        self.addons.push(addon);
        self
    }

    /// Sets the accessible name of the text control.
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Disables the editor and addon actions.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    /// Prevents edits while preserving selection, copying and addon actions.
    pub fn read_only(mut self, value: bool) -> Self {
        self.read_only = value;
        self
    }

    /// Applies validation error colors and semantics to the editor.
    pub fn invalid(mut self, value: bool) -> Self {
        self.invalid = value;
        self
    }
}

impl Styled for InputGroup {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl FieldControl for InputGroup {
    fn field_focus_handle(&self, cx: &App) -> FocusHandle {
        self.control.focus_handle(cx)
    }

    fn into_field_control(self, label: SharedString, disabled: bool, invalid: bool) -> AnyElement {
        self.aria_label(label)
            .disabled(disabled)
            .invalid(invalid)
            .into_any_element()
    }
}

impl RenderOnce for InputGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let focus = self.control.focus_handle(cx);
        let disabled = self.disabled || self.control.is_disabled();
        let read_only = self.read_only || self.control.is_read_only();
        let invalid = self.invalid || self.control.is_invalid();
        let focused = focus.is_focused(window) && !disabled;
        let multiline = self.control.is_multiline();
        let has_block = self.addons.iter().any(|addon| {
            matches!(
                addon.alignment,
                InputGroupAddonAlignment::BlockStart | InputGroupAddonAlignment::BlockEnd
            )
        });
        let mut inline_start = Vec::new();
        let mut inline_end = Vec::new();
        let mut block_start = Vec::new();
        let mut block_end = Vec::new();
        for addon in self.addons {
            let rendered = addon.render_in_group(disabled, &theme);
            match rendered.0 {
                InputGroupAddonAlignment::InlineStart => inline_start.push(rendered.1),
                InputGroupAddonAlignment::InlineEnd => inline_end.push(rendered.1),
                InputGroupAddonAlignment::BlockStart => block_start.push(rendered.1),
                InputGroupAddonAlignment::BlockEnd => block_end.push(rendered.1),
            }
        }
        let control = self
            .control
            .render(self.label.clone(), disabled, read_only, invalid);
        let background = match theme.mode {
            ThemeMode::Light => colors.background.opacity(0.),
            ThemeMode::Dark => colors.input.opacity(0.3),
        };
        let border = if invalid {
            colors.destructive
        } else if focused {
            colors.ring
        } else {
            colors.input
        };
        let click_focus = focus.clone();
        let root = div()
            .id(self.id)
            .role(Role::Group)
            .w_full()
            .min_w_0()
            .flex()
            .flex_col()
            .rounded(theme.radius.lg)
            .border_1()
            .border_color(border)
            .bg(background)
            .font_family(theme.fonts.body.clone())
            .text_color(colors.foreground)
            .when(!multiline && !has_block, |root| root.h(theme.space(8.)))
            .when(disabled, |root| {
                root.opacity(0.5)
                    .cursor_not_allowed()
                    .capture_any_mouse_down(|_, _, cx| cx.stop_propagation())
                    .capture_key_down(|event, _, cx| {
                        if event.keystroke.key != "tab" {
                            cx.stop_propagation();
                        }
                    })
            })
            .when(!disabled, |root| {
                root.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    if !window.default_prevented() {
                        click_focus.focus(window, cx);
                        window.prevent_default();
                    }
                })
            })
            .children(block_start)
            .child(
                div()
                    .flex()
                    .w_full()
                    .min_w_0()
                    .items_center()
                    .when(!multiline && !has_block, |row| row.h_full())
                    .children(inline_start)
                    .child(div().flex_1().min_w_0().child(control))
                    .children(inline_end),
            )
            .children(block_end);
        super::theme::apply_style(root, &self.style)
    }
}

enum InputGroupAddonChild {
    Element(AnyElement),
    Button(Box<InputGroupButton>),
}

/// Text, icons, buttons or custom content on one side of an input group.
#[derive(IntoElement)]
pub struct InputGroupAddon {
    id: ElementId,
    alignment: InputGroupAddonAlignment,
    children: Vec<InputGroupAddonChild>,
    style: StyleRefinement,
}

impl InputGroupAddon {
    /// Creates an inline-start addon with a stable ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            alignment: Default::default(),
            children: Vec::new(),
            style: Default::default(),
        }
    }

    /// Sets the addon's position around the text control.
    pub fn align(mut self, alignment: InputGroupAddonAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Adds a compact action that inherits the group's disabled state.
    pub fn button(mut self, button: InputGroupButton) -> Self {
        self.children
            .push(InputGroupAddonChild::Button(Box::new(button)));
        self
    }

    fn render_in_group(
        self,
        disabled: bool,
        theme: &UiTheme,
    ) -> (InputGroupAddonAlignment, AnyElement) {
        let alignment = self.alignment;
        let children = self.children.into_iter().map(|child| match child {
            InputGroupAddonChild::Element(child) => child,
            InputGroupAddonChild::Button(button) => (*button).disabled(disabled).into_any_element(),
        });
        let root = div()
            .id(self.id)
            .flex()
            .flex_none()
            .items_center()
            .gap(theme.space(2.))
            .font_family(theme.fonts.body.clone())
            .text_size(theme.text(14.))
            .text_color(theme.colors.muted_foreground)
            .cursor_text()
            .when(
                matches!(alignment, InputGroupAddonAlignment::InlineStart),
                |root| root.pl(theme.space(1.5)),
            )
            .when(
                matches!(alignment, InputGroupAddonAlignment::InlineEnd),
                |root| root.pr(theme.space(1.5)),
            )
            .when(
                matches!(alignment, InputGroupAddonAlignment::BlockStart),
                |root| {
                    root.w_full()
                        .justify_start()
                        .px(theme.space(2.5))
                        .pt(theme.space(2.))
                },
            )
            .when(
                matches!(alignment, InputGroupAddonAlignment::BlockEnd),
                |root| {
                    root.w_full()
                        .justify_start()
                        .px(theme.space(2.5))
                        .pb(theme.space(2.))
                },
            )
            .children(children);
        (
            alignment,
            super::theme::apply_style(root, &self.style).into_any_element(),
        )
    }
}

impl ParentElement for InputGroupAddon {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children
            .extend(elements.into_iter().map(InputGroupAddonChild::Element));
    }
}

impl Styled for InputGroupAddon {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for InputGroupAddon {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        self.render_in_group(false, UiTheme::read(cx)).1
    }
}

/// The single-line control used by an input group.
pub type InputGroupInput = Input;

/// The multi-line control used by an input group.
pub type InputGroupTextarea = Textarea;

/// A compact input-group action backed by gpuicn's Button.
#[derive(IntoElement)]
pub struct InputGroupButton {
    button: Button,
    disabled: bool,
}

impl InputGroupButton {
    /// Creates a compact ghost button with a stable ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            button: Button::new(id)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Xs),
            disabled: false,
        }
    }

    /// Sets the visible button label.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.button = self.button.label(label);
        self
    }

    /// Sets the name announced to accessibility clients.
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.button = self.button.aria_label(label);
        self
    }

    /// Disables the action. A group-level disabled state is combined with this value.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled |= value;
        self
    }

    /// Runs a callback when the enabled button is activated.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.button = self.button.on_click(handler);
        self
    }
}

impl ParentElement for InputGroupButton {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.button.extend(elements);
    }
}

impl Styled for InputGroupButton {
    fn style(&mut self) -> &mut StyleRefinement {
        self.button.style()
    }
}

impl RenderOnce for InputGroupButton {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.button.disabled(self.disabled)
    }
}

/// Muted helper text inside an input group.
#[derive(IntoElement, Default)]
pub struct InputGroupText {
    style: StyleRefinement,
    children: Vec<AnyElement>,
}

impl InputGroupText {
    /// Creates an empty muted text container.
    pub fn new() -> Self {
        Self::default()
    }
}

impl ParentElement for InputGroupText {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Styled for InputGroupText {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for InputGroupText {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx);
        let root = div()
            .flex()
            .items_center()
            .gap(theme.space(2.))
            .text_size(theme.text(14.))
            .text_color(theme.colors.muted_foreground)
            .children(self.children);
        super::theme::apply_style(root, &self.style)
    }
}
