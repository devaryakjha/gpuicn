//! Nova image lightbox: a full-window viewer with zoom, pan and keyboard
//! previous and next, for images the application supplies.
//!
//! The application passes `ImageSource`s it has already resolved (a cached
//! file, bytes, or an authenticated URL), so the lightbox never fetches.

use gpui_icons::{LucideIcon, lucide};
use gpui_kit::{
    AnyElement, App, Context, ElementId, Entity, EventEmitter, ImageSource,
    InteractiveElement as _, IntoElement, KeyDownEvent, MouseButton, ObjectFit, ParentElement as _,
    Pixels, Point, Render, Role, SharedString, StatefulInteractiveElement as _, Styled,
    StyledImage as _, Window, div, hsla, img, point, prelude::FluentBuilder as _, px,
};

use super::button::{Button, ButtonSize, ButtonVariant};
use super::dialog::{Dialog, DialogHandle, modal_focus};
use super::theme::UiTheme;

/// The largest zoom, as a multiple of the fitted size.
pub const MAX_ZOOM: f32 = 8.;
const ZOOM_STEP: f32 = 1.25;
/// The top bar's height, in spacing units.
const BAR: f32 = 14.;

/// One image in a lightbox.
#[derive(Clone)]
pub struct LightboxImage {
    /// The image, already resolved by the application.
    pub source: ImageSource,
    /// The file name, shown in the top bar and used as the accessible name.
    pub name: SharedString,
    /// Secondary text, such as who posted it and when.
    pub caption: Option<SharedString>,
}

impl LightboxImage {
    /// Creates an image entry.
    pub fn new(source: impl Into<ImageSource>, name: impl Into<SharedString>) -> Self {
        Self {
            source: source.into(),
            name: name.into(),
            caption: None,
        }
    }
    /// Adds secondary text under the name.
    pub fn caption(mut self, caption: impl Into<SharedString>) -> Self {
        self.caption = Some(caption.into());
        self
    }
}

/// Notifications from a lightbox.
#[derive(Clone, Debug, PartialEq)]
pub enum LightboxEvent {
    /// Another image is showing; its index.
    Change(usize),
}

/// Retained lightbox state: the images, the current one, zoom and pan.
pub struct LightboxState {
    images: Vec<LightboxImage>,
    index: usize,
    zoom: f32,
    pan: Point<Pixels>,
    drag: Option<Point<Pixels>>,
    stage: Point<Pixels>,
}

impl EventEmitter<LightboxEvent> for LightboxState {}

impl LightboxState {
    /// Creates a lightbox over the images, showing the first.
    pub fn new(images: impl IntoIterator<Item = LightboxImage>) -> Self {
        Self {
            images: images.into_iter().collect(),
            index: 0,
            zoom: 1.,
            pan: Point::default(),
            drag: None,
            stage: point(px(800.), px(600.)),
        }
    }

    /// Replaces the images and shows `index`, for example the images of the
    /// post that was clicked.
    pub fn set_images(
        &mut self,
        images: impl IntoIterator<Item = LightboxImage>,
        index: usize,
        cx: &mut Context<Self>,
    ) {
        self.images = images.into_iter().collect();
        self.show(index, cx);
    }

    /// The images.
    pub fn images(&self) -> &[LightboxImage] {
        &self.images
    }

    /// The index of the showing image.
    pub fn index(&self) -> usize {
        self.index
    }

    /// The showing image.
    pub fn current(&self) -> Option<&LightboxImage> {
        self.images.get(self.index)
    }

    /// Shows image `index` at the fitted size.
    pub fn show(&mut self, index: usize, cx: &mut Context<Self>) {
        let index = index.min(self.images.len().saturating_sub(1));
        let changed = index != self.index;
        self.index = index;
        self.reset_zoom(cx);
        if changed {
            cx.emit(LightboxEvent::Change(index));
        }
    }

    /// Shows the next image, wrapping to the first.
    pub fn next(&mut self, cx: &mut Context<Self>) {
        if !self.images.is_empty() {
            self.show((self.index + 1) % self.images.len(), cx);
        }
    }

    /// Shows the previous image, wrapping to the last.
    pub fn previous(&mut self, cx: &mut Context<Self>) {
        if !self.images.is_empty() {
            self.show((self.index + self.images.len() - 1) % self.images.len(), cx);
        }
    }

