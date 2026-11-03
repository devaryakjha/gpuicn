//! The popover catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn popover_preview(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        popover("dimensions")
            .trigger(popover_trigger("dimensions.trigger", cx).child("Dimensions"))
            .content(|_, window, cx| {
                let width = demo_input("dimensions.width", "100%", "Width", window, cx);
                let height = demo_input("dimensions.height", "25px", "Height", window, cx);
                popover_popup("dimensions.popup", "Dimensions", cx)
                    .child(popover_title(cx).child("Dimensions"))
                    .child(popover_description(cx).child("Set the dimensions for the layer."))
                    .child(Field::new("width", &width).label("Width"))
                    .child(Field::new("height", &height).label("Height"))
            })
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.popover_preview(window, cx).into_any_element()
}
