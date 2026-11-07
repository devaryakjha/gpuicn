//! The color picker catalog preview.

use crate::*;
use gpuicn::color_picker::{ColorPicker, ColorPickerState, ColorSwatch, Oklch};

impl Showcase {
    pub(crate) fn color_picker_preview(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let state = window.use_keyed_state("accent", cx, |_, cx| {
            ColorPickerState::new(Oklch::new(0.55, 0.19, 262.), cx)
        });
        let swatch = state.read(cx).value(cx).to_rgba();
        div()
            .w(px(300.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(div().size(px(20.)).rounded_full().bg(swatch))
                    .child("Accent"),
            )
            .child(ColorPicker::new(&state).swatches([
                ColorSwatch::new("Blue", Oklch::new(0.55, 0.19, 262.)),
                ColorSwatch::new("Teal", Oklch::new(0.6, 0.11, 190.)),
                ColorSwatch::new("Green", Oklch::new(0.6, 0.15, 150.)),
                ColorSwatch::new("Amber", Oklch::new(0.72, 0.15, 70.)),
                ColorSwatch::new("Rose", Oklch::new(0.6, 0.2, 10.)),
                ColorSwatch::new("Violet", Oklch::new(0.55, 0.2, 300.)),
            ]))
    }
}

pub(crate) fn preview(
    view: &Showcase,
    window: &mut Window,
    cx: &mut Context<Showcase>,
) -> gpui_kit::AnyElement {
    view.color_picker_preview(window, cx).into_any_element()
}
