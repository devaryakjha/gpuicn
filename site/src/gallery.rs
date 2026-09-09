use super::*;
use gpui_kit::{Entity, Pixels};
use gpuicn::sidebar::{Sidebar, SidebarItem};

gpui_kit::actions!(gallery, [Quit]);

// Matches the site catalog; both surfaces render Showcase examples.
const COMPONENTS: &[(&str, &str, &str)] = &[
    (
        "accordion",
        "Accordion",
        "A vertically stacked set of interactive headings that reveal content.",
    ),
    (
        "alert-dialog",
        "Alert Dialog",
        "A modal dialog that expects a response from the user.",
    ),
    (
        "autocomplete",
        "Autocomplete",
        "A text input that suggests matching values.",
    ),
    (
        "avatar",
        "Avatar",
        "An image element with a fallback for representing a user.",
    ),
    (
        "button",
        "Button",
        "Displays a button or a component that looks like a button.",
    ),
    (
        "checkbox",
        "Checkbox",
        "A control that toggles between checked and unchecked.",
    ),
    (
        "checkbox-group",
        "Checkbox Group",
        "Groups related checkboxes under shared state.",
    ),
    (
        "collapsible",
        "Collapsible",
        "An interactive panel that expands and collapses.",
    ),
    (
        "combobox",
        "Combobox",
        "An input paired with a searchable list of choices.",
    ),
    (
        "context-menu",
        "Context Menu",
        "A menu opened by a secondary pointer action.",
    ),
    (
        "dialog",
        "Dialog",
        "A window overlaid on the primary content.",
    ),
    (
        "drawer",
        "Drawer",
        "A panel that slides from an edge of the window.",
    ),
    (
        "menu",
        "Dropdown Menu",
        "Displays a menu of actions from a trigger.",
    ),
    (
        "field",
        "Field",
        "Composes a control with its label, description, and errors.",
    ),
    (
        "fieldset",
        "Fieldset",
        "Groups related form controls with a legend.",
    ),
    (
        "form",
        "Form",
        "Coordinates validation and submission for fields.",
    ),
    (
        "preview-card",
        "Hover Card",
        "Previews content behind a link or trigger.",
    ),
    (
        "input",
        "Input",
        "A styled text input backed by native GPUI editing.",
    ),
    (
        "otp-field",
        "Input OTP",
        "A segmented input for one-time passcodes.",
    ),
    (
        "menubar",
        "Menubar",
        "A persistent horizontal set of menus.",
    ),
    (
        "meter",
        "Meter",
        "Shows a scalar value within a known range.",
    ),
    (
        "navigation-menu",
        "Navigation Menu",
        "A collection of links for navigating a site or app.",
    ),
    (
        "number-field",
        "Number Field",
        "A numeric input with step controls.",
    ),
    (
        "popover",
        "Popover",
        "Displays rich content in a portal from a trigger.",
    ),
    (
        "progress",
        "Progress",
        "Displays completion progress for a task.",
    ),
    (
        "radio-group",
        "Radio Group",
        "A set of checkable options where one value is selected.",
    ),
    (
        "resizable",
        "Resizable",
        "Controlled panes with pointer and keyboard resizing.",
    ),
    (
        "scroll-area",
        "Scroll Area",
        "Adds styled scrolling to overflowing content.",
    ),
    (
        "select",
        "Select",
        "Displays a list of options for the user to pick from.",
    ),
    ("separator", "Separator", "Visually separates content."),
    (
        "sidebar",
        "Sidebar",
        "Application navigation with responsive layouts, icon rails, nested menus, and mobile sheets.",
    ),
    (
        "slider",
        "Slider",
        "Selects a value or range along a track.",
    ),
    ("switch", "Switch", "Toggles a setting on or off."),
    (
        "tabs",
        "Tabs",
        "Shows one panel from a set of layered sections.",
    ),
    (
        "toast",
        "Toast",
        "A brief notification shown above the application.",
    ),
    (
        "toggle",
        "Toggle",
        "A two-state button that can be on or off.",
    ),
    (
        "toggle-group",
        "Toggle Group",
        "Groups two-state buttons with shared selection.",
    ),
    (
        "toolbar",
        "Toolbar",
        "Groups controls for a related set of actions.",
    ),
    (
        "tooltip",
        "Tooltip",
        "Shows short information on hover or keyboard focus.",
    ),
    (
        "virtual-list",
        "Virtual List",
        "Large lists with stable selection, keyboard navigation, and fast fixed-height rows.",
    ),
    (
        "icons",
        "Icons",
        "Browse the Lucide icons used throughout the components.",
    ),
];

