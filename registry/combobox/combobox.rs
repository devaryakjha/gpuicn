//! The shadcn Nova Combobox visual port.
//!
//! Visual source: shadcn/ui 4.19.0 `combobox.tsx` and `style-nova.css` at
//! `1773ecfeeb4a04366978d353e69b5c7ded78dcb2`. Interaction comes from the
//! pinned Base GPUI combobox primitives.

pub use base_gpui::combobox::{
    ComboboxAlign, ComboboxArrow, ComboboxBackdrop, ComboboxChip, ComboboxChipRemove,
    ComboboxChips, ComboboxClear, ComboboxCollection, ComboboxEmpty, ComboboxGroup,
    ComboboxGroupLabel, ComboboxIcon, ComboboxInput, ComboboxInputGroup, ComboboxItem,
    ComboboxItemIndicator, ComboboxLabel, ComboboxList, ComboboxPopup, ComboboxPortal,
    ComboboxPositioner, ComboboxRoot, ComboboxSelectionMode, ComboboxSeparator, ComboboxSide,
    ComboboxStatus, ComboboxTrigger, ComboboxValue,
};
use gpui::{
    App, Div, ElementId, FontWeight, InteractiveElement as _, ParentElement as _, Styled,
    prelude::FluentBuilder as _, px,
};
use gpui_icons::{LucideIcon, lucide};

use super::theme::{UiTheme, input_text_layout};

/// Creates a combobox root with a caller-owned stable ID.
pub fn combobox_root<T: Clone + Eq + 'static>(id: impl Into<ElementId>) -> ComboboxRoot<T> {
    ComboboxRoot::new().id(id)
}

/// Creates the styled editable combobox input.
pub fn combobox_input<T: Clone + Eq + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> ComboboxInput<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    let input_theme = theme.clone();
    ComboboxInput::new()
        .id(id)
        .style_with_state(move |state, base| {
            let focus_ring = if state.root.invalid {
                theme.destructive_focus_ring()
            } else {
                theme.focus_ring()
            };
            let border = if state.root.invalid {
                theme.colors.destructive
            } else {
                theme.colors.input
            };
            base.flex()
                .items_center()
                .h(spacing * 8_f32)
                .rounded(theme.radius.lg)
                .border_1()
                .border_color(border)
                .bg(theme.colors.background)
                .when(state.root.focused, |base| {
                    base.border_color(if state.root.invalid {
                        border
                    } else {
                        theme.colors.ring
                    })
                    .shadow(focus_ring.clone())
                })
                .when(state.root.disabled, |base| {
                    base.opacity(0.5).cursor_not_allowed()
                })
        })
        .input_style_with_state(move |_state, base| {
            input_text_layout(base, text_scale)
                .w_full()
                .h_full()
                .px(spacing * 2.5_f32)
                .font_family(input_theme.fonts.body.clone())
                .text_size(px(14.) * text_scale)
                .text_color(input_theme.colors.foreground)
        })
}

/// Creates the borderless input used inside [`combobox_input_group`].
pub fn combobox_group_input<T: Clone + Eq + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> ComboboxInput<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    ComboboxInput::new()
        .id(id)
        .style_with_state(move |_state, base| {
            base.flex().items_center().h(spacing * 8. - px(2.)).flex_1()
        })
        .input_style_with_state(move |_state, base| {
            input_text_layout(base, text_scale)
                .w_full()
                .h_full()
                .font_family(theme.fonts.body.clone())
                .text_size(px(14.) * text_scale)
                .text_color(theme.colors.foreground)
        })
}

/// Creates the styled input group used by chips and custom combobox layouts.
pub fn combobox_input_group<T: Clone + Eq + 'static>(cx: &App) -> ComboboxInputGroup<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    ComboboxInputGroup::new().style_with_state(move |state, base| {
        let focus_ring = if state.root.invalid {
            theme.destructive_focus_ring()
        } else {
            theme.focus_ring()
        };
        let border = if state.root.invalid {
            theme.colors.destructive
        } else {
            theme.colors.input
        };
        base.flex()
            .items_center()
            .min_h(spacing * 8_f32)
            .gap(spacing * 1_f32)
            .rounded(theme.radius.lg)
            .border_1()
            .border_color(border)
            .px(spacing * 2.5_f32)
            .bg(if theme.mode == super::theme::ThemeMode::Dark {
                theme.colors.input.opacity(0.30)
            } else {
                theme.colors.background
            })
            .when(state.root.focused, |base| {
                base.border_color(if state.root.invalid {
                    border
                } else {
                    theme.colors.ring
                })
                .shadow(focus_ring.clone())
            })
            .when(state.root.disabled, |base| {
                base.opacity(0.5).cursor_not_allowed()
            })
    })
}

