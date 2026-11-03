//! The fieldset catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn fieldset_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let name = demo_input("shipping.name", "", "Ada Lovelace", window, cx);
        let city = demo_input("shipping.city", "", "London", window, cx);
        fieldset("shipping", cx)
            .w(px(300.))
            .aria_label("Shipping address")
            .child(fieldset_legend(FieldsetLegendVariant::Legend, cx).child("Shipping address"))
            .child(Field::new("shipping-name", &name).label("Full name"))
            .child(Field::new("shipping-city", &city).label("City"))
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.fieldset_preview(window, cx).into_any_element()
}
