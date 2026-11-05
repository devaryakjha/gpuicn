#[cfg(not(target_family = "wasm"))]
mod gallery;
mod previews;
#[cfg(not(target_family = "wasm"))]
mod workspace;

pub(crate) use previews::Demo;
use previews::prelude::*;

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(
    inline_js = "export function previewReady(){let sent=false;const send=()=>{if(sent)return;sent=true;document.documentElement.dataset.gpuicnReady='true';window.parent.postMessage({gpuicn:'preview-ready'},'*')};requestAnimationFrame(()=>requestAnimationFrame(send));setTimeout(send,1000)} export function installPreviewUpdates(update){window.gpuicnPreviewUpdates(update)}"
)]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = previewReady)]
    fn preview_ready();
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = installPreviewUpdates)]
    fn install_preview_updates(update: wasm_bindgen::JsValue);
}

fn main() {
    #[cfg(target_family = "wasm")]
    {
        gpui_kit::platform::web_init();
        let handle = application().run_embedded(launch);
        std::mem::forget(handle);
    }

    #[cfg(not(target_family = "wasm"))]
    application().run(launch);
}

fn application() -> Application {
    // gpui-pre 0.3.11 no longer installs a Fetch client on the web, so image
    // URLs need one passed in.
    #[cfg(target_family = "wasm")]
    let app = {
        // Auto tries WebGPU and falls back to WebGL when the browser has no
        // WebGPU adapter. `?backend=webgl` skips WebGPU for screenshots.
        let backend = match requested_value("backend").as_deref() {
            Some("webgl") => gpui_kit::platform::WebBackendPreference::WebGl,
            _ => gpui_kit::platform::WebBackendPreference::Auto,
        };
        let platform = gpui_kit::web::WebPlatform::new_with_backend(true, backend);
        let http_client = std::sync::Arc::new(platform.fetch_http_client());
        Application::with_platform(std::rc::Rc::new(platform)).with_http_client(http_client)
    };
    #[cfg(not(target_family = "wasm"))]
    let app = gpui_kit::application();

    app.with_assets(LucideAssetSource)
}

fn launch(cx: &mut App) {
    cx.text_system()
        .add_fonts(vec![
            Cow::Borrowed(include_bytes!("../assets/fonts/Geist-Regular.ttf")),
            Cow::Borrowed(include_bytes!("../assets/fonts/Geist-Medium.ttf")),
            Cow::Borrowed(include_bytes!("../assets/fonts/Geist-SemiBold.ttf")),
            Cow::Borrowed(include_bytes!("../assets/fonts/GeistMono-Regular.ttf")),
        ])
        .expect("failed to load pinned Geist fonts");
    gpuicn::init(cx);

    let demo = requested_value("demo")
        .and_then(|value| Demo::parse(&value))
        .unwrap_or_default();
    let mode = match requested_value("theme").as_deref() {
        Some("dark") => ThemeMode::Dark,
        _ => ThemeMode::Light,
    };
    UiTheme::switch(cx, mode);
    // `?dir=rtl` (or `GPUICN_DIR=rtl` natively) previews the gallery in a
    // right-to-left layout.
    if requested_value("dir").as_deref() == Some("rtl") {
        gpuicn::locale::set_direction(cx, gpuicn::Direction::Rtl);
    }
    #[cfg(not(target_family = "wasm"))]
    if requested_value("demo")
        .as_deref()
        .is_none_or(|value| value == "--workspace")
    {
        workspace::launch(cx);
        return;
    }
    #[cfg(not(target_family = "wasm"))]
    if requested_value("demo").as_deref() == Some("--catalog") {
        gallery::launch(cx);
        return;
    }

    let bounds = Bounds::centered(
        None,
        size(
            px(requested_dimension(
                "width",
                if demo.is_wide() { 960. } else { 640. },
                320.0,
                1440.0,
            )),
            px(requested_dimension(
                "height",
                if demo.is_wide() { 600. } else { 288. },
                240.0,
                720.0,
            )),
        ),
        cx,
    );
    let _window = cx
        .open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |_window, cx| cx.new(move |_| Showcase::new(demo)),
        )
        .expect("failed to open showcase window");
    #[cfg(target_family = "wasm")]
    {
        let mut async_cx = cx.to_async();
        let update = wasm_bindgen::closure::Closure::<dyn FnMut(String, String)>::new(
            move |theme: String, icon: String| {
                let mode = match theme.as_str() {
                    "light" => ThemeMode::Light,
                    "dark" => ThemeMode::Dark,
                    _ => return,
                };
                _window
                    .update(&mut async_cx, |view, _, cx| {
                        UiTheme::switch(cx, mode);
                        if let Some(icon) = LucideIcon::from_name(&icon) {
                            view.icon = icon;
                        }
                        cx.notify();
                    })
                    .ok();
            },
        );
        install_preview_updates(update.into_js_value());
        preview_ready();
    }
    #[cfg(target_family = "wasm")]
    cx.activate(true);
    #[cfg(not(target_family = "wasm"))]
    if std::env::var_os("IMAJHA_SHOWCASE_BACKGROUND").is_none() {
        cx.activate(true);
    }
}

