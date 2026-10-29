//! Nova custom window title bar with platform window controls.
//!
//! Behavior follows GPUI Kit 0.8.1's `TitleBar`: the bar drags the window,
//! double-clicks zoom it, macOS keeps its traffic lights, and Windows and
//! client-decorated Linux windows get minimize, maximize and close controls.

use std::rc::Rc;

use gpui_icons::{LucideIcon, lucide};
use gpui_kit::base::InteractiveElementExt as _;
use gpui_kit::{
    AnyElement, App, ClickEvent, Decorations, InteractiveElement as _, IntoElement, MouseButton,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled,
    TitlebarOptions, Window, WindowControlArea, WindowOptions, div, prelude::FluentBuilder as _,
    px,
};

use super::theme::{UiTheme, apply_style};

type CloseHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// Title bar height, matching GPUI Kit so the traffic lights stay centered.
pub const TITLE_BAR_HEIGHT: gpui_kit::Pixels = px(34.);

/// Window options for a window that draws a `TitleBar`. Spread your own options
/// over these: `WindowOptions { ..title_bar_window_options() }`.
pub fn title_bar_window_options() -> WindowOptions {
    gpui_kit::component::TitleBar::window_options()
}

/// macOS titlebar options that place the traffic lights inside a `TitleBar`.
pub fn title_bar_options() -> TitlebarOptions {
    gpui_kit::component::TitleBar::title_bar_options()
}

/// A draggable title bar that hosts application content, such as a search
/// field or workspace switcher, beside the platform window controls.
#[derive(IntoElement)]
pub struct TitleBar {
    style: gpui_kit::StyleRefinement,
    children: Vec<AnyElement>,
    on_close: Option<CloseHandler>,
}

impl TitleBar {
    /// Creates an empty title bar.
    pub fn new() -> Self {
        Self {
            style: gpui_kit::StyleRefinement::default(),
            children: Vec::new(),
            on_close: None,
        }
    }

    /// Runs instead of closing the window when the close control is clicked on
    /// Linux, for example to hide to the tray. Windows and macOS close natively.
    pub fn on_close(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_close = Some(Rc::new(handler));
        self
    }
}

impl Default for TitleBar {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for TitleBar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Styled for TitleBar {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Control {
    Minimize,
    Maximize,
    Restore,
    Close,
}

impl Control {
    fn icon(self) -> LucideIcon {
        match self {
            Self::Minimize => LucideIcon::Minus,
            Self::Maximize => LucideIcon::Square,
            Self::Restore => LucideIcon::Copy,
            Self::Close => LucideIcon::X,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Minimize => "Minimize",
            Self::Maximize => "Maximize",
            Self::Restore => "Restore",
            Self::Close => "Close",
        }
    }
    fn area(self) -> WindowControlArea {
        match self {
            Self::Minimize => WindowControlArea::Min,
            Self::Maximize | Self::Restore => WindowControlArea::Max,
            Self::Close => WindowControlArea::Close,
        }
    }
}

fn control(kind: Control, on_close: Option<CloseHandler>, theme: &UiTheme) -> impl IntoElement {
    let colors = theme.colors;
    let (hover_bg, hover_fg) = if kind == Control::Close {
        (colors.destructive, gpui_kit::white().into())
    } else {
        (colors.accent, colors.accent_foreground)
    };
    div()
        .id(SharedString::from(kind.label()))
        .aria_label(kind.label())
        .flex()
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .w(px(46.))
        .h_full()
        .text_color(colors.muted_foreground)
        .hover(move |style| style.bg(hover_bg).text_color(hover_fg))
        .when(cfg!(target_os = "windows"), |this| {
            this.window_control_area(kind.area())
        })
        .when(cfg!(target_os = "linux"), |this| {
            this.on_mouse_down(MouseButton::Left, |_, window, cx| {
                window.prevent_default();
                cx.stop_propagation();
            })
            .on_click(move |event, window, cx| {
                cx.stop_propagation();
                match kind {
                    Control::Minimize => window.minimize_window(),
                    Control::Maximize | Control::Restore => window.zoom_window(),
                    Control::Close => match &on_close {
                        Some(handler) => handler(event, window, cx),
                        None => window.remove_window(),
                    },
                }
            })
        })
        .child(
            lucide(kind.icon())
                .size(if kind == Control::Close {
                    theme.space(4.)
                } else {
                    theme.space(3.5)
                })
                .text_color(colors.muted_foreground),
        )
}

struct DragState {
    should_move: bool,
}

impl RenderOnce for TitleBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let client_decorated = matches!(window.window_decorations(), Decorations::Client { .. });
        let is_web = cfg!(target_family = "wasm");
        let is_macos = cfg!(target_os = "macos");
        let is_linux = cfg!(target_os = "linux");
        // Under server-side decorations the window manager draws its own controls.
        let draws_controls = !is_macos && !is_web && (!is_linux || client_decorated);
        let state = window.use_state(cx, |_, _| DragState { should_move: false });
        let supported = window.window_controls();
        let maximized = window.is_maximized();
        let left_inset = if is_macos && !window.is_fullscreen() {
            px(80.)
        } else {
            theme.space(3.)
        };

        let bar = div()
            .id("title-bar")
            .flex_shrink_0()
            .w_full()
            .h(TITLE_BAR_HEIGHT)
            .flex()
            .items_center()
            .pl(left_inset)
            .border_b_1()
            .border_color(theme.colors.border)
            .bg(theme.colors.sidebar)
            .text_color(theme.colors.foreground)
            .font_family(theme.fonts.body.clone())
            .text_size(theme.text(13.))
            .when(is_linux, |bar| {
                bar.on_double_click(|_, window, _| window.zoom_window())
            })
            .when(is_macos, |bar| {
                bar.on_double_click(|_, window, _| window.titlebar_double_click())
            })
            .on_mouse_down_out(window.listener_for(&state, |state, _, _, _| {
                state.should_move = false;
            }))
            .on_mouse_down(
                MouseButton::Left,
                window.listener_for(&state, |state, _, _, _| state.should_move = true),
            )
            .on_mouse_up(
                MouseButton::Left,
                window.listener_for(&state, |state, _, _, _| state.should_move = false),
            )
            .on_mouse_move(window.listener_for(&state, |state, _, window, _| {
                if state.should_move {
                    state.should_move = false;
                    window.start_window_move();
                }
            }))
            .child(
                div()
                    .id("title-bar-content")
                    .h_full()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .items_center()
                    .gap(theme.space(2.))
                    .when(!is_web, |content| {
                        content.window_control_area(WindowControlArea::Drag)
                    })
                    .when(is_linux && client_decorated, |content| {
                        content.on_mouse_down(MouseButton::Right, |event, window, _| {
                            window.show_window_menu(event.position)
                        })
                    })
                    .children(self.children),
            )
            .when(draws_controls, |bar| {
                bar.child(
                    div()
                        .flex()
                        .h_full()
                        .flex_shrink_0()
                        .when(supported.minimize, |row| {
                            row.child(control(Control::Minimize, None, &theme))
                        })
                        .when(supported.maximize, |row| {
                            row.child(control(
                                if maximized {
                                    Control::Restore
                                } else {
                                    Control::Maximize
                                },
                                None,
                                &theme,
                            ))
                        })
                        .child(control(Control::Close, self.on_close.clone(), &theme)),
                )
            });
        apply_style(bar, &self.style)
    }
}
