use gpuicn::context_menu::context_menu;
#[cfg(not(target_family = "wasm"))]
mod gallery;
#[cfg(not(target_family = "wasm"))]
mod workspace;

use gpui_icons::{LucideAssetSource, LucideIcon, lucide};
use gpui_kit::{
    App, AppContext as _, Application, Bounds, Context, Entity, InteractiveElement as _,
    IntoElement, ParentElement as _, Render, SharedString, StatefulInteractiveElement as _,
    Styled as _, Window, WindowBounds, WindowOptions, div, prelude::FluentBuilder as _, px, size,
};
use gpuicn::{
    Button, ButtonSize, ButtonVariant, Checkbox, ThemeMode, UiTheme,
    accordion::*,
    alert_dialog::*,
    avatar::{Avatar, AvatarSize},
    checkbox_group::*,
    collapsible::*,
    dialog::*,
    drawer::{
        Drawer, DrawerContent, DrawerSide, drawer_body, drawer_close, drawer_description,
        drawer_footer, drawer_header, drawer_title, drawer_trigger,
    },
    field::*,
    fieldset::*,
    form::form,
    input::{Input, InputEvent, InputState},
    menu::{Menu, MenuItem, MenuState},
    menubar::Menubar,
    meter::Meter,
    navigation_menu::navigation_menu,
    number_field::NumberField,
    otp_field::{OtpField, OtpState},
    popover::*,
    preview_card::*,
    progress::Progress,
    radio_group::*,
    scroll_area::ScrollArea,
    select::{Select, SelectItem, SelectState},
    separator::Separator,
    slider::Slider,
    switch::Switch,
    tabs::*,
    toast::ToastState,
    toggle::{Toggle, ToggleVariant},
    toggle_group::*,
    toolbar::{Toolbar, ToolbarButton},
    tooltip::*,
};
use std::borrow::Cow;

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
    #[cfg(target_family = "wasm")]
    let app = gpui_kit::platform::application_with_web_backend(
        gpui_kit::platform::WebBackendPreference::WebGpu,
    );
    #[cfg(not(target_family = "wasm"))]
    let app = gpui_kit::application();

    app.with_assets(LucideAssetSource)
}