fn virtual_list_state(count: usize) -> gpuicn::virtual_list::VirtualListState {
    use gpuicn::virtual_list::{ListItem, ListSelectionMode, VirtualListState};
    let state = VirtualListState::new(
        (0..count)
            .map(|index| {
                ListItem::new(("file", index), format!("component_{index:05}.rs"))
                    .disabled(index == 7)
            })
            .collect(),
    )
    .expect("fixture IDs are unique");
    state.set_selection_mode(ListSelectionMode::Multiple);
    state
}

struct Showcase {
    demo: Demo,
    count: usize,
    demo_action: String,
    checked: bool,
    pressed: bool,

    collapsible_open: bool,

    drawer_direction: DrawerSide,
    goal: i32,

    pane_width: gpui_kit::Pixels,
    sidebar_state: gpuicn::sidebar::SidebarState,
    sidebar_example: String,
    sidebar_mobile: bool,
    sidebar_selected: usize,
    sidebar_workspace: usize,
    sidebar_search: String,
    sidebar_nested: bool,
    sidebar_projects: usize,
    sidebar_note: String,
    sidebar_loaded: bool,
    sidebar_message: usize,
    sidebar_unread: bool,
    sidebar_sections: [bool; 3],
    virtual_rows: Option<gpuicn::virtual_list::VirtualListState>,
    virtual_details: bool,
    virtual_reversed: bool,
    virtual_note: String,
    icon: LucideIcon,
}

impl Render for Showcase {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        // These examples live in a documentation pane, so use its 540px breakpoint.
        self.sidebar_mobile = _window.viewport_size().width < px(540.);
        div()
            .size_full()
            .overflow_hidden()
            .flex()
            .items_center()
            .justify_center()
            .p(px(if self.demo.is_edge_to_edge() { 0. } else { 16. }))
            .bg(theme.colors.background)
            .text_color(theme.colors.foreground)
            .font_family(theme.fonts.body)
            .when(!self.demo_action.is_empty(), |view| {
                view.flex_col().gap(px(16.))
            })
            .child(self.preview(_window, cx))
            .when(!self.demo_action.is_empty(), |view| {
                view.child(
                    div()
                        .text_size(px(13.))
                        .text_color(theme.colors.muted_foreground)
                        .child(format!("Last action: {}", self.demo_action)),
                )
            })
    }
}

impl Showcase {
    fn new(demo: Demo) -> Self {
        Self {
            demo,
            count: 0,
            demo_action: String::new(),
            checked: false,
            pressed: false,

            collapsible_open: true,

            drawer_direction: DrawerSide::Bottom,
            goal: 350,

            pane_width: px(160.),
            sidebar_state: Default::default(),
            sidebar_example: requested_value("example").unwrap_or_else(|| "workspace".into()),
            sidebar_mobile: false,
            sidebar_selected: 0,
            sidebar_workspace: 0,
            sidebar_search: String::new(),
            sidebar_nested: true,
            sidebar_projects: 2,
            sidebar_note: String::new(),
            sidebar_loaded: false,
            sidebar_message: 0,
            sidebar_unread: false,
            sidebar_sections: [true, true, false],
            virtual_rows: matches!(demo, Demo::VirtualList).then(|| virtual_list_state(100_000)),
            virtual_details: false,
            virtual_reversed: false,
            virtual_note: String::new(),
            icon: requested_value("icon")
                .as_deref()
                .and_then(LucideIcon::from_name)
                .unwrap_or(LucideIcon::House),
        }
    }

