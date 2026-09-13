//! Shared Field configuration checks for mixed control composition.

use std::{cell::RefCell, rc::Rc};

use gpui_kit::{
    AnyElement, App, Context, FocusHandle, IntoElement, Render, SharedString, TestAppContext,
    Window, div,
};
use gpuicn::field::{Field, FieldControl};

type AppliedField = (String, bool, bool);

struct ProbeControl {
    focus: FocusHandle,
    applied: Rc<RefCell<Option<AppliedField>>>,
}

impl FieldControl for ProbeControl {
    fn field_focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }

    fn into_field_control(self, label: SharedString, disabled: bool, invalid: bool) -> AnyElement {
        *self.applied.borrow_mut() = Some((label.to_string(), disabled, invalid));
        div().into_any_element()
    }
}

struct ProbeView {
    focus: FocusHandle,
    applied: Rc<RefCell<Option<AppliedField>>>,
}

impl Render for ProbeView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Field::from_control(
            "mixed-field",
            ProbeControl {
                focus: self.focus.clone(),
                applied: self.applied.clone(),
            },
        )
        .label("Quantity")
        .required(true)
        .disabled(true)
        .error("Enter a quantity.")
    }
}

#[gpui_kit::test]
fn field_applies_shared_state_to_composed_controls(cx: &mut TestAppContext) {
    cx.update(gpuicn::init);
    let applied = Rc::new(RefCell::new(None));
    let captured = applied.clone();
    let (_, visual) = cx.add_window_view(move |_, cx| ProbeView {
        focus: cx.focus_handle(),
        applied,
    });
    visual.update(|window, cx| window.draw(cx).clear(cx));

    assert_eq!(
        captured.borrow().as_ref(),
        Some(&("Quantity (required)".into(), true, true))
    );
}