    /// The zoom, where 1 fits the image to the window.
    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    /// Sets the zoom, from 1 (fitted) to `MAX_ZOOM`.
    pub fn set_zoom(&mut self, zoom: f32, cx: &mut Context<Self>) {
        let previous = self.zoom;
        self.zoom = zoom.clamp(1., MAX_ZOOM);
        // Keep the same point of the image at the center.
        self.pan *= self.zoom / previous;
        self.clamp_pan();
        cx.notify();
    }

    /// Zooms in one step.
    pub fn zoom_in(&mut self, cx: &mut Context<Self>) {
        self.set_zoom(self.zoom * ZOOM_STEP, cx);
    }

    /// Zooms out one step.
    pub fn zoom_out(&mut self, cx: &mut Context<Self>) {
        self.set_zoom(self.zoom / ZOOM_STEP, cx);
    }

    /// Fits the image to the window again.
    pub fn reset_zoom(&mut self, cx: &mut Context<Self>) {
        self.zoom = 1.;
        self.pan = Point::default();
        self.drag = None;
        cx.notify();
    }

    /// How far the image is moved from the center.
    pub fn pan(&self) -> Point<Pixels> {
        self.pan
    }

    /// Moves the image by `delta`, within its zoomed edges.
    pub fn pan_by(&mut self, delta: Point<Pixels>, cx: &mut Context<Self>) {
        self.pan += delta;
        self.clamp_pan();
        cx.notify();
    }

    fn clamp_pan(&mut self) {
        let limit = |stage: Pixels| (stage * (self.zoom - 1.) / 2.).max(px(0.));
        self.pan.x = self.pan.x.clamp(-limit(self.stage.x), limit(self.stage.x));
        self.pan.y = self.pan.y.clamp(-limit(self.stage.y), limit(self.stage.y));
    }

    /// Handles a key press: Left and Right change images, `+`, `-` and `0`
    /// zoom, and arrow keys with Shift pan a zoomed image. Returns whether it
    /// was handled.
    pub fn handle_key(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) -> bool {
        let keystroke = &event.keystroke;
        let modifiers = keystroke.modifiers;
        if modifiers.control || modifiers.alt || modifiers.platform {
            return false;
        }
        let pan = px(48.);
        match (keystroke.key.as_str(), modifiers.shift) {
            ("left", true) => self.pan_by(point(pan, px(0.)), cx),
            ("right", true) => self.pan_by(point(-pan, px(0.)), cx),
            ("up", true) => self.pan_by(point(px(0.), pan), cx),
            ("down", true) => self.pan_by(point(px(0.), -pan), cx),
            ("left", false) => self.previous(cx),
            ("right", false) => self.next(cx),
            ("+" | "=", _) => self.zoom_in(cx),
            ("-", false) => self.zoom_out(cx),
            ("0", false) => self.reset_zoom(cx),
            _ => return false,
        }
        true
    }
}