    fn icons_preview(&self, cx: &App) -> impl IntoElement {
        div()
            .flex()
            .items_end()
            .justify_center()
            .gap(px(20.))
            .children([16., 24., 32., 48.].into_iter().map(|size| {
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(16.))
                    .child(
                        lucide(self.icon)
                            .size(px(size))
                            .text_color(UiTheme::read(cx).colors.foreground),
                    )
                    .child(div().text_size(px(11.)).child(format!("{size:.0}")))
            }))
    }
}

fn switch_row(switch: Switch, label: impl IntoElement) -> gpui_kit::Div {
    div()
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(switch)
        .child(label)
}

fn checkbox_row(checkbox: Checkbox, label: &'static str) -> gpui_kit::Div {
    div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .child(checkbox)
        .child(label)
}

#[cfg(target_family = "wasm")]
fn requested_value(key: &str) -> Option<String> {
    let search = web_sys::window()?.location().search().ok()?;
    search
        .trim_start_matches('?')
        .split('&')
        .find_map(|pair| pair.split_once('=').filter(|(name, _)| *name == key))
        .map(|(_, value)| value.to_owned())
}

#[cfg(not(target_family = "wasm"))]
fn requested_value(key: &str) -> Option<String> {
    match key {
        "demo" => std::env::args().nth(1),
        "theme" => std::env::args().nth(2),
        "example" => std::env::args().nth(3),
        "width" => std::env::args().nth(4),
        "height" => std::env::args().nth(5),
        "dir" => std::env::var("GPUICN_DIR").ok(),
        _ => None,
    }
}

fn requested_dimension(key: &str, default: f32, min: f32, max: f32) -> f32 {
    parse_dimension(requested_value(key).as_deref(), default, min, max)
}

fn parse_dimension(value: Option<&str>, default: f32, min: f32, max: f32) -> f32 {
    value
        .and_then(|value| value.parse::<f32>().ok())
        .filter(|value| value.is_finite())
        .map(|value| value.clamp(min, max))
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::parse_dimension;

    #[test]
    fn preview_dimensions_stay_inside_safe_bounds() {
        assert_eq!(parse_dimension(Some("42"), 640.0, 320.0, 1440.0), 320.0);
        assert_eq!(parse_dimension(Some("1920"), 640.0, 320.0, 1440.0), 1440.0);
        assert_eq!(parse_dimension(Some("nope"), 640.0, 320.0, 1440.0), 640.0);
    }
}

#[cfg(test)]
mod audit_tests {
    use super::*;
    use gpui_kit::TestAppContext;

    fn showcase(demo: Demo, active: bool) -> Showcase {
        Showcase {
            demo_action: String::new(),
            demo,
            count: 0,
            checked: active,
            pressed: active,

            collapsible_open: active,

            drawer_direction: DrawerSide::Bottom,
            goal: 350,

            pane_width: px(160.),
            sidebar_state: Default::default(),
            sidebar_example: "workspace".into(),
            sidebar_mobile: false,
            sidebar_selected: 0,
            sidebar_workspace: 0,
            sidebar_search: String::new(),
            sidebar_nested: true,
            sidebar_projects: 2,
            sidebar_note: String::new(),
            sidebar_loaded: false,
            sidebar_message: 0,
            sidebar_unread: false,
            sidebar_sections: [true, true, false],
            virtual_rows: matches!(demo, Demo::VirtualList).then(|| virtual_list_state(100_000)),
            virtual_details: active,
            virtual_reversed: false,
            virtual_note: String::new(),
            icon: LucideIcon::House,
        }
    }

    #[test]
    fn list_context_action_preserves_range_selection() {
        use gpui_kit::{Modifiers, MouseButton, VisualTestContext, point};
        // Declared first so it is dropped after `cx`: GPUI Kit 0.8 deadlocks when
        // an app is dropped after its test scheduler.
        let dispatcher = gpui_kit::TestDispatcher::new(0);
        let mut cx = TestAppContext::build(dispatcher.clone(), None);
        cx.update(|cx| {
            gpuicn::init(cx);
            UiTheme::set(cx, UiTheme::neutral_light());
        });
        let window = cx.add_window(|_, _| showcase(Demo::VirtualList, false));
        let mut visual = VisualTestContext::from_window(window.into(), &cx);
        visual.simulate_resize(size(px(960.), px(600.)));
        let draw = |cx: &mut TestAppContext| {
            cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
                .unwrap();
        };
        draw(&mut cx);
        visual.simulate_click(point(px(150.), px(130.)), Modifiers::default());
        visual.simulate_keystrokes("shift-down");
        draw(&mut cx);
        let state = cx
            .read_window(&window, |view, cx| {
                view.read(cx).virtual_rows.clone().unwrap()
            })
            .unwrap();
        assert_eq!(state.selected_count(), 2);
        visual.simulate_mouse_down(
            point(px(150.), px(130.)),
            MouseButton::Right,
            Modifiers::default(),
        );
        visual.simulate_mouse_up(
            point(px(150.), px(130.)),
            MouseButton::Right,
            Modifiers::default(),
        );
        draw(&mut cx);
        assert_eq!(state.selected_count(), 2);
        visual.simulate_click(point(px(180.), px(145.)), Modifiers::default());
        draw(&mut cx);
        assert_eq!(state.selected_count(), 2);
        assert_eq!(
            cx.read_window(&window, |view, cx| view.read(cx).virtual_note.clone())
                .unwrap(),
            "Inspect component_00000.rs · 2 selected"
        );
        visual.simulate_keystrokes("down");
        assert_eq!(state.focused(), Some(("file", 1usize).into()));
        assert_eq!(state.selected_count(), 1);
    }