/// Creates the styled combobox trigger with Nova's chevron.
pub fn combobox_trigger<T: Clone + Eq + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> ComboboxTrigger<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let icon_color = theme.colors.muted_foreground;
    let id = id.into();
    let press_id = ElementId::NamedChild(std::sync::Arc::new(id.clone()), "pointer-open".into());
    ComboboxTrigger::new()
        .id(id)
        .style_with_state(move |state, base| {
            let press_id = press_id.clone();
            let pressed = std::rc::Rc::new(std::cell::RefCell::new(None::<gpui::Entity<bool>>));
            let measured = pressed.clone();
            let released = pressed.clone();
            let was_open = state.root.open;
            base.flex()
                .size(spacing * 6_f32)
                .items_center()
                .justify_center()
                .rounded(theme.radius.sm)
                .text_color(theme.colors.muted_foreground)
                // The group opens on down; the popup closes on outside-down. Keep either
                // path from undoing the trigger's intended toggle on mouse-up.
                .on_children_prepainted(move |_, window, cx| {
                    *measured.borrow_mut() =
                        Some(window.use_keyed_state(press_id.clone(), cx, |_, _| false));
                })
                .on_mouse_down(gpui::MouseButton::Left, move |_, _, cx| {
                    if let Some(pressed) = pressed.borrow().as_ref() {
                        pressed.update(cx, |open, _| *open = was_open);
                    }
                    cx.stop_propagation();
                })
                .capture_any_mouse_up(move |event, _, cx| {
                    if event.button == gpui::MouseButton::Left
                        && released.borrow().as_ref().is_some_and(|pressed| {
                            pressed.update(cx, |open, _| std::mem::take(open))
                        })
                    {
                        cx.stop_propagation();
                    }
                })
                .when(state.root.open, |base| base.bg(theme.colors.muted))
                .when(state.root.disabled, |base| base.cursor_not_allowed())
                .when(!state.root.disabled && !state.root.read_only, |base| {
                    base.cursor_pointer()
                        .hover(move |style| style.bg(theme.colors.muted))
                })
        })
        .child(
            lucide(LucideIcon::ChevronDown)
                .size(spacing * 4_f32)
                .text_color(icon_color),
        )
}

/// Creates the styled clear control for a combobox.
pub fn combobox_clear<T: Clone + Eq + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> ComboboxClear<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    ComboboxClear::new()
        .id(id)
        .style_with_state(move |state, base| {
            base.flex()
                .size(spacing * 6_f32)
                .items_center()
                .justify_center()
                .rounded(theme.radius.sm)
                .text_color(theme.colors.muted_foreground)
                .when(state.disabled, |base| base.cursor_not_allowed())
                .when(!state.disabled, |base| {
                    base.cursor_pointer()
                        .hover(move |style| style.bg(theme.colors.muted))
                })
                .when(state.disabled, |base| {
                    base.opacity(0.5).cursor_not_allowed()
                })
        })
        .child(
            lucide(LucideIcon::X)
                .size(spacing * 3.5_f32)
                .text_color(theme.colors.muted_foreground),
        )
}

/// Creates the in-canvas combobox portal.
pub fn combobox_portal<T: Clone + Eq + 'static>() -> ComboboxPortal<T> {
    ComboboxPortal::new()
}

/// Creates a combobox positioner with the pinned 6px content offset.
pub fn combobox_positioner<T: Clone + Eq + 'static>(cx: &App) -> ComboboxPositioner<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    ComboboxPositioner::new()
        .side_offset(spacing * 1.5_f32)
        .style_with_state(|state, base| {
            base.when_some(state.anchor_width, |base, width| base.min_w(width))
        })
}

/// Creates the styled combobox popup.
pub fn combobox_popup<T: Clone + Eq + 'static>(cx: &App) -> ComboboxPopup<T> {
    let theme = UiTheme::read(cx).clone();
    ComboboxPopup::new().style_with_state(move |_state, base| popup_style(base, &theme))
}

/// Creates the styled combobox list.
pub fn combobox_list<T: Clone + Eq + 'static>(cx: &App) -> ComboboxList<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    ComboboxList::new().style_with_state(move |_state, base| {
        base.max_h(spacing * 63_f32)
            .overflow_hidden()
            .p(spacing * 1_f32)
    })
}