impl Render for LightboxState {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let id = ElementId::from(("lightbox", cx.entity_id()));
        let viewport = window.viewport_size();
        let bar = theme.space(BAR);
        self.stage = point(viewport.width, (viewport.height - bar).max(px(0.)));
        self.clamp_pan();
        let white = hsla(0., 0., 1., 1.);
        let count = self.images.len();
        let current = self.current().cloned();
        let zoom = self.zoom;
        let icon_button = |id: ElementId, label: &'static str, icon: LucideIcon| {
            Button::new(id)
                .size(ButtonSize::Icon)
                .variant(ButtonVariant::Ghost)
                .aria_label(label)
                .child(lucide(icon).size(theme.space(4.5)).text_color(white))
        };
        let side = |button: Button| {
            div()
                .absolute()
                .top_1_2()
                .mt(-theme.space(5.))
                .size(theme.space(10.))
                .rounded_full()
                .bg(hsla(0., 0., 0., 0.5))
                .flex()
                .items_center()
                .justify_center()
                .child(button)
        };

        div()
            .id((id.clone(), "stage"))
            .relative()
            .w(self.stage.x)
            .h(self.stage.y)
            .overflow_hidden()
            .when(zoom > 1., |stage| {
                stage.cursor(if self.drag.is_some() {
                    gpui_kit::CursorStyle::ClosedHand
                } else {
                    gpui_kit::CursorStyle::OpenHand
                })
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &gpui_kit::MouseDownEvent, _, cx| {
                    if this.zoom > 1. {
                        this.drag = Some(event.position);
                        cx.notify();
                    }
                }),
            )
            .on_mouse_move(
                cx.listener(|this, event: &gpui_kit::MouseMoveEvent, _, cx| {
                    if let Some(last) = this.drag {
                        if event.pressed_button != Some(MouseButton::Left) {
                            this.drag = None;
                            cx.notify();
                            return;
                        }
                        this.drag = Some(event.position);
                        this.pan_by(event.position - last, cx);
                    }
                }),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.drag = None;
                    cx.notify();
                }),
            )
            .on_scroll(cx.listener(|this, event: &gpui_kit::ScrollEvent, _, cx| {
                let delta = event.delta.pixel_delta(px(16.)).y;
                let factor = (1. + f32::from(delta) * 0.004).clamp(0.5, 2.);
                this.set_zoom(this.zoom * factor, cx);
                cx.stop_propagation();
            }))
            .on_click(cx.listener(|this, event: &gpui_kit::ClickEvent, _, cx| {
                // Double-click toggles between fitted and 2×.
                if event.click_count() == 2 {
                    if this.zoom > 1. {
                        this.reset_zoom(cx);
                    } else {
                        this.set_zoom(2., cx);
                    }
                }
            }))
            .when_some(current.clone(), |stage, image| {
                let width = self.stage.x * zoom;
                let height = self.stage.y * zoom;
                stage.child(
                    img(image.source)
                        .absolute()
                        .left((self.stage.x - width) / 2. + self.pan.x)
                        .top((self.stage.y - height) / 2. + self.pan.y)
                        .w(width)
                        .h(height)
                        .object_fit(ObjectFit::Contain),
                )
            })
            .when(count > 1, |stage| {
                stage
                    .child(
                        side(
                            icon_button(
                                (id.clone(), "previous").into(),
                                "Previous image",
                                LucideIcon::ChevronLeft,
                            )
                            .on_click(cx.listener(|this, _, _, cx| this.previous(cx))),
                        )
                        .left(theme.space(4.)),
                    )
                    .child(
                        side(
                            icon_button(
                                (id.clone(), "next").into(),
                                "Next image",
                                LucideIcon::ChevronRight,
                            )
                            .on_click(cx.listener(|this, _, _, cx| this.next(cx))),
                        )
                        .right(theme.space(4.)),
                    )
            })
    }
}