    #[test]
    fn sidebar_search_updates_without_reborrowing_the_app() {
        use gpui_kit::{Modifiers, VisualTestContext, point};
        for (example, point) in [
            ("mail", point(px(110.), px(66.))),
            ("docs", point(px(90.), px(70.))),
        ] {
            // Declared first so it is dropped after `cx`: GPUI Kit 0.8 deadlocks when
            // an app is dropped after its test scheduler.
            let dispatcher = gpui_kit::TestDispatcher::new(0);
            let mut cx = TestAppContext::build(dispatcher.clone(), None);
            cx.update(|cx| {
                gpuicn::init(cx);
                UiTheme::set(cx, UiTheme::neutral_light());
            });
            let window = cx.add_window(|_, _| {
                let mut view = showcase(Demo::Sidebar, false);
                view.sidebar_example = example.into();
                view
            });
            let mut visual = VisualTestContext::from_window(window.into(), &cx);
            visual.simulate_resize(size(px(960.), px(600.)));
            for _ in 0..2 {
                cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
                    .unwrap();
            }
            visual.simulate_click(point, Modifiers::default());
            for character in ["m", "a", "i", "l"] {
                visual.simulate_input(character);
                cx.run_until_parked();
                cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
                    .unwrap();
            }
            assert_eq!(
                cx.read_window(&window, |view, cx| view.read(cx).sidebar_search.clone())
                    .unwrap(),
                "mail",
                "{example} search"
            );
        }
    }

    #[test]
    fn switch_control_and_label_each_toggle_once() {
        use gpui_kit::{Modifiers, VisualTestContext, point};
        // Declared first so it is dropped after `cx`: GPUI Kit 0.8 deadlocks when
        // an app is dropped after its test scheduler.
        let dispatcher = gpui_kit::TestDispatcher::new(0);
        let mut cx = TestAppContext::build(dispatcher.clone(), None);
        cx.update(|cx| {
            gpuicn::init(cx);
            UiTheme::set(cx, UiTheme::neutral_light());
        });
        let window = cx.add_window(|_, _| showcase(Demo::Switch, false));
        let mut visual = VisualTestContext::from_window(window.into(), &cx);
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let label = visual.debug_bounds("switch-label").unwrap();
        visual.simulate_click(label.center(), Modifiers::default());
        assert!(
            cx.read_window(&window, |view, cx| view.read(cx).checked)
                .unwrap()
        );
        let label = visual.debug_bounds("switch-label").unwrap();
        visual.simulate_click(
            point(label.left() - px(26.), label.center().y),
            Modifiers::default(),
        );
        assert!(
            !cx.read_window(&window, |view, cx| view.read(cx).checked)
                .unwrap()
        );
    }