pub(super) fn launch(cx: &mut App) {
    UiTheme::set(
        cx,
        super::workspace::workspace_theme(UiTheme::read(cx).mode),
    );
    cx.on_window_closed(|cx, _| {
        if cx.windows().is_empty() {
            cx.quit();
        }
    })
    .detach();
    cx.on_action(|_: &Quit, cx| cx.quit());
    cx.bind_keys([gpui_kit::KeyBinding::new("cmd-q", Quit, None)]);
    cx.set_menus([gpui_kit::Menu::new("gpuicn Showcase")
        .items([gpui_kit::MenuItem::action("Quit gpuicn Showcase", Quit)])]);
    open(cx);
}

pub(super) fn open(cx: &mut App) {
    let bounds = Bounds::centered(None, size(px(1180.), px(780.)), cx);
    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            window_min_size: Some(size(px(800.), px(600.))),
            ..Default::default()
        },
        |window, cx| {
            window.set_window_title("gpuicn Showcase");
            cx.new(Gallery::new)
        },
    )
    .expect("failed to open gallery");
    cx.activate(true);
}

struct Gallery {
    selected: usize,
    query: String,
    preview: Entity<Showcase>,
    sidebar_width: Pixels,
}

impl Gallery {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            selected: 4,
            query: String::new(),
            preview: cx.new(|_| Showcase::new(Demo::Button)),
            sidebar_width: px(240.),
        }
    }

    fn select(&mut self, index: usize, cx: &mut Context<Self>) {
        self.selected = index;
        self.preview = cx.new(|_| Showcase::new(Demo::parse(COMPONENTS[index].0).unwrap()));
        cx.notify();
    }
}

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let search = live_input(
            "gallery.search",
            self.query.clone(),
            "Find a component…",
            window,
            cx,
            |this, input, event, cx| {
                if matches!(event, InputEvent::Change) {
                    this.query = input.read(cx).value().to_string();
                    cx.notify();
                }
            },
        );
        let theme = UiTheme::read(cx).clone();
        let (_, name, description) = COMPONENTS[self.selected];
        let dark = theme.mode == ThemeMode::Dark;
        let query = self.query.trim().to_lowercase();
        let mut sidebar = Sidebar::new("gallery.components", "Components")
            .header(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(16.))
                    .p(px(8.))
                    .child(
                        div()
                            .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                            .child("gpuicn"),
                    )
                    .child(Input::new(&search).aria_label("Find a component")),
            )
            .footer(
                div()
                    .px(px(8.))
                    .py(px(4.))
                    .text_size(px(12.))
                    .text_color(theme.colors.muted_foreground)
                    .child(format!("{} components · Native GPUI", COMPONENTS.len() - 1)),
            );
        let mut matches = 0;
        for (index, (_, label, description)) in COMPONENTS.iter().enumerate() {
            if !label.to_lowercase().contains(&query)
                && !description.to_lowercase().contains(&query)
            {
                continue;
            }
            matches += 1;
            sidebar = sidebar.child(
                SidebarItem::new(("gallery.item", index), *label)
                    .selected(index == self.selected)
                    .on_activate(cx.listener(move |this, _, _, cx| this.select(index, cx))),
            );
        }
        if matches == 0 {
            sidebar = sidebar.child(
                div()
                    .p(px(8.))
                    .text_color(theme.colors.muted_foreground)
                    .child("No components found"),
            );
        }
        let header = div()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(16.))
            .px(px(24.))
            .py(px(18.))
            .border_b_1()
            .border_color(theme.colors.border)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(
                        div()
                            .text_size(px(20.))
                            .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                            .child(name),
                    )
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(theme.colors.muted_foreground)
                            .child(description),
                    ),
            )
            .child(
                Button::new("gallery.reset")
                    .aria_label("Reset example")
                    .variant(ButtonVariant::Outline)
                    .label("Reset")
                    .on_click(cx.listener(|this, _, _, cx| this.select(this.selected, cx))),
            )
            .child(
                Button::new("gallery.theme")
                    .aria_label("Toggle light and dark theme")
                    .variant(ButtonVariant::Outline)
                    .child(if dark { "Light" } else { "Dark" })
                    .on_click(cx.listener(move |_, _, _, cx| {
                        UiTheme::set(
                            cx,
                            super::workspace::workspace_theme(if dark {
                                ThemeMode::Light
                            } else {
                                ThemeMode::Dark
                            }),
                        );
                        cx.refresh_windows();
                    })),
            );
        let content = div()
            .size_full()
            .min_w_0()
            .min_h_0()
            .flex()
            .flex_col()
            .child(header)
            .when(COMPONENTS[self.selected].0 == "sidebar", |el| {
                el.child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(4.))
                        .px(px(16.))
                        .py(px(8.))
                        .border_b_1()
                        .border_color(theme.colors.border)
                        .children(
                            [
                                ("workspace", "Workspace"),
                                ("docs", "Docs"),
                                ("mail", "Mail"),
                                ("floating", "Floating"),
                                ("mobile", "Sheet"),
                                ("loading", "Loading"),
                            ]
                            .into_iter()
                            .map(|(value, label)| {
                                Button::new(format!("gallery.sidebar.{value}"))
                                    .label(label)
                                    .aria_label(format!("Show {label} sidebar"))
                                    .variant(if self.preview.read(cx).sidebar_example == value {
                                        ButtonVariant::Secondary
                                    } else {
                                        ButtonVariant::Ghost
                                    })
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.preview = cx.new(|_| {
                                            let mut view = Showcase::new(Demo::Sidebar);
                                            view.sidebar_example = value.into();
                                            view
                                        });
                                        cx.notify();
                                    }))
                            }),
                        ),
                )
            })
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .child(self.preview.clone()),
            );
        div()
            .size_full()
            .bg(theme.colors.background)
            .text_color(theme.colors.foreground)
            .font_family(theme.fonts.body)
            .text_size(px(14.))
            .child(
                gpuicn::resizable::Resizable::new(
                    "gallery.split",
                    "Resize component navigation",
                    self.sidebar_width,
                    sidebar,
                    content,
                    cx.listener(|this, value, _, cx| {
                        this.sidebar_width = *value;
                        cx.notify();
                    }),
                )
                .first_limits(gpuicn::resizable::PaneLimits::new(px(190.), px(360.)))
                .second_limits(gpuicn::resizable::PaneLimits::new(px(480.), px(2000.))),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{Modifiers, TestAppContext, VisualTestContext, point};

    #[test]
    fn gallery_navigates_the_full_catalog_and_searches_without_reborrowing() {
        for slug in include_str!("../../web/src/lib/catalog.ts")
            .lines()
            .filter_map(|line| line.trim().strip_prefix("slug: \""))
            .filter_map(|line| line.split('"').next())
        {
            assert!(
                COMPONENTS.iter().any(|entry| entry.0 == slug),
                "gallery missing {slug}"
            );
        }
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            gpuicn::init(cx);
            UiTheme::set(cx, UiTheme::neutral_light());
        });
        let window = cx.add_window(|_, cx| Gallery::new(cx));
        let mut visual = VisualTestContext::from_window(window.into(), &cx);
        visual.simulate_resize(size(px(1180.), px(780.)));
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            cx.update(|cx| UiTheme::switch(cx, mode));
            for (index, (slug, _, _)) in COMPONENTS.iter().enumerate() {
                assert!(Demo::parse(slug).is_some(), "missing {slug}");
                window
                    .update(&mut cx, |view, _, cx| view.select(index, cx))
                    .unwrap();
                visual.update(|window, cx| window.draw(cx).clear(cx));
            }
        }
        visual.simulate_click(point(px(100.), px(69.)), Modifiers::default());
        for character in ["v", "i", "r", "t", "u", "a", "l"] {
            visual.simulate_input(character);
            cx.run_until_parked();
            visual.update(|window, cx| window.draw(cx).clear(cx));
        }
        assert_eq!(
            cx.read_window(&window, |view, cx| view.read(cx).query.clone())
                .unwrap(),
            "virtual"
        );
        visual.simulate_click(point(px(100.), px(126.)), Modifiers::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(
            cx.read_window(&window, |view, cx| COMPONENTS[view.read(cx).selected].0)
                .unwrap(),
            "virtual-list"
        );
    }
}