fn launch(cx: &mut App) {
    cx.text_system()
        .add_fonts(vec![
            Cow::Borrowed(include_bytes!("../assets/fonts/Geist-Regular.ttf")),
            Cow::Borrowed(include_bytes!("../assets/fonts/Geist-Medium.ttf")),
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
                if matches!(demo, Demo::Sidebar | Demo::VirtualList) {
                    960.
                } else {
                    640.
                },
                320.0,
                1440.0,
            )),
            px(requested_dimension(
                "height",
                if matches!(demo, Demo::Sidebar | Demo::VirtualList) {
                    600.
                } else {
                    288.
                },
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

#[derive(Clone, Copy, Default)]
enum Demo {
    Icons,
    Accordion,
    AlertDialog,
    Autocomplete,
    Avatar,
    #[default]
    Button,
    Checkbox,
    CheckboxGroup,
    Collapsible,
    Combobox,
    ContextMenu,
    Dialog,
    Drawer,
    Field,
    Fieldset,
    Form,
    Input,
    Menu,
    Menubar,
    Meter,
    NavigationMenu,
    NumberField,
    OtpField,
    Popover,
    PreviewCard,
    Progress,
    RadioGroup,
    ScrollArea,
    Select,
    Separator,
    Sidebar,
    Resizable,
    VirtualList,
    Slider,
    Switch,
    Tabs,
    Toast,
    Toggle,
    ToggleGroup,
    Toolbar,
    Tooltip,
}

impl Demo {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "icons" => Some(Self::Icons),
            "accordion" => Some(Self::Accordion),
            "alert-dialog" => Some(Self::AlertDialog),
            "autocomplete" => Some(Self::Autocomplete),
            "avatar" => Some(Self::Avatar),
            "button" => Some(Self::Button),
            "checkbox" => Some(Self::Checkbox),
            "checkbox-group" => Some(Self::CheckboxGroup),
            "collapsible" => Some(Self::Collapsible),
            "combobox" => Some(Self::Combobox),
            "context-menu" => Some(Self::ContextMenu),
            "dialog" => Some(Self::Dialog),
            "drawer" => Some(Self::Drawer),
            "field" => Some(Self::Field),
            "fieldset" => Some(Self::Fieldset),
            "form" => Some(Self::Form),
            "input" => Some(Self::Input),
            "menu" => Some(Self::Menu),
            "menubar" => Some(Self::Menubar),
            "meter" => Some(Self::Meter),
            "navigation-menu" => Some(Self::NavigationMenu),
            "number-field" => Some(Self::NumberField),
            "otp-field" => Some(Self::OtpField),
            "popover" => Some(Self::Popover),
            "preview-card" => Some(Self::PreviewCard),
            "progress" => Some(Self::Progress),
            "radio-group" => Some(Self::RadioGroup),
            "scroll-area" => Some(Self::ScrollArea),
            "select" => Some(Self::Select),
            "separator" => Some(Self::Separator),
            "sidebar" => Some(Self::Sidebar),
            "resizable" => Some(Self::Resizable),
            "virtual-list" => Some(Self::VirtualList),
            "slider" => Some(Self::Slider),
            "switch" => Some(Self::Switch),
            "tabs" => Some(Self::Tabs),
            "toast" => Some(Self::Toast),
            "toggle" => Some(Self::Toggle),
            "toggle-group" => Some(Self::ToggleGroup),
            "toolbar" => Some(Self::Toolbar),
            "tooltip" => Some(Self::Tooltip),
            _ => None,
        }
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
            .p(px(
                if matches!(self.demo, Demo::Sidebar | Demo::VirtualList) {
                    0.
                } else {
                    16.
                },
            ))
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

    fn preview(&self, window: &mut Window, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        match self.demo {
            Demo::Icons => self.icons_preview(cx).into_any_element(),
            Demo::Accordion => self.accordion_preview(window, cx).into_any_element(),
            Demo::AlertDialog => self.alert_dialog_preview(window, cx).into_any_element(),
            Demo::Autocomplete => self.autocomplete_preview(window, cx).into_any_element(),
            Demo::Avatar => self.avatar_preview(cx).into_any_element(),
            Demo::Button => self.button_preview(cx).into_any_element(),
            Demo::Checkbox => self.checkbox_preview(cx).into_any_element(),
            Demo::CheckboxGroup => self.checkbox_group_preview(window, cx).into_any_element(),
            Demo::Collapsible => self.collapsible_preview(window, cx).into_any_element(),
            Demo::Combobox => self.combobox_preview(window, cx).into_any_element(),
            Demo::ContextMenu => self.context_menu_preview(window, cx).into_any_element(),
            Demo::Dialog => self.dialog_preview(window, cx).into_any_element(),
            Demo::Drawer => self.drawer_preview(window, cx).into_any_element(),
            Demo::Field => self.field_preview(window, cx).into_any_element(),
            Demo::Fieldset => self.fieldset_preview(window, cx).into_any_element(),
            Demo::Form => self.form_preview(window, cx).into_any_element(),
            Demo::Input => self.input_preview(window, cx).into_any_element(),
            Demo::Menu => self.menu_preview(window, cx).into_any_element(),
            Demo::Menubar => self.menubar_preview(window, cx).into_any_element(),
            Demo::Meter => self.meter_preview().into_any_element(),
            Demo::NavigationMenu => self.navigation_menu_preview(window, cx).into_any_element(),
            Demo::NumberField => self.number_field_preview(window, cx).into_any_element(),
            Demo::OtpField => self.otp_field_preview(window, cx).into_any_element(),
            Demo::Popover => self.popover_preview(window, cx).into_any_element(),
            Demo::PreviewCard => self.preview_card_preview(window, cx).into_any_element(),
            Demo::Progress => self.progress_preview().into_any_element(),
            Demo::RadioGroup => self.radio_group_preview(window, cx).into_any_element(),
            Demo::ScrollArea => self.scroll_area_preview(window, cx).into_any_element(),
            Demo::Select => self.select_preview(window, cx).into_any_element(),
            Demo::Separator => self.separator_preview().into_any_element(),
            Demo::Sidebar => self.sidebar_preview(window, cx).into_any_element(),
            Demo::VirtualList => self.virtual_list_preview(cx).into_any_element(),
            Demo::Resizable => self.resizable_preview(cx).into_any_element(),
            Demo::Slider => self.slider_preview(window, cx).into_any_element(),
            Demo::Switch => self.switch_preview(cx).into_any_element(),
            Demo::Tabs => self.tabs_preview(window, cx).into_any_element(),
            Demo::Toast => self.toast_preview(window, cx).into_any_element(),
            Demo::Toggle => self.toggle_preview(cx).into_any_element(),
            Demo::ToggleGroup => self.toggle_group_preview(window, cx).into_any_element(),
            Demo::Toolbar => self.toolbar_preview(window, cx).into_any_element(),
            Demo::Tooltip => self.tooltip_preview(window, cx).into_any_element(),
        }
    }

    fn button_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let variants = [
            (ButtonVariant::Default, "Default"),
            (ButtonVariant::Secondary, "Secondary"),
            (ButtonVariant::Outline, "Outline"),
            (ButtonVariant::Ghost, "Ghost"),
            (ButtonVariant::Destructive, "Destructive"),
            (ButtonVariant::Link, "Link"),
        ];

        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(20.0))
            .child(
                Button::new("preview.button.interactive")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.count += 1;
                        cx.notify();
                    }))
                    .label(format!("Clicked {} times", self.count)),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .justify_center()
                    .gap(px(8.0))
                    .children(variants.into_iter().map(|(variant, label)| {
                        Button::new(format!("preview.button.{}", label.to_lowercase()))
                            .variant(variant)
                            .label(label)
                    })),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        Button::new("preview.button.small")
                            .size(ButtonSize::Xs)
                            .label("Extra small"),
                    )
                    .child(
                        Button::new("preview.button.large")
                            .size(ButtonSize::Lg)
                            .label("Large"),
                    )
                    .child(
                        Button::new("preview.button.icon")
                            .size(ButtonSize::Icon)
                            .variant(ButtonVariant::Outline)
                            .aria_label("Add one click")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.count += 1;
                                cx.notify();
                            }))
                            .child(
                                lucide(LucideIcon::Plus)
                                    .size(px(16.))
                                    .text_color(UiTheme::read(cx).colors.foreground),
                            ),
                    )
                    .child(
                        Button::new("preview.button.disabled")
                            .disabled(true)
                            .label("Disabled"),
                    ),
            )
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

    fn accordion_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let expanded = window.use_keyed_state("faq-open", cx, |_, _| 0_usize);
        let selected = *expanded.read(cx);
        accordion("preview.accordion", cx).w(px(480.)).children(
            [
                (
                    "Is it accessible?",
                    "Use Tab and Enter to open each section.",
                ),
                (
                    "Is it styled?",
                    "Yes. It follows the shadcn Nova visual system.",
                ),
                (
                    "Is it native?",
                    "Yes. GPUI Kit supplies the native controls.",
                ),
            ]
            .into_iter()
            .enumerate()
            .map(|(i, (title, text))| {
                let expanded = expanded.clone();
                accordion_item(cx)
                    .open(selected == i)
                    .header(accordion_header(
                        accordion_trigger(("faq", i), selected == i, false, cx)
                            .child(title)
                            .on_click(move |_, _, cx| {
                                expanded.update(cx, |selected, cx| {
                                    *selected = if *selected == i { usize::MAX } else { i };
                                    cx.notify();
                                })
                            }),
                    ))
                    .panel(accordion_content(cx).child(text))
            }),
        )
    }

    fn autocomplete_preview(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let state = window.use_keyed_state("autocomplete_preview", cx, |window, cx| {
            SelectState::new(
                [
                    SelectItem::new("apple", "Apple"),
                    SelectItem::new("banana", "Banana"),
                    SelectItem::new("orange", "Orange").disabled(true),
                    SelectItem::new("pear", "Pear"),
                ],
                window,
                cx,
            )
        });
        gpuicn::autocomplete::autocomplete(&state)
            .aria_label("Fruit")
            .placeholder("Search or enter a fruit…")
            .w(px(240.))
    }

    fn alert_dialog_preview(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let handle = window
            .use_keyed_state("alert_dialog_preview.handle", cx, |_, _| {
                DialogHandle::new(false)
            })
            .read(cx)
            .clone();
        let popup = dialog_popup("alert_dialog_preview.popup", "Are you sure?", cx)
            .child(dialog_title("alert_dialog_preview.title", cx).child("Are you sure?"))
            .child(
                dialog_description("alert_dialog_preview.description", cx)
                    .child("This action cannot be undone."),
            )
            .child(
                dialog_footer(cx)
                    .child(alert_dialog_cancel("cancel").label("Cancel"))
                    .child(alert_dialog_action("confirm").label("Continue")),
            );
        div()
            .child(
                dialog_trigger("alert_dialog_preview.trigger", &handle, cx).label("Delete account"),
            )
            .child(alert_dialog(
                "alert_dialog_preview",
                &handle,
                popup,
                window,
                cx,
            ))
    }

    fn avatar_preview(&self, cx: &App) -> impl IntoElement {
        let theme = UiTheme::read(cx);
        // Pinned to Arya's supplied revision; AJ remains the loading/error fallback.
        let image = "https://raw.githubusercontent.com/devaryakjha/devaryakjha/6526e3d7415b2fb573ba3da4523b5c6948aa5d08/avatar.png";
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(28.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(28.))
                    .child(
                        Avatar::new("preview.avatar.image")
                            .image(image)
                            .aria_label("Arya")
                            .fallback("AJ"),
                    )
                    .child(
                        Avatar::new("preview.avatar.fallback")
                            .aria_label("Initials fallback")
                            .fallback("AJ"),
                    )
                    .child(
                        div().flex().items_center().children(
                            [
                                ("preview.avatar.group.arya", "AJ", true),
                                ("preview.avatar.group.member", "SK", false),
                                ("preview.avatar.group.count", "+3", false),
                            ]
                            .into_iter()
                            .enumerate()
                            .map(
                                |(index, (id, initials, has_image))| {
                                    div()
                                        .rounded_full()
                                        .border_2()
                                        .border_color(theme.colors.background)
                                        .when(index > 0, |item| item.ml(px(-10.)))
                                        .child(
                                            Avatar::new(id)
                                                .when(has_image, |avatar| avatar.image(image))
                                                .aria_label(if initials == "+3" {
                                                    "3 more members"
                                                } else {
                                                    initials
                                                })
                                                .fallback(initials),
                                        )
                                },
                            ),
                        ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(12.))
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(theme.colors.muted_foreground)
                            .child("Sizes"),
                    )
                    .child(
                        div().flex().items_center().gap(px(12.)).children(
                            [
                                ("preview.avatar.small", AvatarSize::Sm),
                                ("preview.avatar.default", AvatarSize::Default),
                                ("preview.avatar.large", AvatarSize::Lg),
                            ]
                            .into_iter()
                            .map(|(id, size)| {
                                Avatar::new(id)
                                    .size(size)
                                    .image(image)
                                    .aria_label("Arya")
                                    .fallback("AJ")
                            }),
                        ),
                    ),
            )
    }

    fn collapsible_preview(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        collapsible(cx)
            .open(self.collapsible_open)
            .w(px(320.))
            .child(
                collapsible_trigger("preview.disclosure", self.collapsible_open, cx)
                    .child("Recent projects")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.collapsible_open = !this.collapsible_open;
                        cx.notify();
                    })),
            )
            .content(collapsible_content(cx).child("Design system · Documentation · Website"))
    }

    fn checkbox_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let checkbox_view = cx.entity().downgrade();
        let label_view = cx.entity().downgrade();
        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        Checkbox::new("preview.checkbox.interactive")
                            .checked(self.checked)
                            .aria_label("Accept terms")
                            .on_change(move |checked, _, _, cx| {
                                checkbox_view
                                    .update(cx, |this, cx| {
                                        this.checked = checked;
                                        cx.notify();
                                    })
                                    .ok();
                            }),
                    )
                    .child(
                        div()
                            .id("preview.checkbox.interactive-label")
                            .cursor_pointer()
                            .on_click(move |_, _, cx| {
                                label_view
                                    .update(cx, |this, cx| {
                                        this.checked = !this.checked;
                                        cx.notify();
                                    })
                                    .ok();
                            })
                            .child(if self.checked {
                                "Accepted"
                            } else {
                                "Accept terms"
                            }),
                    ),
            )
            .child(checkbox_row(
                Checkbox::new("preview.checkbox.indeterminate")
                    .indeterminate(true)
                    .aria_label("Partly selected"),
                "Partly selected",
            ))
            .child(checkbox_row(
                Checkbox::new("preview.checkbox.disabled")
                    .checked(true)
                    .disabled(true)
                    .aria_label("Disabled"),
                "Disabled",
            ))
            .child(checkbox_row(
                Checkbox::new("preview.checkbox.readonly")
                    .checked(true)
                    .read_only(true)
                    .aria_label("Read only"),
                "Read only",
            ))
    }

    fn checkbox_group_preview(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let value = window.use_keyed_state("checkbox-group", cx, |_, _| {
            vec![SharedString::from("email")]
        });
        CheckboxGroup::new("notifications")
            .aria_label("Notifications")
            .value(value.read(cx).clone())
            .item(CheckboxGroupItem::new("email", "email").label("Email notifications"))
            .item(CheckboxGroupItem::new("push", "push").label("Push notifications"))
            .item(
                CheckboxGroupItem::new("sms", "sms")
                    .label("SMS notifications")
                    .disabled(true),
            )
            .on_change(move |next, _, cx| {
                value.update(cx, |value, cx| {
                    *value = next;
                    cx.notify();
                })
            })
    }

    fn combobox_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = window.use_keyed_state("combobox_preview", cx, |window, cx| {
            SelectState::new(
                [
                    SelectItem::new("apple", "Apple"),
                    SelectItem::new("banana", "Banana"),
                    SelectItem::new("orange", "Orange").disabled(true),
                    SelectItem::new("pear", "Pear"),
                ],
                window,
                cx,
            )
        });
        gpuicn::combobox::combobox(&state)
            .aria_label("Fruit")
            .placeholder("Search a fruit…")
            .w(px(240.))
    }

    fn context_menu_preview(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let click = cx.listener(|this, _, _, cx| {
            this.demo_action = "Copied".into();
            cx.notify();
        });
        let menu = window.use_keyed_state("context-menu", cx, |_, cx| {
            MenuState::new(
                [
                    MenuItem::new("copy", "Copy").on_click(click),
                    MenuItem::new("paste", "Paste").disabled(true),
                    MenuItem::separator(),
                    MenuItem::new("show-bookmarks", "Show bookmarks").checked(true),
                    MenuItem::submenu(
                        "share",
                        "Share",
                        [MenuItem::new("documentation", "Documentation")
                            .link()
                            .on_click(|_, _, cx| cx.open_url("https://ui.imajha.com"))],
                    ),
                ],
                cx,
            )
        });
        gpuicn::context_menu::context_menu(
            &menu,
            "File actions",
            div()
                .w(px(280.))
                .h(px(160.))
                .border_1()
                .border_color(UiTheme::read(cx).colors.border)
                .rounded(px(8.))
                .flex()
                .items_center()
                .justify_center()
                .child("Right click here"),
        )
    }

    fn dialog_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let handle = window
            .use_keyed_state("dialog_preview.handle", cx, |_, _| DialogHandle::new(false))
            .read(cx)
            .clone();
        let name = demo_input("dialog_preview.name", "Ada Lovelace", "Name", window, cx);
        let popup = dialog_popup("dialog_preview.popup", "Edit profile", cx)
            .child(dialog_title("dialog_preview.title", cx).child("Edit profile"))
            .child(
                dialog_description("dialog_preview.description", cx)
                    .child("Update the name shown on your profile."),
            )
            .child(Input::new(&name).aria_label("Name"))
            .child(
                dialog_footer(cx).child(dialog_action("save", &handle, cx).label("Save changes")),
            );
        div()
            .child(dialog_trigger("dialog_preview.trigger", &handle, cx).label("Edit profile"))
            .child(dialog("dialog_preview", &handle, popup, window, cx))
    }

    fn drawer_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let handle = window
            .use_keyed_state("drawer.handle", cx, |_, _| DialogHandle::new(false))
            .read(cx)
            .clone();
        let side = self.drawer_direction;
        let theme = UiTheme::read(cx).clone();
        let header = drawer_header(side, cx)
            .child(drawer_title("drawer.title", cx).child("Move goal"))
            .child(
                drawer_description("drawer.description", cx).child("Set your daily activity goal."),
            );
        let body = drawer_body("drawer.body", cx).pt_0().child(
            div()
                .mx_auto()
                .w_full()
                .max_w(theme.space(80.))
                .flex()
                .flex_col()
                .gap(theme.space(5.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap(theme.space(4.))
                        .child(
                            Button::new("goal-minus")
                                .variant(ButtonVariant::Outline)
                                .label("−")
                                .aria_label("Decrease daily goal")
                                .rounded_full()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.goal = (this.goal - 10).max(10);
                                    cx.notify();
                                })),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .child(
                                    div()
                                        .text_size(theme.text(48.))
                                        .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                                        .child(self.goal.to_string()),
                                )
                                .child(
                                    div()
                                        .text_color(theme.colors.muted_foreground)
                                        .text_size(theme.text(11.))
                                        .child("CALORIES / DAY"),
                                ),
                        )
                        .child(
                            Button::new("goal-plus")
                                .variant(ButtonVariant::Outline)
                                .label("+")
                                .aria_label("Increase daily goal")
                                .rounded_full()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.goal += 10;
                                    cx.notify();
                                })),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_end()
                        .gap(theme.space(1.5))
                        .h(theme.space(16.))
                        .children(
                            [40., 65., 45., 85., 55., 95., 70., 60., 90., 75., 50., 80.]
                                .into_iter()
                                .map(|height| {
                                    div()
                                        .flex_1()
                                        .h(theme.space(16.) * (height / 100.))
                                        .rounded_t(theme.radius.sm)
                                        .bg(theme.colors.foreground)
                                }),
                        ),
                ),
        );
        let footer = drawer_footer(cx)
            .mx_auto()
            .w_full()
            .max_w(theme.space(88.))
            .child(
                drawer_close("goal-save", &handle)
                    .variant(ButtonVariant::Default)
                    .label("Save goal"),
            )
            .child(drawer_close("goal-cancel", &handle).label("Cancel"));
        Drawer::new("drawer", &handle)
            .direction(side)
            .show_swipe_handle(true)
            .flex()
            .flex_col()
            .items_center()
            .gap(theme.space(4.))
            .child(drawer_trigger("drawer.trigger", &handle).label("Open drawer"))
            .child(
                div().flex().gap(theme.space(2.)).children(
                    [
                        ("top", DrawerSide::Top),
                        ("right", DrawerSide::Right),
                        ("bottom", DrawerSide::Bottom),
                        ("left", DrawerSide::Left),
                    ]
                    .into_iter()
                    .map(|(label, direction)| {
                        Button::new(label)
                            .variant(if side == direction {
                                ButtonVariant::Secondary
                            } else {
                                ButtonVariant::Ghost
                            })
                            .label(label)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.drawer_direction = direction;
                                cx.notify();
                            }))
                    }),
                ),
            )
            .content(
                DrawerContent::new("drawer.content", "Move goal")
                    .child(header)
                    .child(body)
                    .child(footer),
            )
    }

    fn field_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let username = demo_input("field.username", "", "e.g. ada", window, cx);
        let email = demo_input("field.email", "", "you@example.com", window, cx);
        div()
            .w(px(280.))
            .flex()
            .flex_col()
            .gap(px(20.))
            .child(
                Field::new("username", &username)
                    .label("Username")
                    .description("Visible on your public profile."),
            )
            .child(
                Field::new("email", &email)
                    .label("Email")
                    .error("Enter a valid email address."),
            )
    }

    fn fieldset_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let name = demo_input("shipping.name", "", "Ada Lovelace", window, cx);
        let city = demo_input("shipping.city", "", "London", window, cx);
        fieldset_root("shipping", cx)
            .w(px(300.))
            .aria_label("Shipping address")
            .child(fieldset_legend(FieldsetLegendVariant::Legend, cx).child("Shipping address"))
            .child(Field::new("shipping-name", &name).label("Full name"))
            .child(Field::new("shipping-city", &city).label("City"))
    }

    fn form_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let invalid = window.use_keyed_state("subscribe.invalid", cx, |_, _| false);
        let enter_invalid = invalid.clone();
        let email = live_input(
            "subscribe.email",
            "",
            "you@example.com",
            window,
            cx,
            move |this, input, event, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    this.submit_form(input.read(cx).value(), &enter_invalid, cx);
                }
            },
        );
        let error = *invalid.read(cx);
        form("subscribe", cx)
            .w(px(280.))
            .aria_label("Subscribe")
            .child(
                Field::new("subscribe-email", &email)
                    .label("Email")
                    .required(true)
                    .when(error, |f| f.error("Enter a valid email address.")),
            )
            .child(
                Button::new("subscribe-submit")
                    .label("Subscribe")
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.submit_form(email.read(cx).value(), &invalid, cx);
                    })),
            )
            .when(self.count > 0, |f| f.child("Form submitted."))
    }

    fn submit_form(&mut self, value: SharedString, invalid: &Entity<bool>, cx: &mut Context<Self>) {
        let valid = value
            .split_once('@')
            .is_some_and(|(name, host)| !name.is_empty() && host.contains('.'));
        invalid.update(cx, |invalid, cx| {
            *invalid = !valid;
            cx.notify();
        });
        if valid {
            self.count += 1;
        }
        cx.notify();
    }

    fn input_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let email = demo_input("input.email", "", "Email", window, cx);
        let readonly = demo_input("input.readonly", "read-only@example.com", "", window, cx);
        let disabled = demo_input("input.disabled", "disabled@example.com", "", window, cx);
        div()
            .w(px(320.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(Input::new(&email).aria_label("Email address"))
            .child(
                Input::new(&readonly)
                    .aria_label("Read-only email")
                    .read_only(true),
            )
            .child(
                Input::new(&disabled)
                    .aria_label("Disabled email")
                    .disabled(true),
            )
    }

    fn menu_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sign_out = cx.listener(|this, _, _, cx| {
            this.demo_action = "Signed out".into();
            cx.notify();
        });
        let menu = window.use_keyed_state("account-menu", cx, |_, cx| {
            MenuState::new(
                [
                    MenuItem::label("account-heading", "My account"),
                    MenuItem::new("profile", "Profile"),
                    MenuItem::new("notifications", "Notifications").checked(true),
                    MenuItem::separator(),
                    MenuItem::submenu(
                        "invite",
                        "Invite",
                        [
                            MenuItem::new("email", "Email"),
                            MenuItem::new("message", "Message"),
                        ],
                    ),
                    MenuItem::new("billing", "Billing").disabled(true),
                    MenuItem::new("sign-out", "Sign out").on_click(sign_out),
                ],
                cx,
            )
        });
        Menu::new(&menu, "My account")
    }

    fn menubar_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let file = window.use_keyed_state("menubar.file", cx, |_, cx| {
            MenuState::new(
                [
                    MenuItem::new("new-tab", "New tab"),
                    MenuItem::new("new-window", "New window"),
                    MenuItem::separator(),
                    MenuItem::new("quit", "Quit"),
                ],
                cx,
            )
        });
        let edit = window.use_keyed_state("menubar.edit", cx, |_, cx| {
            MenuState::new(
                [
                    MenuItem::new("undo", "Undo"),
                    MenuItem::new("redo", "Redo").disabled(true),
                    MenuItem::new("copy", "Copy"),
                ],
                cx,
            )
        });
        Menubar::new("app-menu", "Application menu")
            .menu(&file, "File")
            .menu(&edit, "Edit")
    }

    fn meter_preview(&self) -> impl IntoElement {
        div()
            .w_full()
            .max_w(px(320.0))
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child("Storage used · 68%")
            .child(
                Meter::new("preview.meter")
                    .value(68.0)
                    .aria_label("Storage used: 68 percent"),
            )
    }

    fn number_field_preview(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let state = window.use_keyed_state("quantity", cx, |window, cx| {
            InputState::new(window, cx)
                .default_value("3")
                .min(0.)
                .max(20.)
                .step(1.)
        });
        NumberField::new(&state).aria_label("Quantity").w(px(200.))
    }

    fn navigation_menu_preview(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let getting_started = window.use_keyed_state("nav.start", cx, |_, cx| {
            MenuState::new(
                [
                    MenuItem::new("introduction", "Introduction")
                        .link()
                        .on_click(|_, _, cx| cx.open_url("https://ui.imajha.com/docs")),
                    MenuItem::new("installation", "Installation")
                        .link()
                        .on_click(|_, _, cx| {
                            cx.open_url("https://ui.imajha.com/docs/installation")
                        }),
                ],
                cx,
            )
        });
        let components = window.use_keyed_state("nav.components", cx, |_, cx| {
            MenuState::new(
                [
                    MenuItem::new("button", "Button")
                        .link()
                        .on_click(|_, _, cx| {
                            cx.open_url("https://ui.imajha.com/docs/components/button")
                        }),
                    MenuItem::new("dialog", "Dialog")
                        .link()
                        .on_click(|_, _, cx| {
                            cx.open_url("https://ui.imajha.com/docs/components/dialog")
                        }),
                ],
                cx,
            )
        });
        navigation_menu("documentation-nav", "Documentation")
            .menu(&getting_started, "Getting started")
            .menu(&components, "Components")
    }

    fn otp_field_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = window.use_keyed_state("otp", cx, |window, cx| OtpState::new(6, window, cx));
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child("Enter the six-digit code")
            .child(OtpField::new(&state).aria_label("Verification code"))
    }

    fn popover_preview(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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

    fn preview_card_preview(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        preview_card("profile-card")
            .trigger(
                Button::new("profile-card.trigger")
                    .variant(ButtonVariant::Link)
                    .label("@gpuicn"),
            )
            .content(|_, _, cx| {
                preview_card_popup("profile-card.popup", cx).child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(8.))
                        .child("gpuicn")
                        .child("Open-code Nova components for native GPUI applications."),
                )
            })
    }

    fn progress_preview(&self) -> impl IntoElement {
        div()
            .w_full()
            .max_w(px(320.0))
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child("Uploading · 64%")
            .child(
                Progress::new("preview.progress")
                    .value(64.0)
                    .label("Uploading…"),
            )
            .child("Waiting · indeterminate")
            .child(
                Progress::new("preview.progress.indeterminate")
                    .indeterminate()
                    .label("Waiting…"),
            )
    }

    fn radio_group_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let value = window.use_keyed_state("radio", cx, |_, _| SharedString::from("comfortable"));
        RadioGroup::new("density")
            .aria_label("Density")
            .value(value.read(cx).clone())
            .item(RadioItem::new("compact", "compact").label("Compact"))
            .item(RadioItem::new("comfortable", "comfortable").label("Comfortable"))
            .item(
                RadioItem::new("spacious", "spacious")
                    .label("Spacious")
                    .disabled(true),
            )
            .on_change(move |next, _, cx| {
                value.update(cx, |value, cx| {
                    *value = next;
                    cx.notify();
                })
            })
    }

    fn scroll_area_preview(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        ScrollArea::new("tags")
            .aria_label("Tags")
            .w(px(240.))
            .h(px(240.))
            .children((1..=50).map(|i| {
                div()
                    .px(px(12.))
                    .py(px(8.))
                    .child(format!("Version 1.0.{i}"))
            }))
    }

    fn select_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = window.use_keyed_state("select_preview", cx, |window, cx| {
            SelectState::new(
                [
                    SelectItem::new("apple", "Apple"),
                    SelectItem::new("banana", "Banana"),
                    SelectItem::new("orange", "Orange").disabled(true),
                    SelectItem::new("pear", "Pear"),
                ],
                window,
                cx,
            )
        });
        Select::new(&state)
            .aria_label("Fruit")
            .placeholder("Select a fruit…")
            .w(px(240.))
    }

    fn resizable_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        use gpuicn::resizable::{PaneLimits, Resizable};
        div().w(px(400.)).h(px(200.)).child(
            Resizable::new(
                "preview.split",
                "Resize panels",
                self.pane_width,
                div().p(px(16.)).child("One"),
                div().p(px(16.)).child("Two"),
                cx.listener(|this, size: &gpui_kit::Pixels, _, cx| {
                    this.pane_width = *size;
                    cx.notify();
                }),
            )
            .first_limits(PaneLimits::new(px(80.), px(300.)))
            .second_limits(PaneLimits::new(px(80.), px(300.))),
        )
    }

    fn virtual_list_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        use gpuicn::virtual_list::{ListItem, VirtualList, VirtualListEvent};
        let t = UiTheme::read(cx).clone();
        let s = t.spacing.unit;
        let rows = self
            .virtual_rows
            .as_ref()
            .expect("list preview state")
            .clone();
        let details = self.virtual_details;
        let current = rows
            .focused()
            .and_then(|id| rows.index_of(&id))
            .and_then(|index| rows.item(index));
        let view = cx.entity().downgrade();
        let menu_rows = rows.clone();
        let menu_theme = t.clone();
        let list = VirtualList::new(
            "files.list",
            "Source files",
            rows.clone(),
            move |row, window, cx| {
                let state = menu_rows.clone();
                let view = view.clone();
                let target = row.item.id.clone();
                let content = div()
                    .w_full()
                    .min_w_0()
                    .flex()
                    .items_center()
                    .gap(s * 2_f32)
                    .when(details, |el| el.py(s))
                    .child(
                        lucide(LucideIcon::FileCode)
                            .size(s * 4_f32)
                            .text_color(menu_theme.colors.muted_foreground),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .child(div().truncate().child(row.item.label.clone()))
                            .when(details, |el| {
                                el.child(
                                    div()
                                        .text_size(px(12.) * menu_theme.text_scale)
                                        .line_height(px(16.) * menu_theme.text_scale)
                                        .text_color(menu_theme.colors.muted_foreground)
                                        .child("src/components · Updated a moment ago"),
                                )
                            }),
                    )
                    .child(
                        div()
                            .text_size(px(12.) * menu_theme.text_scale)
                            .text_color(menu_theme.colors.muted_foreground)
                            .child(if row.item.disabled { "Locked" } else { "M" }),
                    );
                let menu =
                    window
                        .use_keyed_state((row.item.id.clone(), "menu"), cx, |_, cx| {
                            cx.new(|cx| {
                                MenuState::new(
                                    [MenuItem::new("inspect-selection", "Inspect selection")
                                        .on_click(move |_, _, cx| {
                                            if let Some(item) = state
                                                .index_of(&target)
                                                .and_then(|index| state.item(index))
                                            {
                                                let count = state.selected_count();
                                                let _ = view.update(cx, |this, cx| {
                                                    this.virtual_note = format!(
                                                        "Inspect {} · {count} selected",
                                                        item.label
                                                    );
                                                    cx.notify();
                                                });
                                            }
                                        })],
                                    cx,
                                )
                            })
                        })
                        .read(cx)
                        .clone();
                context_menu(&menu, "File actions", content)
                    .w_full()
                    .disabled(row.item.disabled)
                    .into_any_element()
            },
        )
        .when(details, |list| list.row_height(s * 12_f32))
        .on_event(cx.listener(|this, event, _, cx| {
            if let VirtualListEvent::Activate(id) = event
                && let Some(state) = &this.virtual_rows
                && let Some(item) = state.index_of(id).and_then(|index| state.item(index))
            {
                this.virtual_note = format!("Opened {}", item.label);
            }
            cx.notify();
        }));
        div().size_full().min_w_0().min_h_0().flex().flex_col()
            .font_family(t.fonts.body).text_size(px(14.) * t.text_scale).line_height(px(20.) * t.text_scale)
            .child(div().px(s * 4_f32).py(s * 3_f32).flex().items_center().justify_between().gap(s * 2_f32)
                .border_b_1().border_color(t.colors.border)
                .child(div().flex().flex_col().child(div().font_weight(gpui_kit::FontWeight::MEDIUM).child("Source files"))
                    .child(div().text_size(px(12.) * t.text_scale).text_color(t.colors.muted_foreground)
                        .child(format!("{} rows · Multi-select", rows.len()))))
                .child(Button::new("files.details").aria_label("Toggle row details").variant(if details { ButtonVariant::Secondary } else { ButtonVariant::Outline })
                    .size(ButtonSize::Sm).label("Details")
                    .on_click(cx.listener(|this, _, _, cx| { this.virtual_details = !this.virtual_details; cx.notify(); }))))
            .child(div().px(s * 3_f32).py(s * 2_f32).flex().flex_wrap().gap(s * 2_f32)
                .border_b_1().border_color(t.colors.border)
                .child(Button::new("files.reveal").aria_label("Jump to row 50,000").variant(ButtonVariant::Outline).size(ButtonSize::Sm).label("Jump to 50,000")
                    .on_click(cx.listener(|this, _, _, cx| {
                        if let Some(rows) = &this.virtual_rows { rows.reveal(&("file", 49_999usize).into()); }
                        cx.notify();
                    })))
                .child(Button::new("files.reverse").aria_label("Reverse row order").variant(ButtonVariant::Outline).size(ButtonSize::Sm)
                    .child(if self.virtual_reversed { "Original order" } else { "Reverse order" })
                    .on_click(cx.listener(|this, _, _, cx| {
                        if let Some(rows) = &this.virtual_rows {
                            let items = (0..rows.len()).rev().filter_map(|index| rows.item(index)).collect();
                            rows.replace_items(items).expect("reorder retains unique IDs");
                            this.virtual_reversed = !this.virtual_reversed;
                        }
                        cx.notify();
                    })))
                .child(Button::new("files.refresh").aria_label("Refresh rows").variant(ButtonVariant::Outline).size(ButtonSize::Sm).label("Refresh rows")
                    .on_click(cx.listener(|this, _, _, cx| {
                        if let Some(rows) = &this.virtual_rows {
                            let added = gpui_kit::ElementId::from("new-file");
                            let existed = rows.index_of(&added).is_some();
                            let mut items: Vec<_> = (0..rows.len()).filter_map(|index| rows.item(index)).filter(|item| item.id != added).collect();
                            if !existed { items.insert(0, ListItem::new("new-file", "new_component.rs")); }
                            rows.replace_items(items).expect("refresh keeps unique IDs");
                            this.virtual_note = if existed { "Removed the added row; surviving selection and viewport retained." } else { "Added a row at the top; surviving selection and viewport retained." }.into();
                        }
                        cx.notify();
                    }))))
            .child(div().flex_1().min_h_0().min_w_0().px(s * 2_f32).py(s).child(list))
            .child(div().px(s * 4_f32).py(s * 3_f32).border_t_1().border_color(t.colors.border).flex().flex_col().gap(s)
                .child(div().flex().justify_between().gap(s * 2_f32)
                    .child(format!("{} selected", rows.selected_count()))
                    .child(div().min_w_0().truncate().text_color(t.colors.muted_foreground)
                        .child(current.map(|item| item.label).unwrap_or_else(|| "Use arrows, Shift and Cmd/Ctrl".into()))))
                .when(!self.virtual_note.is_empty(), |el| el.child(div().text_size(px(12.) * t.text_scale)
                    .line_height(px(16.) * t.text_scale).text_color(t.colors.muted_foreground).child(self.virtual_note.clone()))))
    }

    fn sidebar_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        match self.sidebar_example.as_str() {
            "mail" => self.sidebar_mail_preview(window, cx).into_any_element(),
            "docs" => self.sidebar_docs_preview(window, cx).into_any_element(),
            _ => self
                .sidebar_application_preview(window, cx)
                .into_any_element(),
        }
    }

    fn sidebar_application_preview(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        use gpui_kit::FontWeight;
        use gpuicn::sidebar::*;
        let t = UiTheme::read(cx).clone();
        let s = t.spacing.unit;
        let example = self.sidebar_example.as_str();
        let mobile = self.sidebar_mobile || example == "mobile";
        let mode = if example == "loading" {
            SidebarCollapsible::None
        } else {
            SidebarCollapsible::Icon
        };
        let collapsed = self.sidebar_state.icon_collapsed(mobile, mode);
        let variant = match example {
            "workspace" => SidebarVariant::Inset,
            "floating" => SidebarVariant::Floating,
            _ => SidebarVariant::Sidebar,
        };
        let right = example == "floating";
        let labels: &[(&str, LucideIcon)] = match example {
            "floating" => &[
                ("Overview", LucideIcon::LayoutDashboard),
                ("Activity", LucideIcon::Activity),
                ("Files", LucideIcon::Folder),
                ("Settings", LucideIcon::Settings),
            ],
            _ => &[
                ("Overview", LucideIcon::House),
                ("Projects", LucideIcon::Folder),
                ("Inbox", LucideIcon::Inbox),
                ("Team", LucideIcon::Users),
            ],
        };
        let selected = self.sidebar_selected.min(labels.len() - 1);
        let title = labels[selected].0;
        let workspace = if self.sidebar_workspace == 0 {
            "Acme Studio"
        } else {
            "Personal workspace"
        };
        let toggle = cx.listener(move |this, _, _, cx| {
            this.sidebar_state.toggle(mobile);
            cx.notify();
        });
        let items = ["Acme Studio", "Personal workspace"]
            .into_iter()
            .enumerate()
            .map(|(index, label)| {
                let view = cx.entity().downgrade();
                MenuItem::new(label, label).on_click(move |_, _, cx| {
                    let _ = view.update(cx, |this, cx| {
                        this.sidebar_workspace = index;
                        cx.notify();
                    });
                })
            })
            .collect::<Vec<_>>();
        let state = window
            .use_keyed_state("sidebar.team.state", cx, |_, cx| {
                cx.new(|cx| MenuState::new(items, cx))
            })
            .read(cx)
            .clone();
        let team_menu = sidebar_dropdown(&state, "Switch workspace", cx).trigger(
            div()
                .flex()
                .w_full()
                .items_center()
                .gap(s * 2_f32)
                .child(
                    div()
                        .size(s * 8_f32)
                        .flex_shrink_0()
                        .rounded(t.radius.lg)
                        .bg(t.colors.sidebar_primary)
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            lucide(LucideIcon::Command)
                                .size(s * 4_f32)
                                .text_color(t.colors.sidebar_primary_foreground),
                        ),
                )
                .when(!collapsed, |el| {
                    el.child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .truncate()
                                    .font_weight(FontWeight::MEDIUM)
                                    .line_height(px(17.5) * t.text_scale)
                                    .child(workspace),
                            )
                            .child(
                                div()
                                    .truncate()
                                    .text_size(px(12.) * t.text_scale)
                                    .line_height(px(16.) * t.text_scale)
                                    .child("Enterprise"),
                            ),
                    )
                    .child(
                        lucide(LucideIcon::ChevronsUpDown)
                            .size(s * 4_f32)
                            .text_color(t.colors.sidebar_foreground),
                    )
                }),
        );
        let header = div().flex().flex_col().gap(s * 2_f32).child(team_menu);
        let mut menu = sidebar_menu();
        let loading = example == "loading" && !self.sidebar_loaded;
        if loading {
            for _ in 0..5 {
                menu = menu.child(sidebar_menu_skeleton(!collapsed, cx));
            }
        } else {
            for (index, (label, icon)) in labels.iter().enumerate() {
                let expandable = index == 1 && example == "workspace";
                let item = SidebarItem::new(("sidebar.nav", index), (*label).to_owned())
                    .icon(
                        lucide(*icon)
                            .size(s * 4_f32)
                            .text_color(t.colors.sidebar_foreground),
                    )
                    .collapsed(collapsed)
                    .selected(selected == index)
                    .when(expandable, |el| el.expanded(self.sidebar_nested))
                    .when(index == 1 && example == "workspace" && !collapsed, |el| {
                        el.trailing(
                            lucide(if self.sidebar_nested {
                                LucideIcon::ChevronDown
                            } else {
                                LucideIcon::ChevronRight
                            })
                            .size(s * 4_f32)
                            .text_color(t.colors.sidebar_foreground),
                        )
                    })
                    .when(*label == "Inbox" && !collapsed, |el| {
                        el.trailing(sidebar_badge("12", cx))
                    })
                    .on_activate(cx.listener(move |this, _, _, cx| {
                        this.sidebar_selected = index;
                        if expandable {
                            this.sidebar_nested = !this.sidebar_nested;
                        } else {
                            this.sidebar_state.mobile_open = false;
                        }
                        cx.notify();
                    }));
                menu = menu.child(item);
                if example == "workspace" && index == 1 && !collapsed && self.sidebar_nested {
                    menu = menu.child(
                        sidebar_menu_sub(cx)
                            .child(
                                SidebarItem::new("sidebar.nested.website", "Website")
                                    .size(SidebarItemSize::Small)
                                    .on_activate(cx.listener(|this, _, _, cx| {
                                        this.sidebar_note = "Website project opened".into();
                                        cx.notify();
                                    })),
                            )
                            .child(
                                SidebarItem::new("sidebar.nested.mobile", "Mobile app")
                                    .size(SidebarItemSize::Small)
                                    .on_activate(cx.listener(|this, _, _, cx| {
                                        this.sidebar_note = "Mobile app project opened".into();
                                        cx.notify();
                                    })),
                            ),
                    );
                }
            }
        }
        let mut navigation = Sidebar::new("sidebar.navigation", "Main navigation")
            .header(header)
            .when(!collapsed, |el| {
                el.child(sidebar_group_label("Platform", cx))
            })
            .child(menu);
        if !collapsed {
            navigation = navigation.child(div().h(s * 4_f32).flex_shrink_0()).child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(sidebar_group_label("Projects", cx))
                    .child(
                        sidebar_menu_action("sidebar.add", "Add project", cx)
                            .child(
                                lucide(LucideIcon::Plus)
                                    .size(s * 4_f32)
                                    .text_color(t.colors.sidebar_foreground),
                            )
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.sidebar_projects += 1;
                                this.sidebar_note =
                                    format!("Created project {}", this.sidebar_projects);
                                cx.notify();
                            })),
                    ),
            );
            for index in 0..self.sidebar_projects {
                navigation = navigation.child(
                    SidebarItem::new(("sidebar.project", index), format!("Project {}", index + 1))
                        .icon(
                            lucide(LucideIcon::Folder)
                                .size(s * 4_f32)
                                .text_color(t.colors.sidebar_foreground),
                        )
                        .on_activate(cx.listener(move |this, _, _, cx| {
                            this.sidebar_note = format!("Project {} opened", index + 1);
                            cx.notify();
                        })),
                );
            }
        }
        let items = vec![
            MenuItem::new("view-profile", "View profile").on_click({
                let view = cx.entity().downgrade();
                move |_, _, cx| {
                    let _ = view.update(cx, |this, cx| {
                        this.sidebar_note = "Alex Morgan · alex@example.com".into();
                        cx.notify();
                    });
                }
            }),
            MenuItem::new("notifications", "Notifications").checked(true),
        ];
        let state = window
            .use_keyed_state("sidebar.account.state", cx, |_, cx| {
                cx.new(|cx| MenuState::new(items, cx))
            })
            .read(cx)
            .clone();
        let account = sidebar_dropdown(&state, "Account menu", cx).trigger(
            div()
                .flex()
                .w_full()
                .items_center()
                .gap(s * 2_f32)
                .child(
                    div()
                        .size(s * 8_f32)
                        .flex_shrink_0()
                        .rounded(t.radius.lg)
                        .bg(t.colors.sidebar_accent)
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(12.) * t.text_scale)
                        .line_height(px(16.) * t.text_scale)
                        .font_weight(FontWeight::MEDIUM)
                        .child("AM"),
                )
                .when(!collapsed, |el| {
                    el.child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .truncate()
                                    .font_weight(FontWeight::MEDIUM)
                                    .line_height(px(17.5) * t.text_scale)
                                    .child("Alex Morgan"),
                            )
                            .child(
                                div()
                                    .truncate()
                                    .text_size(px(12.) * t.text_scale)
                                    .line_height(px(16.) * t.text_scale)
                                    .child("alex@example.com"),
                            ),
                    )
                    .child(
                        lucide(LucideIcon::ChevronsUpDown)
                            .size(s * 4_f32)
                            .text_color(t.colors.sidebar_foreground),
                    )
                }),
        );
        navigation = navigation.footer(
            div()
                .flex()
                .flex_col()
                .gap(s * 2_f32)
                .when(mobile, |el| {
                    el.child(
                        Button::new("sidebar.close")
                            .variant(ButtonVariant::Ghost)
                            .label("Close navigation")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.sidebar_state.mobile_open = false;
                                cx.notify();
                            })),
                    )
                })
                .child(account),
        );
        let mut body = div().flex_1().min_h_0().min_w_0().p(s * 5_f32).flex().flex_col().gap(s * 4_f32)
            .child(div().text_size(px(20.) * t.text_scale).line_height(px(28.) * t.text_scale).font_weight(FontWeight::MEDIUM).child(title.to_owned()))
            .child(div().text_color(t.colors.muted_foreground).child(match example {
                "floating" => "Project navigation on the right, inside a floating surface.",
                "mobile" => "Open navigation to try the modal sheet. Escape and outside clicks close it.",
                "loading" => "Loading placeholders keep navigation stable until data arrives.",
                _ => "Your workspace at a glance. Switch teams, browse projects, or check your account.",
            }));
        body = body.child(div().flex().gap(s * 3_f32).children(
            [("Projects", "12"), ("Members", "8")].map(|(label, value)| {
                div()
                    .flex_1()
                    .min_w_0()
                    .p(s * 4_f32)
                    .border_1()
                    .border_color(t.colors.border)
                    .rounded(t.radius.lg)
                    .child(div().text_color(t.colors.muted_foreground).child(label))
                    .child(
                        div()
                            .text_size(px(24.) * t.text_scale)
                            .line_height(px(32.) * t.text_scale)
                            .child(value),
                    )
            }),
        ));
        if example == "loading" {
            body = body.child(
                Button::new("sidebar.load")
                    .variant(ButtonVariant::Outline)
                    .child(if loading {
                        "Load navigation"
                    } else {
                        "Show loading state"
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.sidebar_loaded = !this.sidebar_loaded;
                        cx.notify();
                    })),
            );
        }
        body = body.child(
            div()
                .text_size(px(12.) * t.text_scale)
                .line_height(px(16.) * t.text_scale)
                .text_color(t.colors.muted_foreground)
                .child(self.sidebar_note.clone()),
        );
        let content = div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .h(s * 12_f32)
                    .flex_shrink_0()
                    .px(s * 3_f32)
                    .border_b_1()
                    .border_color(t.colors.border)
                    .flex()
                    .items_center()
                    .gap(s * 3_f32)
                    .when(mode != SidebarCollapsible::None, |el| {
                        el.child(sidebar_trigger("sidebar.toggle", toggle, cx))
                    })
                    .child(workspace),
            )
            .child(body);
        SidebarLayout::new(
            "sidebar.layout",
            self.sidebar_state,
            mobile,
            navigation,
            content,
            cx.listener(|this, state: &SidebarState, _, cx| {
                this.sidebar_state = *state;
                cx.notify();
            }),
        )
        .variant(variant)
        .collapsible(mode)
        .rail(true)
        .side(if right {
            SidebarSide::Right
        } else {
            SidebarSide::Left
        })
    }

    fn sidebar_mail_preview(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let search_state = live_input(
            "mail.search",
            self.sidebar_search.clone(),
            "Search mail…",
            window,
            cx,
            |this, input, event, cx| {
                if matches!(event, InputEvent::Change) {
                    this.sidebar_search = input.read(cx).value().to_string();
                    cx.notify();
                }
            },
        );
        use gpui_kit::FontWeight;
        use gpuicn::sidebar::*;
        let t = UiTheme::read(cx).clone();
        let s = t.spacing.unit;
        let mobile = self.sidebar_mobile;
        let collapsed = self
            .sidebar_state
            .icon_collapsed(mobile, SidebarCollapsible::Icon);
        let folders = [
            ("Inbox", LucideIcon::Inbox),
            ("Drafts", LucideIcon::File),
            ("Sent", LucideIcon::Send),
            ("Archive", LucideIcon::Archive),
            ("Trash", LucideIcon::Trash),
        ];
        let folder = self.sidebar_selected.min(folders.len() - 1);
        let messages = [
            (
                "William Smith",
                "Meeting tomorrow",
                "Hi team, just a reminder about our meeting tomorrow at 10 AM. We’ll review the new design and agree on the next steps.",
                "9:34 AM",
            ),
            (
                "Alice Smith",
                "Re: Project update",
                "Thanks for the update. The progress looks great so far. I’ve added a few comments to the proposal.",
                "Yesterday",
            ),
            (
                "Bob Johnson",
                "Weekend plans",
                "Hey everyone! I’m thinking of organizing a team outing this weekend. Let me know if you can make it.",
                "2 days ago",
            ),
            (
                "Emily Davis",
                "Question about the budget",
                "I’ve reviewed the budget numbers you sent over. Can we find a time to go through the details?",
                "2 days ago",
            ),
            (
                "Michael Wilson",
                "An update from the team",
                "Please join us for our next team meeting. We’ll share what we’ve shipped and what comes next.",
                "1 week ago",
            ),
            (
                "Sarah Brown",
                "Feedback on the proposal",
                "I had a chance to review the proposal. I have a few thoughts and would love to discuss them.",
                "1 week ago",
            ),
        ];
        let selected = self.sidebar_message.min(messages.len() - 1);
        let (sender, subject, message, time) = messages[selected];
        let mut rail = Sidebar::new("mail.folders", "Mail folders")
            .header(
                div()
                    .size(s * 8_f32)
                    .rounded(t.radius.lg)
                    .bg(t.colors.sidebar_primary)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        lucide(LucideIcon::Command)
                            .size(s * 4_f32)
                            .text_color(t.colors.sidebar_primary_foreground),
                    ),
            )
            .footer(
                SidebarItem::new("mail.account", "Account")
                    .collapsed(true)
                    .icon(
                        div()
                            .size(s * 8_f32)
                            .rounded(t.radius.lg)
                            .bg(t.colors.sidebar_accent)
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(12.) * t.text_scale)
                            .line_height(px(16.) * t.text_scale)
                            .child("AM"),
                    )
                    .on_activate(cx.listener(|this, _, _, cx| {
                        this.sidebar_note = "Alex Morgan · alex@example.com".into();
                        cx.notify();
                    })),
            );
        for (index, (name, icon)) in folders.iter().enumerate() {
            rail = rail.child(
                SidebarItem::new(("mail.folder", index), *name)
                    .collapsed(true)
                    .selected(folder == index)
                    .icon(
                        lucide(*icon)
                            .size(s * 4_f32)
                            .text_color(t.colors.sidebar_foreground),
                    )
                    .on_activate(cx.listener(move |this, _, _, cx| {
                        this.sidebar_selected = index;
                        this.sidebar_message = 0;
                        cx.notify();
                    })),
            );
        }
        let search = Input::new(&search_state).aria_label("Search mail");
        let mut list = Sidebar::new("mail.list", "Messages")
            .content_padding(px(0.))
            .header(
                div()
                    .flex()
                    .flex_col()
                    .gap(s * 3_f32)
                    .p(s * 2_f32)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap(s * 2_f32)
                            .child(
                                div()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(folders[folder].0),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(s * 2_f32)
                                    .child("Unread")
                                    .child(
                                        Switch::new("mail.unread")
                                            .aria_label("Show unread messages only")
                                            .checked(self.sidebar_unread)
                                            .on_change({
                                                let view = cx.entity().downgrade();
                                                move |checked, _, _, cx| {
                                                    let _ = view.update(cx, |this, cx| {
                                                        this.sidebar_unread = checked;
                                                        cx.notify();
                                                    });
                                                }
                                            }),
                                    ),
                            ),
                    )
                    .child(search),
            );
        let query = self.sidebar_search.to_lowercase();
        let mut count = 0;
        for (index, (sender, subject, body, time)) in messages.iter().enumerate() {
            if self.sidebar_unread && index % 2 != 0 {
                continue;
            }
            if !format!("{sender} {subject}")
                .to_lowercase()
                .contains(&query)
            {
                continue;
            }
            count += 1;
            list = list.child(
                Button::new(("mail.message", index))
                    .variant(ButtonVariant::Ghost)
                    .w_full()
                    .h(s * 28_f32)
                    .rounded(px(0.))
                    .p(s * 4_f32)
                    .border_0()
                    .border_b_1()
                    .border_color(t.colors.sidebar_border)
                    .when(selected == index, |el| el.bg(t.colors.sidebar_accent))
                    .child(
                        div()
                            .w_full()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap(s)
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .gap(s * 2_f32)
                                    .child(
                                        div()
                                            .truncate()
                                            .font_weight(FontWeight::NORMAL)
                                            .line_height(px(17.5) * t.text_scale)
                                            .child(*sender),
                                    )
                                    .child(
                                        div()
                                            .flex_shrink_0()
                                            .text_size(px(12.) * t.text_scale)
                                            .line_height(px(16.) * t.text_scale)
                                            .text_color(t.colors.muted_foreground)
                                            .child(*time),
                                    ),
                            )
                            .child(
                                div()
                                    .truncate()
                                    .font_weight(FontWeight::MEDIUM)
                                    .line_height(px(17.5) * t.text_scale)
                                    .child(*subject),
                            )
                            .child(
                                div()
                                    .truncate()
                                    .text_size(px(12.) * t.text_scale)
                                    .line_height(px(16.) * t.text_scale)
                                    .text_color(t.colors.muted_foreground)
                                    .child(*body),
                            ),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.sidebar_message = index;
                        this.sidebar_state.mobile_open = false;
                        cx.notify();
                    })),
            );
        }
        if count == 0 {
            list = list.child(
                div()
                    .p(s * 4_f32)
                    .text_color(t.colors.muted_foreground)
                    .child("No matching messages"),
            );
        }
        let navigation = div()
            .size_full()
            .flex()
            .child(div().w(s * 12_f32).h_full().flex_shrink_0().child(rail))
            .when(!collapsed, |el| {
                el.child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .h_full()
                        .border_l_1()
                        .border_color(t.colors.sidebar_border)
                        .child(list),
                )
            });
        let content = div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .h(s * 14_f32)
                    .px(s * 3_f32)
                    .flex()
                    .items_center()
                    .gap(s * 3_f32)
                    .border_b_1()
                    .border_color(t.colors.border)
                    .child(sidebar_trigger(
                        "mail.toggle",
                        cx.listener(move |this, _, _, cx| {
                            this.sidebar_state.toggle(mobile);
                            cx.notify();
                        }),
                        cx,
                    ))
                    .child(
                        div()
                            .text_color(t.colors.muted_foreground)
                            .child("All inboxes"),
                    )
                    .child(
                        lucide(LucideIcon::ChevronRight)
                            .size(s * 3_f32)
                            .text_color(t.colors.muted_foreground),
                    )
                    .child(folders[folder].0),
            )
            .child(
                div()
                    .id("mail.reader")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .p(s * 5_f32)
                    .child(
                        div()
                            .text_size(px(18.) * t.text_scale)
                            .line_height(px(24.) * t.text_scale)
                            .font_weight(FontWeight::MEDIUM)
                            .child(subject),
                    )
                    .child(
                        div()
                            .mt(s * 3_f32)
                            .text_size(px(12.) * t.text_scale)
                            .line_height(px(16.) * t.text_scale)
                            .text_color(t.colors.muted_foreground)
                            .child(format!("{sender} · {time}")),
                    )
                    .child(
                        div()
                            .mt(s * 6_f32)
                            .text_size(px(14.) * t.text_scale)
                            .child(message),
                    )
                    .child(
                        div()
                            .mt(s * 5_f32)
                            .text_color(t.colors.muted_foreground)
                            .child(self.sidebar_note.clone()),
                    ),
            );
        SidebarLayout::new(
            "mail.layout",
            self.sidebar_state,
            mobile,
            navigation,
            content,
            cx.listener(|this, state: &SidebarState, _, cx| {
                this.sidebar_state = *state;
                cx.notify();
            }),
        )
        .width(s * 87_f32)
        .collapsible(SidebarCollapsible::Icon)
        .rail(true)
    }

    fn sidebar_docs_preview(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let search_state = live_input(
            "docs.search",
            self.sidebar_search.clone(),
            "Search documentation…",
            window,
            cx,
            |this, input, event, cx| {
                if matches!(event, InputEvent::Change) {
                    this.sidebar_search = input.read(cx).value().to_string();
                    cx.notify();
                }
            },
        );
        use gpui_kit::FontWeight;
        use gpuicn::sidebar::*;
        let t = UiTheme::read(cx).clone();
        let s = t.spacing.unit;
        let mobile = self.sidebar_mobile;
        let groups = [
            (
                "Getting started",
                ["Introduction", "Installation", "Project structure", "CLI"],
            ),
            (
                "Build your application",
                ["Components", "Theme tokens", "Typography", "Accessibility"],
            ),
            (
                "Resources",
                ["Registry", "Examples", "Changelog", "Contributing"],
            ),
        ];
        let selected = self.sidebar_selected.min(11);
        let title = groups[selected / 4].1[selected % 4];
        let header = div()
            .flex()
            .flex_col()
            .gap(s * 2_f32)
            .child(
                div()
                    .h(s * 10_f32)
                    .px(s * 2_f32)
                    .flex()
                    .items_center()
                    .gap(s * 2_f32)
                    .child(
                        lucide(LucideIcon::BookOpen)
                            .size(s * 5_f32)
                            .text_color(t.colors.sidebar_foreground),
                    )
                    .child(div().font_weight(FontWeight::MEDIUM).child("Documentation")),
            )
            .child(Input::new(&search_state).aria_label("Search documentation pages"));
        let query = self.sidebar_search.to_lowercase();
        let mut navigation = Sidebar::new("docs.navigation", "Documentation pages")
            .header(header)
            .footer(
                div()
                    .px(s * 2_f32)
                    .py(s * 2_f32)
                    .text_size(px(12.) * t.text_scale)
                    .line_height(px(16.) * t.text_scale)
                    .text_color(t.colors.muted_foreground)
                    .child("gpuicn · Developer documentation"),
            );
        let mut count = 0;
        for (section, (label, pages)) in groups.iter().enumerate() {
            let matching: Vec<_> = pages
                .iter()
                .enumerate()
                .filter(|(_, title)| title.to_lowercase().contains(&query))
                .collect();
            if matching.is_empty() {
                continue;
            }
            count += matching.len();
            let open = self.sidebar_sections[section] || !query.is_empty();
            let mut group = sidebar_group().mb(s * 3_f32).child(
                SidebarItem::new(("docs.section", section), *label)
                    .expanded(open)
                    .trailing(
                        lucide(if open {
                            LucideIcon::ChevronDown
                        } else {
                            LucideIcon::ChevronRight
                        })
                        .size(s * 4_f32)
                        .text_color(t.colors.sidebar_foreground),
                    )
                    .on_activate(cx.listener(move |this, _, _, cx| {
                        this.sidebar_sections[section] = !this.sidebar_sections[section];
                        cx.notify();
                    })),
            );
            if open {
                let mut submenu = sidebar_menu_sub(cx);
                for (index, title) in matching {
                    let page = section * 4 + index;
                    submenu = submenu.child(
                        SidebarItem::new(("docs.page", page), *title)
                            .h(s * 7_f32)
                            .selected(selected == page)
                            .on_activate(cx.listener(move |this, _, _, cx| {
                                this.sidebar_selected = page;
                                this.sidebar_state.mobile_open = false;
                                cx.notify();
                            })),
                    );
                }
                group = group.child(submenu);
            }
            navigation = navigation.child(group);
        }
        if count == 0 {
            navigation = navigation.child(
                div()
                    .p(s * 2_f32)
                    .text_color(t.colors.muted_foreground)
                    .child("No matching pages"),
            );
        }
        let description = match selected {
            1 => {
                "Install the components you need. Their Rust source stays in your app, ready to edit."
            }
            5 => {
                "One theme controls colors, spacing, type, corners, and motion across your application."
            }
            7 => {
                "Use named controls, visible focus, and the native keyboard behavior supplied by GPUI Kit."
            }
            _ => "Build native interfaces with components that share a consistent visual language.",
        };
        let content = div().size_full().flex().flex_col()
            .child(div().h(s * 14_f32).px(s * 3_f32).flex_shrink_0().flex().items_center().gap(s * 3_f32)
                .border_b_1().border_color(t.colors.border)
                .child(sidebar_trigger("docs.toggle", cx.listener(move |this, _, _, cx| { this.sidebar_state.toggle(mobile); cx.notify(); }), cx))
                .child(div().text_size(px(12.) * t.text_scale).line_height(px(16.) * t.text_scale).text_color(t.colors.muted_foreground).child(groups[selected / 4].0)))
            .child(div().id("docs.article").flex_1().min_h_0().min_w_0().overflow_y_scroll().p(s * 6_f32)
                .child(div().text_size(px(20.) * t.text_scale).line_height(px(28.) * t.text_scale).font_weight(FontWeight::MEDIUM).child(title))
                .child(div().mt(s * 3_f32).text_size(px(14.) * t.text_scale).text_color(t.colors.muted_foreground).child(description))
                .child(div().mt(s * 6_f32).p(s * 4_f32).rounded(t.radius.lg).bg(t.colors.muted)
                    .font_family(t.fonts.mono).text_size(px(12.) * t.text_scale).line_height(px(16.) * t.text_scale).child("gpuicn add sidebar"))
                .child(div().mt(s * 6_f32).font_weight(FontWeight::MEDIUM).child("In this guide"))
                .child(div().mt(s * 3_f32).text_size(px(14.) * t.text_scale).text_color(t.colors.muted_foreground)
                    .child("Browse the sections on the left, search for a page, or hide navigation to give the article more room."))
                .child(div().mt(s * 6_f32).flex().justify_between().gap(s * 2_f32)
                    .child(Button::new("docs.previous").variant(ButtonVariant::Outline).disabled(selected == 0).label("Previous")
                        .on_click(cx.listener(|this, _, _, cx| { this.sidebar_selected = this.sidebar_selected.saturating_sub(1); cx.notify(); })))
                    .child(Button::new("docs.next").variant(ButtonVariant::Outline).disabled(selected == 11).label("Next")
                        .on_click(cx.listener(|this, _, _, cx| { this.sidebar_selected = (this.sidebar_selected + 1).min(11); cx.notify(); })))));
        SidebarLayout::new(
            "docs.layout",
            self.sidebar_state,
            mobile,
            navigation,
            content,
            cx.listener(|this, state: &SidebarState, _, cx| {
                this.sidebar_state = *state;
                cx.notify();
            }),
        )
        .collapsible(SidebarCollapsible::Offcanvas)
        .rail(true)
    }

    fn separator_preview(&self) -> impl IntoElement {
        div()
            .w_full()
            .max_w(px(320.0))
            .flex()
            .flex_col()
            .gap(px(12.0))
            .child("gpuicn")
            .child(Separator::new("preview.separator"))
            .child("Open-code components for GPUI")
            .child(
                div()
                    .flex()
                    .items_center()
                    .h(px(20.))
                    .gap(px(12.))
                    .child("Docs")
                    .child(Separator::new("preview.separator.vertical").vertical())
                    .child("Components"),
            )
    }

    fn slider_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = window.use_keyed_state("volume", cx, |_, _| {
            gpuicn::slider::SliderState::new()
                .min(0.)
                .max(100.)
                .step(1.)
                .default_value(50.)
        });
        div()
            .w(px(280.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .child("Volume")
            .child(Slider::new(&state).aria_label("Volume"))
    }

    fn switch_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let switch_view = cx.entity().downgrade();
        let row_view = cx.entity().downgrade();
        div()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(switch_row(
                Switch::new("preview.switch")
                    .checked(self.checked)
                    .aria_label("Airplane mode")
                    .on_change(move |checked, _, _, cx| {
                        switch_view
                            .update(cx, |this, cx| {
                                this.checked = checked;
                                cx.notify();
                            })
                            .ok();
                    }),
                div()
                    .id("preview.switch.interactive-label")
                    .debug_selector(|| "switch-label".into())
                    .cursor_pointer()
                    .on_click(move |_, _, cx| {
                        row_view
                            .update(cx, |this, cx| {
                                this.checked = !this.checked;
                                cx.notify();
                            })
                            .ok();
                    })
                    .child(if self.checked {
                        "Airplane mode on"
                    } else {
                        "Airplane mode"
                    }),
            ))
            .child(switch_row(
                Switch::new("preview.switch.checked")
                    .checked(true)
                    .aria_label("Notifications"),
                "Notifications",
            ))
            .child(switch_row(
                Switch::new("preview.switch.disabled")
                    .checked(true)
                    .disabled(true)
                    .aria_label("Disabled setting"),
                "Disabled",
            ))
            .child(switch_row(
                Switch::new("preview.switch.read-only")
                    .checked(true)
                    .read_only(true)
                    .aria_label("Read-only setting"),
                "Read only",
            ))
    }

    fn tabs_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let value = window.use_keyed_state("tabs", cx, |_, _| SharedString::from("account"));
        let selected = value.read(cx).clone();
        div()
            .w(px(320.))
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(
                Tabs::new("settings-tabs")
                    .aria_label("Settings")
                    .selected(selected.clone())
                    .item(Tab::new("account", "account", "Account"))
                    .item(Tab::new("password", "password", "Password"))
                    .item(Tab::new("billing", "billing", "Billing").disabled(true))
                    .on_change(move |next, _, cx| {
                        value.update(cx, |value, cx| {
                            *value = next;
                            cx.notify();
                        })
                    }),
            )
            .child(tabs_content("settings-panel", selected.clone(), cx).child(
                if selected.as_ref() == "account" {
                    "Manage your account details."
                } else {
                    "Change your account password."
                },
            ))
    }

    fn toast_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = window.use_keyed_state("notifications", cx, |_, cx| ToastState::new(cx));
        let trigger = state.clone();
        div()
            .child(
                Button::new("notify")
                    .variant(ButtonVariant::Outline)
                    .label("Show notification")
                    .on_click(move |_, _, cx| {
                        trigger.update(cx, |state, cx| {
                            state.push(
                                "saved",
                                "Changes saved",
                                "Your settings are up to date.",
                                Some(std::time::Duration::from_secs(5)),
                                cx,
                            )
                        })
                    }),
            )
            .child(state)
    }

    fn toggle_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity().downgrade();
        let icon_color = UiTheme::read(cx).colors.foreground;
        div()
            .flex()
            .items_center()
            .gap(px(8.0))
            .child(
                Toggle::new("preview.toggle")
                    .pressed(self.pressed)
                    .variant(ToggleVariant::Outline)
                    .aria_label("Bold")
                    .on_change(move |pressed, _, _, cx| {
                        view.update(cx, |this, cx| {
                            this.pressed = pressed;
                            cx.notify();
                        })
                        .ok();
                    })
                    .child(
                        lucide(LucideIcon::Bold)
                            .size(px(16.0))
                            .text_color(icon_color),
                    ),
            )
            .child(
                Toggle::new("preview.toggle.pressed")
                    .pressed(true)
                    .aria_label("Italic")
                    .child(
                        lucide(LucideIcon::Italic)
                            .size(px(16.0))
                            .text_color(icon_color),
                    ),
            )
            .child(
                Toggle::new("preview.toggle.disabled")
                    .disabled(true)
                    .aria_label("Underline")
                    .child(
                        lucide(LucideIcon::Underline)
                            .size(px(16.0))
                            .text_color(icon_color),
                    ),
            )
    }

    fn toggle_group_preview(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let value =
            window.use_keyed_state("format-values", cx, |_, _| vec![SharedString::from("bold")]);
        ToggleGroup::new("format")
            .aria_label("Text styles")
            .multiple(true)
            .value(value.read(cx).clone())
            .item(
                ToggleGroupItem::new("bold", "bold")
                    .aria_label("Bold")
                    .child("Bold"),
            )
            .item(
                ToggleGroupItem::new("italic", "italic")
                    .aria_label("Italic")
                    .child("Italic"),
            )
            .item(
                ToggleGroupItem::new("underline", "underline")
                    .aria_label("Underline")
                    .child("Underline"),
            )
            .on_change(move |next, _, cx| {
                value.update(cx, |value, cx| {
                    *value = next;
                    cx.notify();
                })
            })
    }

    fn toolbar_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let input = demo_input("toolbar.search", "", "Find…", window, cx);
        Toolbar::new("tools", "Formatting", cx)
            .button(
                ToolbarButton::new("bold", "Bold", cx)
                    .child("Bold")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.demo_action = "Bold".into();
                        cx.notify();
                    })),
            )
            .button(ToolbarButton::new("italic", "Italic", cx).child("Italic"))
            .separator()
            .input(&input, "Find text", false)
            .button(ToolbarButton::new("more", "More options", cx).child("More"))
    }

    fn tooltip_preview(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        gpui_kit::base::Button::new("tooltip-trigger")
            .accessibility_label("Add to library")
            .tooltip(|_, cx| text_tooltip("Add to library".into(), cx))
            .px(px(12.))
            .py(px(8.))
            .border_1()
            .border_color(UiTheme::read(cx).colors.border)
            .rounded(px(8.))
            .child("Hover for tooltip")
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
        let mut cx = TestAppContext::single();
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
            let mut cx = TestAppContext::single();
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
        let mut cx = TestAppContext::single();
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
        custom.text_scale = 1.1;
        custom.radius = gpuicn::theme::UiRadius::new(px(4.));
        for theme in [UiTheme::neutral_light(), UiTheme::neutral_dark(), custom] {
            for width in [360., 960.] {
                for example in ["workspace", "docs", "mail", "floating", "mobile", "loading"] {
                    let mut cx = TestAppContext::single();
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
        ];
        let mut custom = UiTheme::neutral_light();
        custom.spacing.unit = px(5.);
        custom.radius = gpuicn::theme::UiRadius::new(px(4.));
        custom.text_scale = 1.1;
        custom.colors.primary = gpui_kit::rgb(0x2563eb);
        for theme in [UiTheme::neutral_light(), UiTheme::neutral_dark(), custom] {
            for active in [false, true] {
                for name in demos {
                    let mut cx = TestAppContext::single();
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