    #[test]
    fn sidebar_examples_render_across_viewports_and_themes() {
        use gpui_kit::VisualTestContext;
        let mut custom = UiTheme::neutral_dark();
        custom.spacing.unit = px(5.);
        custom.text_scale = 1.25;
        custom.radius = gpuicn::theme::UiRadius::new(px(4.));
        for theme in [UiTheme::neutral_light(), UiTheme::neutral_dark(), custom] {
            for width in [360., 960.] {
                for example in ["workspace", "docs", "mail", "floating", "mobile", "loading"] {
                    // Declared first so it is dropped after `cx`: GPUI Kit 0.8 deadlocks when
                    // an app is dropped after its test scheduler.
                    let dispatcher = gpui_kit::TestDispatcher::new(0);
                    let mut cx = TestAppContext::build(dispatcher.clone(), None);
                    cx.update(|cx| {
                        gpuicn::init(cx);
                        UiTheme::set(cx, theme.clone());
                    });
                    let window = cx.add_window(|_, _| {
                        let mut view = showcase(Demo::Sidebar, false);
                        view.sidebar_example = example.into();
                        view
                    });
                    VisualTestContext::from_window(window.into(), &cx)
                        .simulate_resize(size(px(width), px(600.)));
                    for open in [false, true] {
                        window
                            .update(&mut cx, |view, _, cx| {
                                view.sidebar_state.open = open;
                                view.sidebar_state.mobile_open = open;
                                cx.notify();
                            })
                            .unwrap();
                        for _ in 0..2 {
                            cx.update_window(window.into(), |_, window, cx| {
                                window.draw(cx).clear(cx)
                            })
                            .unwrap();
                            cx.run_until_parked();
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn every_preview_renders_in_both_themes_and_controlled_states() {
        let demos = [
            "accordion",
            "alert-dialog",
            "autocomplete",
            "avatar",
            "button",
            "checkbox",
            "checkbox-group",
            "collapsible",
            "combobox",
            "context-menu",
            "dialog",
            "drawer",
            "field",
            "fieldset",
            "form",
            "input",
            "menu",
            "menubar",
            "meter",
            "navigation-menu",
            "number-field",
            "otp-field",
            "popover",
            "preview-card",
            "progress",
            "radio-group",
            "scroll-area",
            "select",
            "separator",
            "sidebar",
            "resizable",
            "virtual-list",
            "slider",
            "switch",
            "tabs",
            "toast",
            "toggle",
            "toggle-group",
            "toolbar",
            "tooltip",
            "kbd",
            "badge",
            "tag",
            "spinner",
            "skeleton",
            "markdown",
            "command",
            "composer",
            "emoji-picker",
            "lightbox",
            "message",
            "message-list",
            "attachment",
            "title-bar",
        ];
        let mut custom = UiTheme::neutral_light();
        custom.spacing.unit = px(5.);
        custom.radius = gpuicn::theme::UiRadius::new(px(4.));
        custom.text_scale = 1.25;
        custom.colors.primary = gpui_kit::rgb(0x2563eb);
        for theme in [UiTheme::neutral_light(), UiTheme::neutral_dark(), custom] {
            for active in [false, true] {
                for name in demos {
                    // Declared first so it is dropped after `cx`: GPUI Kit 0.8 deadlocks when
                    // an app is dropped after its test scheduler.
                    let dispatcher = gpui_kit::TestDispatcher::new(0);
                    let mut cx = TestAppContext::build(dispatcher.clone(), None);
                    cx.update(|cx| {
                        gpuicn::init(cx);
                        UiTheme::set(cx, theme.clone());
                    });
                    let window =
                        cx.add_window(move |_, _| showcase(Demo::parse(name).unwrap(), active));
                    for _ in 0..2 {
                        cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
                            .unwrap();
                        cx.run_until_parked();
                    }
                }
            }
        }
    }
}

fn demo_input(
    id: &'static str,
    value: &'static str,
    placeholder: &'static str,
    window: &mut Window,
    cx: &mut App,
) -> Entity<InputState> {
    window.use_keyed_state(id, cx, |window, cx| {
        InputState::new(window, cx)
            .default_value(value)
            .placeholder(placeholder)
    })
}

fn live_input<T: 'static>(
    id: impl Into<gpui_kit::ElementId>,
    value: impl Into<gpui_kit::SharedString>,
    placeholder: &'static str,
    window: &mut Window,
    cx: &mut Context<T>,
    on_event: impl Fn(&mut T, &Entity<InputState>, &InputEvent, &mut Context<T>) + 'static,
) -> Entity<InputState> {
    let owner = cx.entity().downgrade();
    let value = value.into();
    window
        .use_keyed_state(id, cx, move |window, cx| {
            let input = cx.new(|cx| {
                InputState::new(window, cx)
                    .default_value(value)
                    .placeholder(placeholder)
            });
            let subscription = cx.subscribe(&input, move |_, input, event, cx| {
                let _ = owner.update(cx, |this, cx| on_event(this, &input, event, cx));
            });
            (input, subscription)
        })
        .read(cx)
        .0
        .clone()
}
