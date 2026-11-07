//! Color Picker: picks a color with hue, chroma and lightness sliders in
//! OKLCH, plus optional swatches.
//!
//! shadcn/ui has no color picker; this follows its Slider and Toggle Group
//! in Nova. OKLCH keeps the three sliders perceptual: moving the hue keeps
//! the color as light and as colorful, which an HSL picker cannot. Each
//! slider is gpuicn's [`Slider`] with a gradient track that previews what
//! moving it does, so pointer, keyboard and screen reader behavior come from
//! GPUI Kit's slider. Swatches are GPUI Kit toggles.

use gpui_kit::{
    App, AppContext as _, Context, ElementId, Entity, EventEmitter, InteractiveElement as _,
    IntoElement, ParentElement as _, RenderOnce, Rgba, SharedString, StyleRefinement, Styled,
    Subscription, Window, div, prelude::FluentBuilder as _,
};

use super::slider::{Slider, SliderState};
use super::theme::{
    UiTheme, apply_style,
    locale::{self, DirectionalStyled as _, UiText},
    primitives::{focus, text},
};

/// A color in OKLCH.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Oklch {
    /// Perceived lightness, 0 (black) to 1 (white).
    pub lightness: f32,
    /// Colorfulness, 0 (grey) to about 0.37 at the edge of sRGB.
    pub chroma: f32,
    /// Hue angle in degrees, 0 to 360.
    pub hue: f32,
}

/// The largest chroma the picker offers; past it most hues leave sRGB.
pub const MAX_CHROMA: f32 = 0.37;

impl Oklch {
    /// Creates a color from lightness, chroma and hue.
    pub fn new(lightness: f32, chroma: f32, hue: f32) -> Self {
        Self {
            lightness,
            chroma,
            hue,
        }
    }

    /// The color in sRGB, clipped to its gamut.
    pub fn to_rgba(self) -> Rgba {
        let h = self.hue.to_radians();
        let (a, b) = (self.chroma * h.cos(), self.chroma * h.sin());
        let l = (self.lightness + 0.396_337_78 * a + 0.215_803_76 * b).powi(3);
        let m = (self.lightness - 0.105_561_346 * a - 0.063_854_17 * b).powi(3);
        let s = (self.lightness - 0.089_484_18 * a - 1.291_485_5 * b).powi(3);
        let [r, g, b] = [
            4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s,
            -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s,
            -0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s,
        ]
        .map(|x| {
            let x = x.clamp(0., 1.);
            if x <= 0.003_130_8 {
                12.92 * x
            } else {
                1.055 * x.powf(1. / 2.4) - 0.055
            }
        });
        Rgba { r, g, b, a: 1. }
    }

    /// The nearest OKLCH color to an sRGB one.
    pub fn from_rgba(color: Rgba) -> Self {
        let linear = |c: f32| {
            if c <= 0.040_45 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        let (r, g, b) = (linear(color.r), linear(color.g), linear(color.b));
        let l = (0.412_221_46 * r + 0.536_332_55 * g + 0.051_445_995 * b).cbrt();
        let m = (0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b).cbrt();
        let s = (0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b).cbrt();
        let lightness = 0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s;
        let a = 1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s;
        let b = 0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s;
        let chroma = a.hypot(b);
        let hue = if chroma < 1e-4 {
            0.
        } else {
            b.atan2(a).to_degrees().rem_euclid(360.)
        };
        Self::new(lightness, chroma, hue)
    }
}

/// What a [`ColorPickerState`] tells its subscribers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColorPickerEvent {
    /// The color changed, by a slider, a swatch or [`ColorPickerState::set_value`].
    Change(Oklch),
}

/// The retained state of a [`ColorPicker`]: one slider state per channel.
pub struct ColorPickerState {
    hue: Entity<SliderState>,
    chroma: Entity<SliderState>,
    lightness: Entity<SliderState>,
    last: Oklch,
    _observers: Vec<Subscription>,
}

impl EventEmitter<ColorPickerEvent> for ColorPickerState {}

