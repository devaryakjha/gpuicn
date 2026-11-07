# Color Picker parity

Visual reference: shadcn/ui 4.19.0 has no color picker; this follows its
Slider and Toggle Group in Neutral Nova. Implementation:
`crates/ui/registry/color_picker/color_picker.rs`, using GPUI Kit 0.9.0.

ColorPicker takes a retained `ColorPickerState` holding an OKLCH color. GPUI
Kit's slider owns pointer, keyboard and accessibility for the hue, chroma and
lightness sliders, and its toggle does the same for the optional swatches.
gpuicn supplies the Nova layout and the gradient tracks (`Slider::gradient`),
which preview what moving each slider does. The application owns the swatch
list, what the color is used for, and any contrast checks against its theme.

OKLCH keeps the sliders perceptual: changing the hue keeps the color equally
light and colorful. Colors outside sRGB are clipped when drawn.

The Rust source and Usage example define the supported API.
