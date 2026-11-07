//! Nova image lightbox: a full-window viewer with zoom, pan and keyboard
//! previous and next, for images the application supplies.
//!
//! The application passes `ImageSource`s it has already resolved (a cached
//! file, bytes, or an authenticated URL), so the lightbox never fetches.

use gpui_icons::{LucideIcon, lucide};
use gpui_kit::{
    AnyElement, App, Context, ElementId, Entity, EventEmitter, ImageSource,
    InteractiveElement as _, IntoElement, KeyDownEvent, MouseButton, ObjectFit, ParentElement as _,
    Pixels, Point, Render, Role, SharedString, Size, StatefulInteractiveElement as _, Styled,
    StyledImage as _, Window, div, hsla, point, prelude::FluentBuilder as _, px, size,
};

use super::button::{Button, ButtonSize, ButtonVariant};
use super::dialog::{Dialog, DialogHandle, modal_focus};
use super::theme::{
    UiText, UiTheme,
    locale::{self, DirectionalStyled as _},
    primitives::dismiss,
};

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
    /// The image's natural size in pixels, when known. A known size keeps a
    /// small image at that size instead of blowing it up to the window.
    pub size: Option<Size<Pixels>>,
}

impl LightboxImage {
    /// Creates an image entry.
    pub fn new(source: impl Into<ImageSource>, name: impl Into<SharedString>) -> Self {
        Self {
            source: source.into(),
            name: name.into(),
            caption: None,
            size: None,
        }
    }
    /// Sets the image's natural size in pixels, such as the width and
    /// height a server reports for an upload. The fitted image is never
    /// larger than this, so small images stay sharp.
    pub fn dimensions(mut self, width: u32, height: u32) -> Self {
        self.size = (width > 0 && height > 0).then(|| size(px(width as f32), px(height as f32)));
        self
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
    /// The image's size at zoom 1: the stage, or less for a small image.
    fit: Point<Pixels>,
    /// The pointer went down on the image or a control, not the backdrop.
    press_inside: bool,
    /// The pointer went down on the backdrop around the image.
    backdrop_press: bool,
    /// Closes the lightbox from a click on the backdrop.
    dialog: Option<DialogHandle>,
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
            fit: point(px(800.), px(600.)),
            press_inside: false,
            backdrop_press: false,
            dialog: None,
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
        // The zoomed image moves until its edge meets the stage's.
        let limit = |fit: Pixels, stage: Pixels| ((fit * self.zoom - stage) / 2.).max(px(0.));
        let (x, y) = (
            limit(self.fit.x, self.stage.x),
            limit(self.fit.y, self.stage.y),
        );
        self.pan.x = self.pan.x.clamp(-x, x);
        self.pan.y = self.pan.y.clamp(-y, y);
    }

    /// The image's size at zoom 1: its natural size scaled down to fit the
    /// stage, never up; the whole stage when the size is unknown.
    fn fitted(&self) -> Point<Pixels> {
        match self.current().and_then(|image| image.size) {
            Some(natural) => {
                let scale = (self.stage.x / natural.width)
                    .min(self.stage.y / natural.height)
                    .min(1.);
                point(natural.width * scale, natural.height * scale)
            }
            None => self.stage,
        }
    }