impl ColorPickerState {
    /// Creates the state holding `value`.
    pub fn new(value: Oklch, cx: &mut Context<Self>) -> Self {
        let slider = |min: f32, max: f32, step: f32, value: f32, cx: &mut Context<Self>| {
            cx.new(|_| {
                SliderState::new()
                    .min(min)
                    .max(max)
                    .step(step)
                    .default_value(value.clamp(min, max))
            })
        };
        let hue = slider(0., 360., 1., value.hue, cx);
        let chroma = slider(0., MAX_CHROMA, 0.005, value.chroma, cx);
        let lightness = slider(0., 1., 0.01, value.lightness, cx);
        let observers = [&hue, &chroma, &lightness]
            .into_iter()
            .map(|slider| {
                cx.observe(slider, |this, _, cx| {
                    let value = this.value(cx);
                    if value != this.last {
                        this.last = value;
                        cx.emit(ColorPickerEvent::Change(value));
                        cx.notify();
                    }
                })
            })
            .collect();
        let mut state = Self {
            hue,
            chroma,
            lightness,
            last: value,
            _observers: observers,
        };
        state.last = state.value(cx);
        state
    }

    /// The color now.
    pub fn value(&self, cx: &App) -> Oklch {
        Oklch::new(
            self.lightness.read(cx).value().end(),
            self.chroma.read(cx).value().end(),
            self.hue.read(cx).value().end(),
        )
    }

    /// Sets the color, clamped to what the sliders offer.
    pub fn set_value(&mut self, value: Oklch, window: &mut Window, cx: &mut Context<Self>) {
        for (slider, channel) in [
            (&self.hue, value.hue.rem_euclid(360.)),
            (&self.chroma, value.chroma),
            (&self.lightness, value.lightness),
        ] {
            slider.update(cx, |state, cx| {
                let channel = channel.clamp(state.min_value(), state.max_value());
                state.set_value(channel, window, cx);
            });
        }
    }
}

/// A preset color offered beside the sliders.
#[derive(Clone, Debug, PartialEq)]
pub struct ColorSwatch {
    /// The swatch's accessible name, such as "Teal".
    pub name: SharedString,
    /// Its color.
    pub color: Oklch,
}

impl ColorSwatch {
    /// Creates a swatch named `name`.
    pub fn new(name: impl Into<SharedString>, color: Oklch) -> Self {
        Self {
            name: name.into(),
            color,
        }
    }
}

/// Picks a color with hue, chroma and lightness sliders in OKLCH, plus
/// optional swatches.
#[derive(IntoElement)]
pub struct ColorPicker {
    state: Entity<ColorPickerState>,
    swatches: Vec<ColorSwatch>,
    disabled: bool,
    style: StyleRefinement,
}

impl ColorPicker {
    /// Creates a picker on the caller's retained state.
    pub fn new(state: &Entity<ColorPickerState>) -> Self {
        Self {
            state: state.clone(),
            swatches: Vec::new(),
            disabled: false,
            style: StyleRefinement::default(),
        }
    }

    /// Offers `swatches` above the sliders; picking one sets every slider.
    pub fn swatches(mut self, swatches: impl IntoIterator<Item = ColorSwatch>) -> Self {
        self.swatches = swatches.into_iter().collect();
        self
    }