/// A full-window lightbox opened and closed with `handle`.
///
/// Keep it mounted while closed, like `dialog`. Toolbar `actions` render
/// before the close button, such as a download or open-in-browser button.
/// Escape and the close button close it; focus returns to where it was.
pub fn lightbox(
    id: impl Into<ElementId>,
    handle: &DialogHandle,
    state: &Entity<LightboxState>,
    actions: impl IntoIterator<Item = AnyElement>,
    window: &mut Window,
    cx: &mut App,
) -> Dialog {
    let id = id.into();
    let theme = UiTheme::read(cx).clone();
    let open = handle.is_open();
    let focus = modal_focus::prepare(id.clone(), open, window, cx);
    let name = state
        .read(cx)
        .current()
        .map(|image| image.name.clone())
        .unwrap_or_else(|| "Image".into());
    let close = handle.clone();
    let escape = handle.clone();
    let (current, index, count, zoom) = {
        let state = state.read(cx);
        (
            state.current().cloned(),
            state.index(),
            state.images().len(),
            state.zoom(),
        )
    };
    let white = hsla(0., 0., 1., 1.);
    let dim = hsla(0., 0., 1., 0.7);
    let icon_button = |id: ElementId, label: &'static str, icon: LucideIcon| {
        Button::new(id)
            .size(ButtonSize::Icon)
            .variant(ButtonVariant::Ghost)
            .aria_label(label)
            .child(lucide(icon).size(theme.space(4.5)).text_color(white))
    };
    let zoom_out = state.clone();
    let zoom_in = state.clone();
    let top = div()
        .flex_none()
        .h(theme.space(BAR))
        .flex()
        .items_center()
        .gap(theme.space(2.))
        .px(theme.space(4.))
        .text_color(white)
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .when_some(current, |title, image| {
                    title
                        .child(
                            div()
                                .overflow_hidden()
                                .text_ellipsis()
                                .whitespace_nowrap()
                                .font_weight(gpui_kit::FontWeight::MEDIUM)
                                .child(image.name),
                        )
                        .children(image.caption.map(|caption| {
                            div()
                                .overflow_hidden()
                                .text_ellipsis()
                                .whitespace_nowrap()
                                .text_size(theme.text(12.))
                                .text_color(dim)
                                .child(caption)
                        }))
                }),
        )
        .when(count > 1, |top| {
            top.child(
                div()
                    .text_color(dim)
                    .text_size(theme.text(13.))
                    .mr(theme.space(2.))
                    .child(format!("{} of {count}", index + 1)),
            )
        })
        .child(
            icon_button(
                (id.clone(), "zoom-out").into(),
                "Zoom out",
                LucideIcon::ZoomOut,
            )
            .on_click(move |_, _, cx| zoom_out.update(cx, |state, cx| state.zoom_out(cx))),
        )
        .child(
            div()
                .id((id.clone(), "zoom"))
                .role(Role::Status)
                .aria_label(SharedString::from(format!(
                    "Zoom {}%",
                    (zoom * 100.).round() as i32
                )))
                .w(theme.space(12.))
                .text_center()
                .text_size(theme.text(13.))
                .child(format!("{}%", (zoom * 100.).round() as i32)),
        )
        .child(
            icon_button(
                (id.clone(), "zoom-in").into(),
                "Zoom in",
                LucideIcon::ZoomIn,
            )
            .on_click(move |_, _, cx| zoom_in.update(cx, |state, cx| state.zoom_in(cx))),
        )
        .children(actions)
        .child(
            icon_button((id.clone(), "close").into(), "Close", LucideIcon::X)
                .on_click(move |_, window, cx| close.close(window, cx)),
        );
    let key_state = state.clone();
    let popup = div()
        .id(id.clone())
        .role(Role::Dialog)
        .aria_label(SharedString::from(format!("Image viewer: {name}")))
        .track_focus(&focus)
        .relative()
        .size_full()
        .occlude()
        .on_key_down(move |event, window, cx| {
            if event.keystroke.key == "escape" {
                escape.close(window, cx);
                cx.stop_propagation();
            } else if key_state.update(cx, |state, cx| state.handle_key(event, cx)) {
                cx.stop_propagation();
            }
        })
        .flex()
        .flex_col()
        .font_family(theme.fonts.body.clone())
        .text_size(theme.text(14.))
        .child(top)
        .child(state.clone());
    Dialog::new(cx)
        .handle(handle.clone())
        .close_on_escape(false)
        .close_on_backdrop_press(false)
        .backdrop(
            div()
                .absolute()
                .inset_0()
                .bg(hsla(0., 0., 0., 0.92))
                .occlude(),
        )
        .popup(popup)
}

#[cfg(test)]
mod tests {
    use super::{LightboxImage, LightboxState, MAX_ZOOM};
    use gpui_kit::{AppContext as _, TestAppContext, point, px};

    #[gpui_kit::test]
    fn zoom_and_pan_stay_within_the_image(cx: &mut TestAppContext) {
        let state = cx.new(|_| {
            LightboxState::new([
                LightboxImage::new("a.png", "a.png"),
                LightboxImage::new("b.png", "b.png"),
            ])
        });
        state.update(cx, |state, cx| {
            // At the fitted size the image cannot move.
            state.pan_by(point(px(100.), px(0.)), cx);
            assert_eq!(state.pan(), point(px(0.), px(0.)));
            state.set_zoom(2., cx);
            // The default stage is 800×600, so a 2× image moves 400×300 at most.
            state.pan_by(point(px(1000.), px(-50.)), cx);
            assert_eq!(state.pan(), point(px(400.), px(-50.)));
            state.set_zoom(100., cx);
            assert_eq!(state.zoom(), MAX_ZOOM);
            state.next(cx);
            assert_eq!((state.index(), state.zoom()), (1, 1.));
            state.next(cx);
            assert_eq!(state.index(), 0);
            state.previous(cx);
            assert_eq!(state.index(), 1);
        });
    }
}