/// Creates a styled combobox item.
pub fn combobox_item<T: Clone + Eq + 'static>(
    id: impl Into<ElementId>,
    cx: &App,
) -> ComboboxItem<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    ComboboxItem::new()
        .id(id)
        .style_with_state(move |state, base| {
            base.relative()
                .flex()
                .items_center()
                .gap(spacing * 2_f32)
                .rounded(theme.radius.sm)
                .py(spacing * 1_f32)
                .pr(spacing * 8_f32)
                .pl(spacing * 1.5_f32)
                .font_family(theme.fonts.body.clone())
                .text_size(px(14.) * text_scale)
                .text_color(theme.colors.popover_foreground)
                .when(!state.disabled, |base| base.cursor_pointer())
                .when(state.highlighted && !state.disabled, |base| {
                    base.bg(theme.colors.accent)
                        .text_color(theme.colors.accent_foreground)
                })
                .when(state.disabled, |base| {
                    base.opacity(0.5).cursor_not_allowed()
                })
        })
        .child(combobox_item_indicator(cx))
}

/// Creates the selected-item check indicator.
pub fn combobox_item_indicator<T: Clone + Eq + 'static>(cx: &App) -> ComboboxItemIndicator<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    ComboboxItemIndicator::new()
        .keep_mounted(true)
        .style_with_state(move |state, base| {
            base.absolute()
                .right(spacing * 2_f32)
                .flex()
                .size(spacing * 4_f32)
                .items_center()
                .justify_center()
                .opacity(if state.selected { 1.0 } else { 0.0 })
        })
        .child(
            lucide(LucideIcon::Check)
                .size(spacing * 4_f32)
                .text_color(theme.colors.foreground),
        )
}

/// Creates a styled combobox group.
pub fn combobox_group<T: Clone + Eq + 'static>(cx: &App) -> ComboboxGroup<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    ComboboxGroup::new().style_with_state(move |_state, base| base.p(spacing * 1_f32))
}

/// Creates a styled combobox group label.
pub fn combobox_group_label<T: Clone + Eq + 'static>(cx: &App) -> ComboboxGroupLabel<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    ComboboxGroupLabel::new().style_with_state(move |_state, base| {
        base.px(spacing * 2_f32)
            .py(spacing * 1.5_f32)
            .font_family(theme.fonts.body.clone())
            .text_size(px(12.) * text_scale)
            .text_color(theme.colors.muted_foreground)
    })
}

/// Creates the styled empty state container.
pub fn combobox_empty<T: Clone + Eq + 'static>(cx: &App) -> ComboboxEmpty<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    ComboboxEmpty::new().style_with_state(move |_state, base| {
        base.flex()
            .w_full()
            .justify_center()
            .px(spacing * 2_f32)
            .py(spacing * 2_f32)
            .font_family(theme.fonts.body.clone())
            .text_size(px(14.) * text_scale)
            .text_color(theme.colors.muted_foreground)
    })
}

/// Creates a styled combobox separator.
pub fn combobox_separator(cx: &App) -> ComboboxSeparator {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    ComboboxSeparator::new().style_with_state(move |_state, base| {
        base.h(px(1.))
            .mx(spacing * -1_f32)
            .my(spacing * 1_f32)
            .bg(theme.colors.border)
    })
}

/// Creates the styled container for selected chips.
pub fn combobox_chips<T: Clone + Eq + 'static>(cx: &App) -> ComboboxChips<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    ComboboxChips::new().style_with_state(move |_state, base| {
        base.flex()
            .flex_wrap()
            .items_center()
            .gap(spacing * 1_f32)
            .font_family(theme.fonts.body.clone())
    })
}

/// Creates one styled selected-value chip.
pub fn combobox_chip<T: Clone + Eq + 'static>(cx: &App) -> ComboboxChip<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    ComboboxChip::new().style_with_state(move |state, base| {
        base.flex()
            .items_center()
            .gap(spacing * 1_f32)
            .h(spacing * 5.25_f32)
            .rounded(theme.radius.sm * (2. / 3.))
            .px(spacing * 1.5_f32)
            .bg(theme.colors.muted)
            .font_family(theme.fonts.body.clone())
            .font_weight(FontWeight::MEDIUM)
            .text_size(px(12.) * text_scale)
            .text_color(theme.colors.foreground)
            .when(state.highlighted, |base| base.bg(theme.colors.accent))
            .when(state.disabled, |base| {
                base.opacity(0.5).cursor_not_allowed()
            })
    })
}