    /// Whether a click at the end of this press should close the viewer:
    /// it started on the backdrop around a fitted image.
    fn take_backdrop_click(&mut self) -> bool {
        std::mem::take(&mut self.backdrop_press) && self.zoom <= 1.
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
        // Panning follows the arrow on screen; previous and next follow the
        // reading direction.
        let direction = UiTheme::read(cx).direction;
        let key = keystroke.key.as_str();
        let logical = direction.logical_key(key);
        match (key, modifiers.shift) {
            ("left", true) => self.pan_by(point(pan, px(0.)), cx),
            ("right", true) => self.pan_by(point(-pan, px(0.)), cx),
            ("up", true) => self.pan_by(point(px(0.), pan), cx),
            ("down", true) => self.pan_by(point(px(0.), -pan), cx),
            ("left" | "right", false) if logical == "left" => self.previous(cx),
            ("left" | "right", false) => self.next(cx),
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
        self.fit = self.fitted();
        self.clamp_pan();
        let white = hsla(0., 0., 1., 1.);
        let count = self.images.len();
        let current = self.current().cloned();
        let zoom = self.zoom;
        let dir = theme.direction;
        let previous_label = locale::text(cx, UiText::PreviousImage);
        let next_label = locale::text(cx, UiText::NextImage);
        let icon_button = |id: ElementId, label: SharedString, icon: LucideIcon| {
            Button::new(id)
                .size(ButtonSize::Icon)
                .variant(ButtonVariant::Ghost)
                .aria_label(label)
                .child(lucide(icon).size(theme.space(4.5)).text_color(white))
        };
        let side = |button: Button| {
            div()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, _| this.press_inside = true),
                )
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
                    // The image and the controls mark their presses first.
                    this.backdrop_press = !std::mem::take(&mut this.press_inside);
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
            .on_click(
                cx.listener(|this, event: &gpui_kit::ClickEvent, window, cx| {
                    // A click on the dark area around a fitted image closes the
                    // viewer, as in every image viewer.
                    if this.take_backdrop_click() {
                        if let Some(dialog) = this.dialog.clone() {
                            dialog.close(window, cx);
                        }
                        return;
                    }
                    // Double-click toggles between fitted and 2×.
                    if event.click_count() == 2 {
                        if this.zoom > 1. {
                            this.reset_zoom(cx);
                        } else {
                            this.set_zoom(2., cx);
                        }
                    }
                }),
            )
            .when_some(current.clone(), |stage, image| {
                let width = self.fit.x * zoom;
                let height = self.fit.y * zoom;
                stage.child(
                    div()
                        .absolute()
                        .left((self.stage.x - width) / 2. + self.pan.x)
                        .top((self.stage.y - height) / 2. + self.pan.y)
                        .w(width)
                        .h(height)
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, _| this.press_inside = true),
                        )
                        .child(
                            super::theme::primitives::image(image.source)
                                .size_full()
                                .object_fit(ObjectFit::Contain),
                        ),
                )
            })
            .when(count > 1, |stage| {
                stage
                    .child(
                        side(
                            icon_button(
                                (id.clone(), "previous").into(),
                                previous_label,
                                dir.icon(false, LucideIcon::ChevronLeft, LucideIcon::ChevronRight),
                            )
                            .on_click(cx.listener(|this, _, _, cx| this.previous(cx))),
                        )
                        .inset_start(dir, theme.space(4.)),
                    )
                    .child(
                        side(
                            icon_button(
                                (id.clone(), "next").into(),
                                next_label,
                                dir.icon(true, LucideIcon::ChevronLeft, LucideIcon::ChevronRight),
                            )
                            .on_click(cx.listener(|this, _, _, cx| this.next(cx))),
                        )
                        .inset_end(dir, theme.space(4.)),
                    )
            })
    }
}