    /// Disables every slider and swatch.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Styled for ColorPicker {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

/// `count` evenly spaced colors from `at(0.)` to `at(1.)`.
fn stops(count: usize, at: impl Fn(f32) -> Oklch) -> Vec<Rgba> {
    (0..count)
        .map(|index| at(index as f32 / (count - 1) as f32).to_rgba())
        .collect()
}

impl RenderOnce for ColorPicker {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let state = self.state.read(cx);
        let value = state.value(cx);
        let (hue, chroma, lightness) = (
            state.hue.clone(),
            state.chroma.clone(),
            state.lightness.clone(),
        );
        let row = |label: SharedString, slider: Slider| {
            div()
                .flex()
                .flex_col()
                .gap(theme.space(1.5))
                .child(
                    text::sm(div(), &theme)
                        .text_color(theme.colors.muted_foreground)
                        .child(label),
                )
                .child(slider)
        };
        let swatches = (!self.swatches.is_empty()).then(|| {
            div()
                .flex()
                .flex_row_in(theme.direction)
                .flex_wrap()
                .gap(theme.space(2.))
                .children(
                    self.swatches
                        .into_iter()
                        .enumerate()
                        .map(|(index, swatch)| {
                            let picked = (swatch.color.hue - value.hue).abs() < 0.5
                                && (swatch.color.chroma - value.chroma).abs() < 0.003
                                && (swatch.color.lightness - value.lightness).abs() < 0.006;
                            let target = self.state.clone();
                            let color = swatch.color;
                            gpui_kit::base::Toggle::new(ElementId::from(("color-swatch", index)))
                                .pressed(picked)
                                .disabled(self.disabled)
                                .accessibility_label(swatch.name)
                                .size(theme.space(6.))
                                .rounded_full()
                                .border_2()
                                .border_color(if picked {
                                    theme.colors.foreground
                                } else {
                                    theme.colors.border
                                })
                                .bg(color.to_rgba())
                                .focus_visible({
                                    let theme = theme.clone();
                                    move |s| focus::ring(s, &theme)
                                })
                                .when(!self.disabled, |toggle| {
                                    toggle.on_change(move |_, _, window, cx| {
                                        target.update(cx, |state, cx| {
                                            state.set_value(color, window, cx)
                                        });
                                    })
                                })
                        }),
                )
        });
        let root = div()
            .flex()
            .flex_col()
            .gap(theme.space(4.))
            .w_full()
            .when_some(swatches, |root, swatches| root.child(swatches))
            .child(row(
                locale::text(cx, UiText::ColorHue),
                Slider::new(&hue)
                    .disabled(self.disabled)
                    .aria_label(locale::text(cx, UiText::ColorHue))
                    .gradient(stops(13, |t| Oklch {
                        hue: t * 360.,
                        ..value
                    })),
            ))
            .child(row(
                locale::text(cx, UiText::ColorChroma),
                Slider::new(&chroma)
                    .disabled(self.disabled)
                    .aria_label(locale::text(cx, UiText::ColorChroma))
                    .gradient(stops(6, |t| Oklch {
                        chroma: t * MAX_CHROMA,
                        ..value
                    })),
            ))
            .child(row(
                locale::text(cx, UiText::ColorLightness),
                Slider::new(&lightness)
                    .disabled(self.disabled)
                    .aria_label(locale::text(cx, UiText::ColorLightness))
                    .gradient(stops(6, |t| Oklch {
                        lightness: t,
                        ..value
                    })),
            ));
        apply_style(root, &self.style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{Render, TestAppContext, VisualTestContext};

    #[test]
    fn oklch_round_trips_through_srgb() {
        for color in [
            Oklch::new(0.62, 0.19, 255.),
            Oklch::new(0.75, 0.12, 30.),
            Oklch::new(0.45, 0.08, 150.),
        ] {
            let back = Oklch::from_rgba(color.to_rgba());
            assert!((back.lightness - color.lightness).abs() < 0.01, "{back:?}");
            assert!((back.chroma - color.chroma).abs() < 0.01, "{back:?}");
            assert!((back.hue - color.hue).abs() < 1.5, "{back:?}");
        }
        let white = Oklch::new(1., 0., 0.).to_rgba();
        assert!(white.r > 0.99 && white.g > 0.99 && white.b > 0.99);
        assert_eq!(Oklch::from_rgba(white).chroma.round(), 0.);
    }

    struct Probe {
        state: Entity<ColorPickerState>,
        changes: Vec<Oklch>,
    }

    impl Render for Probe {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            ColorPicker::new(&self.state).swatches([
                ColorSwatch::new("Blue", Oklch::new(0.55, 0.2, 262.)),
                ColorSwatch::new("Rose", Oklch::new(0.62, 0.2, 10.)),
            ])
        }
    }

    #[gpui_kit::test]
    fn draws_and_reports_changes(cx: &mut TestAppContext) {
        cx.update(super::super::theme::init);
        let window = cx.add_window(|_, cx| {
            let state = cx.new(|cx| ColorPickerState::new(Oklch::new(0.55, 0.2, 262.), cx));
            cx.subscribe(&state, |probe: &mut Probe, _, event, _| {
                let ColorPickerEvent::Change(value) = event;
                probe.changes.push(*value);
            })
            .detach();
            Probe {
                state,
                changes: Vec::new(),
            }
        });
        let mut cx = VisualTestContext::from_window(window.into(), cx);
        for mode in [
            super::super::theme::ThemeMode::Light,
            super::super::theme::ThemeMode::Dark,
        ] {
            cx.update(|_, cx| UiTheme::switch(cx, mode));
            cx.update(|window, cx| window.draw(cx).clear(cx));
        }
        let state = window
            .read_with(&cx, |probe, _| probe.state.clone())
            .unwrap();
        cx.update(|window, cx| {
            state.update(cx, |state, cx| {
                state.set_value(Oklch::new(0.62, 0.2, 10.), window, cx)
            })
        });
        cx.run_until_parked();
        let changes = window
            .read_with(&cx, |probe, _| probe.changes.clone())
            .unwrap();
        let last = changes.last().copied().unwrap();
        assert_eq!(last.hue, 10.);
        assert!((last.lightness - 0.62).abs() < 0.001);
        assert_eq!(state.read_with(&cx, |state, cx| state.value(cx)).hue, 10.);
    }
}