/// Creates the styled remove control for a selected-value chip.
pub fn combobox_chip_remove<T: Clone + Eq + 'static>(cx: &App) -> ComboboxChipRemove<T> {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    ComboboxChipRemove::new()
        .style_with_state(move |state, base| {
            base.flex()
                .size(spacing * 4_f32)
                .items_center()
                .justify_center()
                .rounded(theme.radius.sm * 0.5)
                .text_color(theme.colors.muted_foreground)
                .when(state.disabled, |base| base.cursor_not_allowed())
                .when(!state.disabled, |base| {
                    base.cursor_pointer()
                        .hover(move |style| style.bg(theme.colors.background))
                })
        })
        .child(
            lucide(LucideIcon::X)
                .size(spacing * 3_f32)
                .text_color(theme.colors.muted_foreground),
        )
}

/// Creates the styled text value shown by non-chip comboboxes.
pub fn combobox_value<T: Clone + Eq + 'static>() -> ComboboxValue<T> {
    ComboboxValue::new().style_with_state(move |_state, base| base.flex_1())
}

fn popup_style(base: Div, theme: &UiTheme) -> Div {
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    base.min_w(spacing * 36_f32)
        .max_h(spacing * 63_f32)
        .overflow_hidden()
        .rounded(theme.radius.lg)
        .bg(theme.colors.popover)
        .text_color(theme.colors.popover_foreground)
        .font_family(theme.fonts.body.clone())
        .text_size(px(14.) * text_scale)
        .border_1()
        .border_color(theme.colors.foreground.opacity(0.10))
        .shadow(theme.shadows.md.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{
        AppContext as _, Bounds, Context, IntoElement, Pixels, Render, TestAppContext, Window,
    };
    use std::{cell::RefCell, rc::Rc};

    struct TriggerView(Rc<RefCell<Vec<bool>>>);
    impl Render for TriggerView {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let changes = self.0.clone();
            gpui::div().w(px(240.)).child(
                combobox_root::<String>("trigger-test")
                    .on_open_change(move |open, _, _, _| changes.borrow_mut().push(open))
                    .child(
                        combobox_input_group(cx)
                            .child(combobox_group_input("trigger-test.input", cx))
                            .child(combobox_trigger("trigger-test.trigger", cx)),
                    )
                    .child(
                        combobox_portal().child(
                            combobox_positioner(cx).child(
                                combobox_popup(cx).child(
                                    combobox_list(cx).child(
                                        combobox_item("trigger-test.item", cx)
                                            .value("Apple".into())
                                            .child_any("Apple"),
                                    ),
                                ),
                            ),
                        ),
                    ),
            )
        }
    }

    #[test]
    fn grouped_trigger_opens_on_one_click() {
        use gpui::{Modifiers, VisualTestContext, point};
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            crate::init(cx);
            UiTheme::set(cx, UiTheme::neutral_light());
        });
        let changes = Rc::new(RefCell::new(Vec::new()));
        let captured = changes.clone();
        let window = cx.add_window(move |_, _| TriggerView(captured));
        let mut visual = VisualTestContext::from_window(window.into(), &cx);
        visual.simulate_click(point(px(219.), px(16.)), Modifiers::default());
        assert_eq!(*changes.borrow(), vec![true]);
        visual.simulate_click(point(px(219.), px(16.)), Modifiers::default());
        assert_eq!(*changes.borrow(), vec![true, false]);
    }

    struct View {
        grouped: bool,
        bounds: Rc<RefCell<Vec<Bounds<Pixels>>>>,
    }
    impl Render for View {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let bounds = self.bounds.clone();
            let input = if self.grouped {
                combobox_group_input::<String>("editor", cx)
            } else {
                combobox_input::<String>("editor", cx)
            };
            gpui::div().w(px(240.)).child(
                combobox_root::<String>("alignment").child(
                    input
                        .placeholder("Search fruits…")
                        .input_style_with_state(move |_, base| {
                            let bounds = bounds.clone();
                            input_text_layout(base, 1.)
                                .w_full()
                                .h_full()
                                .text_size(px(14.))
                                .on_children_prepainted(move |children, _, _| {
                                    *bounds.borrow_mut() = children.to_vec()
                                })
                        }),
                ),
            )
        }
    }
    #[test]
    fn combobox_wrapper_centers_the_editor() {
        for grouped in [false, true] {
            let mut cx = TestAppContext::single();
            cx.update(|cx| UiTheme::set(cx, UiTheme::neutral_light()));
            let bounds = Rc::new(RefCell::new(Vec::new()));
            let captured = bounds.clone();
            let window = cx.add_window(move |_, _| View {
                grouped,
                bounds: captured,
            });
            cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
                .unwrap();
            let bounds = bounds.borrow();
            assert_eq!(bounds.len(), 1);
            assert_eq!(
                bounds[0].center().y,
                px(if grouped { 15. } else { 16. }),
                "grouped={grouped}: {:?}",
                bounds[0]
            );
        }
    }
}