/// A full-window lightbox opened and closed with `handle`.
///
/// Keep it mounted while closed, like `dialog`. Toolbar `actions` render
/// before the close button, such as a download or open-in-browser button.
/// Escape, the close button and a click on the dark area around the image
/// close it; focus returns to where it was.
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
        .unwrap_or_else(|| locale::text(cx, UiText::Image));
    let dir = theme.direction;
    let position = locale::text(
        cx,
        UiText::Position {
            index: state.read(cx).index() + 1,
            count: state.read(cx).images().len(),
        },
    );
    let close = handle.clone();
    let escape = handle.clone();
    state.update(cx, |state, _| state.dialog = Some(handle.clone()));
    let (current, count, zoom) = {
        let state = state.read(cx);
        (state.current().cloned(), state.images().len(), state.zoom())
    };
    let white = hsla(0., 0., 1., 1.);
    let dim = hsla(0., 0., 1., 0.7);
    let icon_button = |id: ElementId, label: SharedString, icon: LucideIcon| {
        Button::new(id)
            .size(ButtonSize::Icon)
            .variant(ButtonVariant::Ghost)
            .aria_label(label)
            .child(lucide(icon).size(theme.space(4.5)).text_color(white))
    };
    let zoom_status = locale::text(cx, UiText::Zoom((zoom * 100.).round() as u32));
    let zoom_out_label = locale::text(cx, UiText::ZoomOut);
    let zoom_in_label = locale::text(cx, UiText::ZoomIn);
    let close_label = locale::text(cx, UiText::Close);
    let viewer_label = locale::text(cx, UiText::ImageViewer(name.clone()));
    let zoom_out = state.clone();
    let zoom_in = state.clone();
    let top = div()
        .flex_none()
        .h(theme.space(BAR))
        .flex_row_in(dir)
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
                    .margin_end(dir, theme.space(2.))
                    .child(position),
            )
        })
        .child(
            icon_button(
                (id.clone(), "zoom-out").into(),
                zoom_out_label,
                LucideIcon::ZoomOut,
            )
            .on_click(move |_, _, cx| zoom_out.update(cx, |state, cx| state.zoom_out(cx))),
        )
        .child(
            div()
                .id((id.clone(), "zoom"))
                .role(Role::Status)
                .aria_label(zoom_status)
                .w(theme.space(12.))
                .text_center()
                .text_size(theme.text(13.))
                .child(format!("{}%", (zoom * 100.).round() as i32)),
        )
        .child(
            icon_button(
                (id.clone(), "zoom-in").into(),
                zoom_in_label,
                LucideIcon::ZoomIn,
            )
            .on_click(move |_, _, cx| zoom_in.update(cx, |state, cx| state.zoom_in(cx))),
        )
        .children(actions)
        .child(
            icon_button((id.clone(), "close").into(), close_label, LucideIcon::X)
                .on_click(move |_, window, cx| close.close(window, cx)),
        );
    let key_state = state.clone();
    // Escape is an action in the lightbox's own key context, so an app's
    // Escape shortcut around it doesn't swallow the key.
    let popup = dismiss::on_escape(div(), move |window, cx| {
        escape.close(window, cx);
        true
    })
    .id(id.clone())
    .role(Role::Dialog)
    .aria_label(viewer_label)
    .track_focus(&focus)
    .relative()
    .size_full()
    .occlude()
    .on_key_down(move |event, _, cx| {
        if key_state.update(cx, |state, cx| state.handle_key(event, cx)) {
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

    gpui_kit::actions!(lightbox_test, [MarkRead]);

    struct App {
        handle: super::DialogHandle,
        state: gpui_kit::Entity<LightboxState>,
        marked: bool,
    }

    impl gpui_kit::Render for App {
        fn render(
            &mut self,
            window: &mut gpui_kit::Window,
            cx: &mut gpui_kit::Context<Self>,
        ) -> impl gpui_kit::IntoElement {
            use gpui_kit::{InteractiveElement as _, ParentElement as _, Styled as _};
            gpui_kit::div()
                .key_context("App")
                .size_full()
                .on_action(cx.listener(|this, _: &MarkRead, _, _| this.marked = true))
                .child(super::lightbox(
                    "viewer",
                    &self.handle,
                    &self.state,
                    [],
                    window,
                    cx,
                ))
        }
    }

    #[gpui_kit::test]
    fn escape_closes_it_inside_an_app_escape_shortcut(cx: &mut TestAppContext) {
        cx.update(|cx| {
            super::super::theme::init(cx);
            // An app shortcut on Escape around the viewer, like "mark read".
            cx.bind_keys([gpui_kit::KeyBinding::new("escape", MarkRead, Some("App"))]);
        });
        let (app, cx) = cx.add_window_view(|_, cx| App {
            handle: super::DialogHandle::new(false),
            state: cx.new(|_| LightboxState::new([LightboxImage::new("a.png", "a.png")])),
            marked: false,
        });
        cx.update(|window, cx| {
            window.activate_window();
            window.draw(cx).clear(cx);
            let handle = app.read(cx).handle.clone();
            handle.open(window, cx);
        });
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
        assert!(app.read_with(cx, |app, _| app.handle.is_open()));
        cx.simulate_keystrokes("escape");
        cx.update(|window, cx| window.draw(cx).clear(cx));
        app.read_with(cx, |app, _| {
            assert!(!app.handle.is_open(), "Escape closes the viewer");
            assert!(!app.marked, "the app's shortcut doesn't run");
        });
    }

    #[gpui_kit::test]
    fn small_images_keep_their_natural_size(cx: &mut TestAppContext) {
        let state = cx.new(|_| {
            LightboxState::new([
                LightboxImage::new("small.png", "small.png").dimensions(400, 200),
                LightboxImage::new("large.png", "large.png").dimensions(3200, 1200),
                LightboxImage::new("unknown.png", "unknown.png"),
            ])
        });
        state.update(cx, |state, cx| {
            // The default stage is 800×600.
            assert_eq!(state.fitted(), point(px(400.), px(200.)));
            state.next(cx);
            assert_eq!(state.fitted(), point(px(800.), px(300.)));
            state.next(cx);
            assert_eq!(state.fitted(), point(px(800.), px(600.)));
        });
    }

    #[gpui_kit::test]
    fn only_a_press_on_the_backdrop_of_a_fitted_image_closes(cx: &mut TestAppContext) {
        let state = cx.new(|_| LightboxState::new([LightboxImage::new("a.png", "a.png")]));
        state.update(cx, |state, cx| {
            state.backdrop_press = true;
            assert!(state.take_backdrop_click());
            // A click without a new backdrop press doesn't close.
            assert!(!state.take_backdrop_click());
            state.set_zoom(2., cx);
            state.backdrop_press = true;
            assert!(!state.take_backdrop_click());
        });
    }
}
