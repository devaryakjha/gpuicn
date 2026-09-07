use base_gpui::{alert_dialog::AlertDialogHandle, dialog::DialogHandle, drawer::DrawerHandle};
use std::borrow::Cow;

use gpui::{
    App, AppContext as _, Application, Bounds, Context, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, StatefulInteractiveElement as _, Styled as _, Window, WindowBounds,
    WindowOptions, div, prelude::FluentBuilder as _, px, size,
};
use gpui_icons::{LucideAssetSource, LucideIcon, lucide};
use gpuicn::{
    Button, ButtonSize, ButtonVariant, Checkbox, ThemeMode, UiTheme,
    accordion::{
        accordion, accordion_content, accordion_header, accordion_item, accordion_trigger,
    },
    alert_dialog::{
        alert_dialog_backdrop, alert_dialog_description, alert_dialog_footer, alert_dialog_popup,
        alert_dialog_portal, alert_dialog_root, alert_dialog_title, alert_dialog_trigger,
        alert_dialog_viewport,
    },
    autocomplete::{
        autocomplete_empty, autocomplete_input, autocomplete_item, autocomplete_list,
        autocomplete_popup, autocomplete_portal, autocomplete_positioner, autocomplete_root,
    },
    avatar::{Avatar, AvatarSize},
    checkbox_group::{CheckboxGroup, CheckboxGroupItem},
    collapsible::{collapsible, collapsible_content, collapsible_trigger},
    combobox::{
        combobox_empty, combobox_group_input, combobox_input_group, combobox_item, combobox_list,
        combobox_popup, combobox_portal, combobox_positioner, combobox_root, combobox_trigger,
    },
    context_menu::{
        context_menu_checkbox_item, context_menu_item, context_menu_popup, context_menu_portal,
        context_menu_positioner, context_menu_radio_group, context_menu_radio_item,
        context_menu_root, context_menu_separator, context_menu_trigger,
    },
    dialog::{
        dialog_backdrop, dialog_close, dialog_description, dialog_footer, dialog_popup,
        dialog_portal, dialog_root, dialog_title, dialog_trigger, dialog_viewport,
    },
    drawer::{
        DrawerSwipeDirection, drawer_backdrop, drawer_content, drawer_description, drawer_popup,
        drawer_portal, drawer_root, drawer_swipe_handle, drawer_title, drawer_viewport,
    },
    field::{
        FieldOrientation, field_control, field_description, field_error, field_label, field_root,
    },
    fieldset::{FieldsetLegendVariant, fieldset_legend, fieldset_root},
    form::{FormSubmitAction, form},
    input::Input,
    menu::{
        menu_checkbox_item, menu_group, menu_group_label, menu_item, menu_popup, menu_portal,
        menu_positioner, menu_radio_group, menu_radio_item, menu_root, menu_separator,
        menu_trigger,
    },
    menubar::{
        menubar, menubar_checkbox_item, menubar_content, menubar_item, menubar_menu,
        menubar_portal, menubar_separator, menubar_trigger,
    },
    meter::Meter,
    navigation_menu::{
        navigation_menu, navigation_menu_content, navigation_menu_item, navigation_menu_link,
        navigation_menu_list, navigation_menu_popup, navigation_menu_portal,
        navigation_menu_positioner, navigation_menu_trigger, navigation_menu_viewport,
    },
    number_field::NumberField,
    otp_field::OtpField,
    popover::{
        popover_description, popover_popup, popover_portal, popover_positioner, popover_root,
        popover_title, popover_trigger,
    },
    preview_card::{
        preview_card_popup, preview_card_portal, preview_card_positioner, preview_card_root,
        preview_card_trigger,
    },
    progress::Progress,
    radio_group::{RadioGroup, RadioItem},
    scroll_area::{
        ScrollAreaOrientation, scroll_area, scroll_area_content, scroll_area_scrollbar,
        scroll_area_thumb, scroll_area_viewport,
    },
    select::{
        select_item, select_item_text, select_list, select_popup, select_portal, select_positioner,
        select_root, select_trigger, select_value,
    },
    separator::Separator,
    slider::Slider,
    switch::Switch,
    tabs::{TabsVariant, tabs, tabs_content, tabs_list, tabs_trigger},
    toast::{ToastOptions, create_toast_manager, toast_portal, toast_provider, toast_viewport},
    toggle::{Toggle, ToggleVariant},
    toggle_group::{ToggleGroup, ToggleGroupItem},
    toolbar::{toolbar, toolbar_button, toolbar_group, toolbar_separator},
    tooltip::{
        tooltip_popup, tooltip_portal, tooltip_positioner, tooltip_provider, tooltip_root,
        tooltip_trigger,
    },
};

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
        gpui_platform::web_init();
        let handle = application().run_embedded(launch);
        std::mem::forget(handle);
    }

    #[cfg(not(target_family = "wasm"))]
    application().run(launch);
}

fn application() -> Application {
    #[cfg(target_family = "wasm")]
    let app =
        gpui_platform::application_with_web_backend(gpui_platform::WebBackendPreference::WebGpu);
    #[cfg(not(target_family = "wasm"))]
    let app = gpui_platform::application();

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
    #[cfg(target_family = "wasm")]
    {
        // WASM has no macOS target_os, so Base GPUI only registers Control
        // shortcuts. Accept Command as well for previews on macOS browsers.
        use base_gpui::primitives::input::{
            INPUT_KEY_CONTEXT, InputCopy, InputCut, InputEnd, InputHome, InputPaste, InputSelectAll,
        };
        use gpui::KeyBinding;
        cx.bind_keys([
            KeyBinding::new("cmd-a", InputSelectAll, Some(INPUT_KEY_CONTEXT)),
            KeyBinding::new("cmd-c", InputCopy, Some(INPUT_KEY_CONTEXT)),
            KeyBinding::new("cmd-v", InputPaste, Some(INPUT_KEY_CONTEXT)),
            KeyBinding::new("cmd-x", InputCut, Some(INPUT_KEY_CONTEXT)),
            KeyBinding::new("cmd-left", InputHome, Some(INPUT_KEY_CONTEXT)),
            KeyBinding::new("cmd-right", InputEnd, Some(INPUT_KEY_CONTEXT)),
        ]);
    }

    let demo = requested_value("demo")
        .and_then(|value| Demo::parse(&value))
        .unwrap_or_default();
    let mode = match requested_value("theme").as_deref() {
        Some("dark") => ThemeMode::Dark,
        _ => ThemeMode::Light,
    };
    UiTheme::switch(cx, mode);

    let bounds = Bounds::centered(
        None,
        size(
            px(requested_dimension("width", 640.0, 320.0, 1440.0)),
            px(requested_dimension("height", 288.0, 240.0, 720.0)),
        ),
        cx,
    );
    let _window = cx
        .open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |_window, cx| {
                cx.new(move |_| Showcase {
                    demo,
                    count: 0,
                    checked: false,
                    pressed: false,
                    italic: false,
                    underline: false,
                    collapsible_open: true,
                    alert_dialog_open: false,
                    dialog_open: false,
                    drawer_open: false,
                    drawer_direction: DrawerSwipeDirection::Down,
                    goal: 350,
                    icon: requested_value("icon")
                        .as_deref()
                        .and_then(LucideIcon::from_name)
                        .unwrap_or(LucideIcon::House),
                })
            },
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

struct Showcase {
    demo: Demo,
    count: usize,
    checked: bool,
    pressed: bool,
    italic: bool,
    underline: bool,
    collapsible_open: bool,
    alert_dialog_open: bool,
    dialog_open: bool,
    drawer_open: bool,
    drawer_direction: DrawerSwipeDirection,
    goal: i32,
    icon: LucideIcon,
}

impl Render for Showcase {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        div()
            .size_full()
            .overflow_hidden()
            .flex()
            .items_center()
            .justify_center()
            .p(px(16.0))
            .bg(theme.colors.background)
            .text_color(theme.colors.foreground)
            .font_family(theme.fonts.body)
            .child(self.preview(cx))
    }
}

impl Showcase {
    fn preview(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        match self.demo {
            Demo::Icons => self.icons_preview(cx).into_any_element(),
            Demo::Accordion => self.accordion_preview(cx).into_any_element(),
            Demo::AlertDialog => self.alert_dialog_preview(cx).into_any_element(),
            Demo::Autocomplete => self.autocomplete_preview(cx).into_any_element(),
            Demo::Avatar => self.avatar_preview().into_any_element(),
            Demo::Button => self.button_preview(cx).into_any_element(),
            Demo::Checkbox => self.checkbox_preview(cx).into_any_element(),
            Demo::CheckboxGroup => self.checkbox_group_preview().into_any_element(),
            Demo::Collapsible => self.collapsible_preview(cx).into_any_element(),
            Demo::Combobox => self.combobox_preview(cx).into_any_element(),
            Demo::ContextMenu => self.context_menu_preview(cx).into_any_element(),
            Demo::Dialog => self.dialog_preview(cx).into_any_element(),
            Demo::Drawer => self.drawer_preview(cx).into_any_element(),
            Demo::Field => self.field_preview(cx).into_any_element(),
            Demo::Fieldset => self.fieldset_preview(cx).into_any_element(),
            Demo::Form => self.form_preview(cx).into_any_element(),
            Demo::Input => self.input_preview().into_any_element(),
            Demo::Menu => self.menu_preview(cx).into_any_element(),
            Demo::Menubar => self.menubar_preview(cx).into_any_element(),
            Demo::Meter => self.meter_preview().into_any_element(),
            Demo::NavigationMenu => self.navigation_menu_preview(cx).into_any_element(),
            Demo::NumberField => self.number_field_preview().into_any_element(),
            Demo::OtpField => self.otp_field_preview().into_any_element(),
            Demo::Popover => self.popover_preview(cx).into_any_element(),
            Demo::PreviewCard => self.preview_card_preview(cx).into_any_element(),
            Demo::Progress => self.progress_preview().into_any_element(),
            Demo::RadioGroup => self.radio_group_preview().into_any_element(),
            Demo::ScrollArea => self.scroll_area_preview(cx).into_any_element(),
            Demo::Select => self.select_preview(cx).into_any_element(),
            Demo::Separator => self.separator_preview().into_any_element(),
            Demo::Slider => self.slider_preview().into_any_element(),
            Demo::Switch => self.switch_preview(cx).into_any_element(),
            Demo::Tabs => self.tabs_preview(cx).into_any_element(),
            Demo::Toast => self.toast_preview(cx).into_any_element(),
            Demo::Toggle => self.toggle_preview(cx).into_any_element(),
            Demo::ToggleGroup => self.toggle_group_preview(cx).into_any_element(),
            Demo::Toolbar => self.toolbar_preview(cx).into_any_element(),
            Demo::Tooltip => self.tooltip_preview(cx).into_any_element(),
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
                    .child(format!("Clicked {} times", self.count)),
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
                            .child(label)
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
                            .child("Extra small"),
                    )
                    .child(
                        Button::new("preview.button.large")
                            .size(ButtonSize::Lg)
                            .child("Large"),
                    )
                    .child(
                        Button::new("preview.button.disabled")
                            .disabled(true)
                            .child("Disabled"),
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

    fn accordion_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let items = [
            (
                "accessible",
                "Is it accessible?",
                "Use Tab, arrow keys, and Enter to navigate. See platform notes for accessibility limits.",
            ),
            (
                "styled",
                "Is it styled?",
                "Yes. It matches the rest of the shadcn visual system.",
            ),
            (
                "native",
                "Is it native?",
                "Yes. Base GPUI owns focus, keyboard input, and disclosure state.",
            ),
        ];
        accordion(cx)
            .id("preview.accordion")
            .w_full()
            .max_w(px(480.0))
            .children(items.into_iter().map(|(value, trigger, content)| {
                accordion_item(value, cx)
                    .id(value)
                    .child(
                        accordion_header().child(
                            accordion_trigger(cx)
                                .id(format!("preview.accordion.{value}.trigger"))
                                .child(trigger),
                        ),
                    )
                    .child(accordion_content(cx).child(content))
            }))
    }

    fn autocomplete_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        autocomplete_root::<&'static str>("preview.autocomplete")
            .w(px(240.0))
            .child(
                autocomplete_input("preview.autocomplete.input", cx)
                    .placeholder("Search components…")
                    .aria_label("Search components"),
            )
            .child(
                autocomplete_portal().child(
                    autocomplete_positioner().child(
                        autocomplete_popup(cx)
                            .child(
                                autocomplete_list()
                                    .child(
                                        autocomplete_item("preview.autocomplete.button", cx)
                                            .value("button")
                                            .label("Button")
                                            .child_any("Button"),
                                    )
                                    .child(
                                        autocomplete_item("preview.autocomplete.dialog", cx)
                                            .value("dialog")
                                            .label("Dialog")
                                            .child_any("Dialog"),
                                    )
                                    .child(
                                        autocomplete_item("preview.autocomplete.menu", cx)
                                            .value("menu")
                                            .label("Menu")
                                            .disabled(true)
                                            .child_any("Menu"),
                                    ),
                            )
                            .child(autocomplete_empty(cx).child("No components found.")),
                    ),
                ),
            )
    }

    fn alert_dialog_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let root_view = cx.entity().downgrade();
        let handle = AlertDialogHandle::new();
        let cancel = handle.clone();
        let confirm = handle.clone();
        alert_dialog_root("preview.alert-dialog")
            .handle(handle)
            .open(self.alert_dialog_open)
            .on_open_change(move |open, _, _, cx| {
                root_view
                    .update(cx, |this, cx| {
                        this.alert_dialog_open = open;
                        cx.notify();
                    })
                    .ok();
            })
            .child(alert_dialog_trigger("preview.alert-dialog.trigger", cx).child("Delete account"))
            .child(
                alert_dialog_portal().child(alert_dialog_backdrop()).child(
                    alert_dialog_viewport().child(
                        alert_dialog_popup("preview.alert-dialog.popup", "Confirm deletion", cx)
                            .child(
                                alert_dialog_title("preview.alert-dialog.title", cx)
                                    .mb(px(-10.0))
                                    .child("Are you sure?"),
                            )
                            .child(
                                alert_dialog_description("preview.alert-dialog.description", cx)
                                    .child("This action cannot be undone."),
                            )
                            .child_any(
                                alert_dialog_footer(cx)
                                    .child(
                                        Button::new("preview.alert-dialog.cancel")
                                            .variant(ButtonVariant::Outline)
                                            .on_click(move |_, window, cx| {
                                                cancel.close(window, cx);
                                            })
                                            .child("Cancel"),
                                    )
                                    .child(
                                        Button::new("preview.alert-dialog.action")
                                            .on_click(move |_, window, cx| {
                                                confirm.close(window, cx);
                                            })
                                            .child("Continue"),
                                    ),
                            ),
                    ),
                ),
            )
    }

    fn avatar_preview(&self) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap(px(12.0))
            .child(
                Avatar::new("preview.avatar.small")
                    .size(AvatarSize::Sm)
                    .child("AJ"),
            )
            .child(Avatar::new("preview.avatar.default").child("AJ"))
            .child(
                Avatar::new("preview.avatar.large")
                    .size(AvatarSize::Lg)
                    .child("AJ"),
            )
    }

    fn collapsible_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let collapsible_view = cx.entity().downgrade();
        div()
            .w_full()
            .max_w(px(280.0))
            .overflow_hidden()
            .rounded(px(12.0))
            .border_1()
            .border_color(theme.colors.border)
            .bg(theme.colors.popover)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(16.0))
                    .h(px(40.0))
                    .px(px(12.0))
                    .border_b_1()
                    .border_color(theme.colors.border)
                    .child("Project files"),
            )
            .child(
                div()
                    .p(px(8.0))
                    .child(
                        collapsible(cx)
                            .id("preview.collapsible.files")
                            .open(Some(self.collapsible_open))
                            .on_open_change(move |open, _, _, cx| {
                                collapsible_view
                                    .update(cx, |this, cx| {
                                        this.collapsible_open = open;
                                        cx.notify();
                                    })
                                    .ok();
                            })
                            .child(
                                collapsible_trigger(cx)
                                    .w_full()
                                    .justify_start()
                                    .gap(px(8.0))
                                    .child(
                                        lucide(if self.collapsible_open {
                                            LucideIcon::ChevronDown
                                        } else {
                                            LucideIcon::ChevronRight
                                        })
                                        .size(px(16.0))
                                        .text_color(theme.colors.muted_foreground),
                                    )
                                    .child(tree_folder_mark(&theme))
                                    .child("components"),
                            )
                            .child(
                                collapsible_content(cx)
                                    .pt(px(2.0))
                                    .pl(px(48.0))
                                    .child(tree_row("button.rs", &theme))
                                    .child(tree_row("dialog.rs", &theme))
                                    .child(tree_row("input.rs", &theme)),
                            ),
                    )
                    .child(tree_row("lib.rs", &theme).ml(px(24.0)))
                    .child(tree_row("README.md", &theme).ml(px(24.0))),
            )
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
                            .on_checked_change(move |checked, _, _, cx| {
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
                    .default_checked(true)
                    .disabled(true)
                    .aria_label("Disabled"),
                "Disabled",
            ))
            .child(checkbox_row(
                Checkbox::new("preview.checkbox.readonly")
                    .default_checked(true)
                    .read_only(true)
                    .aria_label("Read only"),
                "Read only",
            ))
    }

    fn checkbox_group_preview(&self) -> impl IntoElement {
        CheckboxGroup::new("preview.checkbox-group")
            .aria_label("Notifications")
            .default_value(["updates"])
            .all_values(["updates", "digest", "mentions"])
            .item(
                CheckboxGroupItem::new("preview.checkbox-group.updates", "updates")
                    .aria_label("Product updates")
                    .label("Product updates"),
            )
            .item(
                CheckboxGroupItem::new("preview.checkbox-group.digest", "digest")
                    .aria_label("Weekly digest")
                    .label("Weekly digest"),
            )
            .item(
                CheckboxGroupItem::new("preview.checkbox-group.mentions", "mentions")
                    .aria_label("Mentions")
                    .disabled(true)
                    .label("Mentions"),
            )
    }

    fn combobox_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        combobox_root::<&'static str>("preview.combobox")
            .item_to_string_value(|value| (*value).into())
            .w(px(240.0))
            .child(
                combobox_input_group(cx)
                    .child(
                        combobox_group_input("preview.combobox.input", cx)
                            .placeholder("Search fruits…")
                            .aria_label("Fruits"),
                    )
                    .child(combobox_trigger("preview.combobox.trigger", cx)),
            )
            .child(
                combobox_portal().child(
                    combobox_positioner().child(
                        combobox_popup(cx)
                            .child(
                                combobox_list()
                                    .child(
                                        combobox_item("preview.combobox.apple", cx)
                                            .value("apple")
                                            .label("Apple")
                                            .child_any("Apple"),
                                    )
                                    .child(
                                        combobox_item("preview.combobox.banana", cx)
                                            .value("banana")
                                            .label("Banana")
                                            .child_any("Banana"),
                                    )
                                    .child(
                                        combobox_item("preview.combobox.orange", cx)
                                            .value("orange")
                                            .label("Orange")
                                            .disabled(true)
                                            .child_any("Orange"),
                                    ),
                            )
                            .child(combobox_empty(cx).child("No fruit found.")),
                    ),
                ),
            )
    }

    fn context_menu_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let border = UiTheme::read(cx).colors.border;
        context_menu_root::<()>("preview.context-menu")
            .w_full()
            .max_w(px(280.0))
            .child(
                context_menu_trigger("preview.context-menu.trigger")
                    .w_full()
                    .h(px(144.0))
                    .px(px(24.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(8.0))
                    .border_1()
                    .border_color(border)
                    .child("Right-click here"),
            )
            .child(
                context_menu_portal().child(
                    context_menu_positioner().child(
                        context_menu_popup("preview.context-menu.popup", cx)
                            .child(
                                context_menu_item("preview.context-menu.back", cx)
                                    .label("Back")
                                    .child("Back")
                                    .child(shortcut("Cmd [", cx)),
                            )
                            .child(
                                context_menu_item("preview.context-menu.forward", cx)
                                    .label("Forward")
                                    .disabled(true)
                                    .child("Forward")
                                    .child(shortcut("Cmd ]", cx)),
                            )
                            .child(
                                context_menu_item("preview.context-menu.reload", cx)
                                    .label("Reload")
                                    .child("Reload")
                                    .child(shortcut("Cmd R", cx)),
                            )
                            .child(context_menu_separator(cx))
                            .child(
                                context_menu_checkbox_item("preview.context-menu.bookmarks", cx)
                                    .label("Show Bookmarks")
                                    .default_checked(true)
                                    .child_any("Show Bookmarks"),
                            )
                            .child(context_menu_separator(cx))
                            .child(
                                context_menu_radio_group::<(), &'static str>()
                                    .default_value(Some("pedro"))
                                    .child(
                                        context_menu_radio_item("preview.context-menu.pedro", cx)
                                            .value("pedro")
                                            .label("Pedro Duarte")
                                            .child_any("Pedro Duarte"),
                                    )
                                    .child(
                                        context_menu_radio_item("preview.context-menu.colm", cx)
                                            .value("colm")
                                            .label("Colm Tuite")
                                            .child_any("Colm Tuite"),
                                    ),
                            ),
                    ),
                ),
            )
    }

    fn dialog_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let root_view = cx.entity().downgrade();
        let handle = DialogHandle::new();
        dialog_root("preview.dialog")
            .handle(handle.clone())
            .open(self.dialog_open)
            .on_open_change(move |open, _, _, cx| {
                root_view
                    .update(cx, |this, cx| {
                        this.dialog_open = open;
                        cx.notify();
                    })
                    .ok();
            })
            .child(dialog_trigger("preview.dialog.trigger", cx).child("Open dialog"))
            .child(
                dialog_portal().child(dialog_backdrop()).child(
                    dialog_viewport().child(
                        dialog_popup("preview.dialog.popup", "Edit profile", cx)
                            .child(
                                dialog_title("preview.dialog.title", cx)
                                    .mb(px(-8.0))
                                    .child("Edit profile"),
                            )
                            .child(dialog_description("preview.dialog.description", cx).child(
                                "Make changes to your profile here. Click save when you're done.",
                            ))
                            .child_any(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(12.0))
                                    .child(dialog_field(
                                        "Name",
                                        Input::new("preview.dialog.name")
                                            .default_value("Pedro Duarte")
                                            .aria_label("Name"),
                                        cx,
                                    ))
                                    .child(dialog_field(
                                        "Username",
                                        Input::new("preview.dialog.username")
                                            .default_value("@peduarte")
                                            .aria_label("Username"),
                                        cx,
                                    )),
                            )
                            .child_any(
                                dialog_footer(cx).child(
                                    Button::new("preview.dialog.save")
                                        .on_click(move |_, window, cx| {
                                            handle.close(window, cx);
                                        })
                                        .child("Save changes"),
                                ),
                            )
                            .child(dialog_close("preview.dialog.close", cx)),
                    ),
                ),
            )
    }

    fn drawer_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let root_view = cx.entity().downgrade();
        let handle = DrawerHandle::new();
        let close = handle.clone();
        let submit = handle.clone();
        let minus_view = cx.entity().downgrade();
        let plus_view = cx.entity().downgrade();
        let top_view = cx.entity().downgrade();
        let right_view = cx.entity().downgrade();
        let bottom_view = cx.entity().downgrade();
        let left_view = cx.entity().downgrade();
        let direction = self.drawer_direction;
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(12.0))
            .child(
                div()
                    .text_size(px(14.0))
                    .text_color(UiTheme::read(cx).colors.muted_foreground)
                    .child("Open from an edge"),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .justify_center()
                    .gap(px(8.0))
                    .child(drawer_direction_button(
                        "preview.drawer.top",
                        "Top",
                        DrawerSwipeDirection::Up,
                        top_view,
                        handle.clone(),
                    ))
                    .child(drawer_direction_button(
                        "preview.drawer.right",
                        "Right",
                        DrawerSwipeDirection::Right,
                        right_view,
                        handle.clone(),
                    ))
                    .child(drawer_direction_button(
                        "preview.drawer.bottom",
                        "Bottom",
                        DrawerSwipeDirection::Down,
                        bottom_view,
                        handle.clone(),
                    ))
                    .child(drawer_direction_button(
                        "preview.drawer.left",
                        "Left",
                        DrawerSwipeDirection::Left,
                        left_view,
                        handle.clone(),
                    )),
            )
            .child(
                drawer_root("preview.drawer")
                    .handle(handle)
                    .open(self.drawer_open)
                    .swipe_direction(direction)
                    .on_open_change(move |open, _, _, cx| {
                        root_view
                            .update(cx, |this, cx| {
                                this.drawer_open = open;
                                cx.notify();
                            })
                            .ok();
                    })
                    .child(
                        drawer_portal().child(drawer_backdrop()).child(
                            drawer_viewport().child(
                                drawer_popup("preview.drawer.popup", "Activity goal", cx).child(
                                    drawer_content(cx)
                                        .when(direction == DrawerSwipeDirection::Down, |content| {
                                            content.child(drawer_swipe_handle(cx))
                                        })
                                        .child(
                                            div()
                                                .w_full()
                                                .max_w(px(360.))
                                                .mx_auto()
                                                .p(px(20.0))
                                                .flex()
                                                .flex_col()
                                                .gap(px(16.0))
                                                .child(
                                                    drawer_title("preview.drawer.title", cx)
                                                        .child("Move goal"),
                                                )
                                                .child(
                                                    drawer_description(
                                                        "preview.drawer.description",
                                                        cx,
                                                    )
                                                    .child("Set your daily activity goal."),
                                                )
                                                .child(
                                                    div()
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .gap(px(16.0))
                                                        .child(
                                                            Button::new("preview.drawer.minus")
                                                                .aria_label("Decrease daily goal")
                                                                .variant(ButtonVariant::Outline)
                                                                .size(ButtonSize::Icon)
                                                                .on_click(move |_, _, cx| {
                                                                    minus_view
                                                                        .update(cx, |this, cx| {
                                                                            this.goal = (this.goal
                                                                                - 10)
                                                                                .max(0);
                                                                            cx.notify();
                                                                        })
                                                                        .ok();
                                                                })
                                                                .child(
                                                                    lucide(LucideIcon::Minus)
                                                                        .size(px(16.0))
                                                                        .text_color(
                                                                            UiTheme::read(cx)
                                                                                .colors
                                                                                .foreground,
                                                                        ),
                                                                ),
                                                        )
                                                        .child(
                                                            div()
                                                                .w(px(72.0))
                                                                .text_center()
                                                                .text_size(px(24.0))
                                                                .child(self.goal.to_string()),
                                                        )
                                                        .child(
                                                            Button::new("preview.drawer.plus")
                                                                .aria_label("Increase daily goal")
                                                                .variant(ButtonVariant::Outline)
                                                                .size(ButtonSize::Icon)
                                                                .on_click(move |_, _, cx| {
                                                                    plus_view
                                                                        .update(cx, |this, cx| {
                                                                            this.goal += 10;
                                                                            cx.notify();
                                                                        })
                                                                        .ok();
                                                                })
                                                                .child(
                                                                    lucide(LucideIcon::Plus)
                                                                        .size(px(16.0))
                                                                        .text_color(
                                                                            UiTheme::read(cx)
                                                                                .colors
                                                                                .foreground,
                                                                        ),
                                                                ),
                                                        ),
                                                )
                                                .child(
                                                    div()
                                                        .flex()
                                                        .gap(px(8.0))
                                                        .child(
                                                            Button::new("preview.drawer.submit")
                                                                .on_click(move |_, window, cx| {
                                                                    submit.close(window, cx);
                                                                })
                                                                .child("Submit"),
                                                        )
                                                        .child(
                                                            Button::new("preview.drawer.close")
                                                                .variant(ButtonVariant::Outline)
                                                                .on_click(move |_, window, cx| {
                                                                    close.close(window, cx);
                                                                })
                                                                .child("Cancel"),
                                                        ),
                                                ),
                                        ),
                                ),
                            ),
                        ),
                    ),
            )
    }

    fn field_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        field_root("preview.field", FieldOrientation::Vertical, cx)
            .w_full()
            .max_w(px(280.0))
            .name("username")
            .child(field_label(cx).text("Username"))
            .child(field_control("preview.field.control", cx).placeholder("e.g. ada"))
            .child(field_description(cx).child("Visible on your public profile."))
    }

    fn fieldset_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        fieldset_root("preview.fieldset", cx)
            .w_full()
            .max_w(px(300.0))
            .aria_label("Shipping address")
            .child(fieldset_legend(FieldsetLegendVariant::Legend, cx).child("Shipping address"))
            .child_any(
                field_root("preview.fieldset.name", FieldOrientation::Vertical, cx)
                    .name("name")
                    .child(field_label(cx).text("Full name"))
                    .child(
                        field_control("preview.fieldset.name.control", cx)
                            .placeholder("Ada Lovelace"),
                    ),
            )
            .child_any(
                field_root("preview.fieldset.city", FieldOrientation::Vertical, cx)
                    .name("city")
                    .child(field_label(cx).text("City"))
                    .child(
                        field_control("preview.fieldset.city.control", cx).placeholder("London"),
                    ),
            )
    }

    fn form_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity().downgrade();
        form("preview.form", cx)
            .w_full()
            .max_w(px(280.0))
            .aria_label("Subscribe")
            .on_form_submit(move |_, _, _, cx| {
                view.update(cx, |this, cx| {
                    this.count += 1;
                    cx.notify();
                })
                .ok();
            })
            .child(
                field_root("preview.form.email", FieldOrientation::Vertical, cx)
                    .name("email")
                    .child(field_label(cx).text("Email"))
                    .child(
                        field_control("preview.form.email.control", cx)
                            .required(true)
                            .placeholder("you@example.com"),
                    )
                    .child(field_error(cx).child("Enter your email address.")),
            )
            .child(
                Button::new("preview.form.submit")
                    .on_click(|_, window, cx| {
                        window.dispatch_action(Box::new(FormSubmitAction), cx)
                    })
                    .child("Subscribe"),
            )
            .when(self.count > 0, |form| form.child("Form submitted."))
    }

    fn input_preview(&self) -> impl IntoElement {
        div()
            .w_full()
            .max_w(px(320.0))
            .flex()
            .flex_col()
            .gap(px(12.0))
            .child(
                Input::new("preview.input")
                    .placeholder("Email")
                    .aria_label("Email address"),
            )
            .child(
                Input::new("preview.input.read-only")
                    .default_value("read-only@example.com")
                    .read_only(true)
                    .aria_label("Read-only email address"),
            )
            .child(
                Input::new("preview.input.disabled")
                    .default_value("disabled@example.com")
                    .disabled(true)
                    .aria_label("Disabled email address"),
            )
    }

    fn menu_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        menu_root::<()>("preview.menu")
            .child(menu_trigger("preview.menu.trigger", cx).child("Open menu"))
            .child(
                menu_portal().child(
                    menu_positioner().child(
                        menu_popup("preview.menu.popup", cx)
                            .child(
                                menu_group()
                                    .child(
                                        menu_group_label(cx)
                                            .label("My Account")
                                            .child("My Account"),
                                    )
                                    .child(
                                        menu_item("preview.menu.profile", cx)
                                            .label("Profile")
                                            .child("Profile")
                                            .child(shortcut("Shift Cmd P", cx)),
                                    )
                                    .child(
                                        menu_item("preview.menu.billing", cx)
                                            .label("Billing")
                                            .child("Billing")
                                            .child(shortcut("Cmd B", cx)),
                                    )
                                    .child(
                                        menu_item("preview.menu.settings", cx)
                                            .label("Settings")
                                            .disabled(true)
                                            .child("Settings")
                                            .child(shortcut("Cmd ,", cx)),
                                    ),
                            )
                            .child(menu_separator(cx))
                            .child(
                                menu_checkbox_item("preview.menu.status", cx)
                                    .label("Show Status Bar")
                                    .default_checked(true)
                                    .child_any("Show Status Bar"),
                            )
                            .child(
                                menu_checkbox_item("preview.menu.activity", cx)
                                    .label("Show Activity Bar")
                                    .child_any("Show Activity Bar"),
                            )
                            .child(menu_separator(cx))
                            .child(
                                menu_radio_group::<(), &'static str>()
                                    .default_value(Some("pedro"))
                                    .child(
                                        menu_radio_item("preview.menu.pedro", cx)
                                            .value("pedro")
                                            .label("Pedro Duarte")
                                            .child_any("Pedro Duarte"),
                                    )
                                    .child(
                                        menu_radio_item("preview.menu.colm", cx)
                                            .value("colm")
                                            .label("Colm Tuite")
                                            .child_any("Colm Tuite"),
                                    ),
                            ),
                    ),
                ),
            )
    }

    fn menubar_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        menubar("preview.menubar", cx)
            .aria_label("Application menu")
            .child(
                menubar_menu::<()>("preview.menubar.file")
                    .child(menubar_trigger("preview.menubar.file.trigger", cx).child("File"))
                    .child(
                        menubar_portal().child(
                            menu_positioner().child(
                                menubar_content("preview.menubar.file.content", cx)
                                    .child(
                                        menubar_item("preview.menubar.new", cx)
                                            .label("New File")
                                            .child("New File")
                                            .child(shortcut("Cmd N", cx)),
                                    )
                                    .child(
                                        menubar_item("preview.menubar.open", cx)
                                            .label("Open")
                                            .child("Open…")
                                            .child(shortcut("Cmd O", cx)),
                                    )
                                    .child(
                                        menubar_item("preview.menubar.save", cx)
                                            .label("Save")
                                            .disabled(true)
                                            .child("Save")
                                            .child(shortcut("Cmd S", cx)),
                                    )
                                    .child(menubar_separator(cx))
                                    .child(
                                        menubar_checkbox_item("preview.menubar.autosave", cx)
                                            .label("Auto Save")
                                            .default_checked(true)
                                            .child_any("Auto Save"),
                                    ),
                            ),
                        ),
                    ),
            )
            .child(
                menubar_menu::<()>("preview.menubar.edit")
                    .child(menubar_trigger("preview.menubar.edit.trigger", cx).child("Edit"))
                    .child(
                        menubar_portal().child(
                            menu_positioner().child(
                                menubar_content("preview.menubar.edit.content", cx)
                                    .child(
                                        menubar_item("preview.menubar.undo", cx)
                                            .label("Undo")
                                            .child("Undo"),
                                    )
                                    .child(
                                        menubar_item("preview.menubar.redo", cx)
                                            .label("Redo")
                                            .child("Redo"),
                                    ),
                            ),
                        ),
                    ),
            )
    }

    fn meter_preview(&self) -> impl IntoElement {
        div()
            .w_full()
            .max_w(px(320.0))
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child("Storage used")
            .child(
                Meter::new("preview.meter")
                    .value(68.0)
                    .aria_label("Storage used: 68 percent"),
            )
    }

    fn number_field_preview(&self) -> impl IntoElement {
        div()
            .w(px(160.0))
            .flex()
            .flex_col()
            .gap(px(12.0))
            .child(
                NumberField::new("preview.number-field")
                    .default_value(4.0)
                    .range(Some(0.0), Some(10.0)),
            )
            .child(
                NumberField::new("preview.number-field.read-only")
                    .default_value(6.0)
                    .read_only(true),
            )
            .child(
                NumberField::new("preview.number-field.disabled")
                    .default_value(8.0)
                    .disabled(true),
            )
    }

    fn navigation_menu_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        navigation_menu::<&'static str>(cx)
            .id("preview.navigation-menu")
            .aria_label("Main navigation")
            .child(
                navigation_menu_list()
                    .child(
                        navigation_menu_item()
                            .value("docs")
                            .child(navigation_menu_trigger(cx).child_any("Docs"))
                            .child(
                                navigation_menu_content(cx)
                                    .w(px(200.0))
                                    .flex()
                                    .flex_col()
                                    .child(
                                        div().id("preview.navigation-menu.getting-started").child(
                                            navigation_menu_link::<&str>(cx)
                                                .child("Getting started"),
                                        ),
                                    )
                                    .child(div().id("preview.navigation-menu.components").child(
                                        navigation_menu_link::<&str>(cx).child("Components"),
                                    ))
                                    .child(
                                        div().id("preview.navigation-menu.theming").child(
                                            navigation_menu_link::<&str>(cx).child("Theming"),
                                        ),
                                    ),
                            ),
                    )
                    .child(navigation_menu_link(cx).child("Blog"))
                    .child(navigation_menu_link(cx).child("About")),
            )
            .child(
                navigation_menu_portal().child(
                    navigation_menu_positioner()
                        .child(navigation_menu_popup(cx).child(navigation_menu_viewport(cx))),
                ),
            )
    }

    fn otp_field_preview(&self) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(16.0))
            .child(
                OtpField::new("preview.otp-field", 6)
                    .default_value("123")
                    .aria_label("Verification code"),
            )
            .child(
                OtpField::new("preview.otp-field.disabled", 6)
                    .default_value("123456")
                    .disabled(true)
                    .aria_label("Disabled verification code"),
            )
            .child(
                OtpField::new("preview.otp-field.read-only", 6)
                    .default_value("654321")
                    .read_only(true)
                    .aria_label("Read-only verification code"),
            )
    }

    fn popover_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        popover_root("preview.popover")
            .child(popover_trigger("preview.popover.trigger", cx).child("Open popover"))
            .child(
                popover_portal().child(
                    popover_positioner().child(
                        popover_popup("preview.popover.popup", "Dimensions", cx)
                            .child(popover_title(cx).child("Dimensions"))
                            .child(
                                popover_description(cx).child("Set the dimensions for the layer."),
                            )
                            .child_any(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(8.0))
                                    .child(popover_field(
                                        "Width",
                                        Input::new("preview.popover.width")
                                            .default_value("100%")
                                            .aria_label("Width"),
                                        cx,
                                    ))
                                    .child(popover_field(
                                        "Height",
                                        Input::new("preview.popover.height")
                                            .default_value("25px")
                                            .aria_label("Height"),
                                        cx,
                                    )),
                            ),
                    ),
                ),
            )
    }

    fn preview_card_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        preview_card_root("preview.preview-card")
            .child(preview_card_trigger("preview.preview-card.trigger").child("@gpuicn"))
            .child(
                preview_card_portal().child(
                    preview_card_positioner().child(
                        preview_card_popup("preview.preview-card.popup", cx).child_any(
                            div()
                                .flex()
                                .w_full()
                                .gap(px(12.0))
                                .child(Avatar::new("preview.preview-card.avatar").child("CN"))
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .flex_1()
                                        .min_w_0()
                                        .gap(px(4.0))
                                        .child(
                                            div()
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .child("@gpuicn"),
                                        )
                                        .child(
                                            "Open-code shadcn visual ports for native GPUI apps.",
                                        )
                                        .child(
                                            div()
                                                .mt(px(4.0))
                                                .text_color(
                                                    UiTheme::read(cx).colors.muted_foreground,
                                                )
                                                .child("Joined August 2026"),
                                        ),
                                ),
                        ),
                    ),
                ),
            )
    }

    fn progress_preview(&self) -> impl IntoElement {
        div()
            .w_full()
            .max_w(px(320.0))
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(
                Progress::new("preview.progress")
                    .value(64.0)
                    .label("Uploading…"),
            )
            .child(
                Progress::new("preview.progress.indeterminate")
                    .indeterminate()
                    .label("Waiting…"),
            )
    }

    fn radio_group_preview(&self) -> impl IntoElement {
        RadioGroup::new("preview.radio-group")
            .aria_label("Interface density")
            .default_value("comfortable")
            .item(
                RadioItem::new("preview.radio-group.compact", "compact")
                    .aria_label("Compact")
                    .label("Compact"),
            )
            .item(
                RadioItem::new("preview.radio-group.comfortable", "comfortable")
                    .aria_label("Comfortable")
                    .label("Comfortable"),
            )
            .item(
                RadioItem::new("preview.radio-group.spacious", "spacious")
                    .aria_label("Spacious")
                    .label("Spacious")
                    .disabled(true),
            )
    }

    fn scroll_area_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        scroll_area(cx)
            .id("preview.scroll-area")
            .w_full()
            .max_w(px(300.0))
            .h(px(180.0))
            .rounded(px(8.0))
            .border_1()
            .border_color(theme.colors.border)
            .child(
                scroll_area_viewport(cx)
                    .id("preview.scroll-area.viewport")
                    .aria_label("Changelog entries")
                    .child(
                        scroll_area_content(cx)
                            .flex()
                            .flex_col()
                            .children((1..=16).map(|number| {
                                div()
                                    .px(px(12.0))
                                    .py(px(6.0))
                                    .child(format!("v0.{number:02} — patch notes"))
                            })),
                    ),
            )
            .child(
                scroll_area_scrollbar(ScrollAreaOrientation::Vertical, cx)
                    .id("preview.scroll-area.scrollbar")
                    .child(scroll_area_thumb(cx)),
            )
    }

    fn select_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        select_root::<&'static str>("preview.select")
            .default_value(Some("system"))
            .item_to_string_value(|value| (*value).into())
            .w(px(200.0))
            .child(
                select_trigger("preview.select.trigger", cx)
                    .aria_label("Theme")
                    .child(select_value(cx).placeholder("Theme")),
            )
            .child(
                select_portal().child(
                    select_positioner().child(
                        select_popup(cx).child(
                            select_list()
                                .child(
                                    select_item("preview.select.system", cx)
                                        .value("system")
                                        .label("System")
                                        .child(select_item_text().text("System")),
                                )
                                .child(
                                    select_item("preview.select.light", cx)
                                        .value("light")
                                        .label("Light")
                                        .child(select_item_text().text("Light")),
                                )
                                .child(
                                    select_item("preview.select.dark", cx)
                                        .value("dark")
                                        .label("Dark")
                                        .disabled(true)
                                        .child(select_item_text().text("Dark")),
                                ),
                        ),
                    ),
                ),
            )
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
    }

    fn slider_preview(&self) -> impl IntoElement {
        div()
            .w_full()
            .max_w(px(320.0))
            .flex()
            .flex_col()
            .gap(px(20.0))
            .child(
                Slider::new("preview.slider")
                    .default_value(48.0)
                    .aria_label("Volume"),
            )
            .child(
                Slider::new("preview.slider.disabled")
                    .default_value(64.0)
                    .disabled(true)
                    .aria_label("Disabled volume"),
            )
    }

    fn switch_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let switch_view = cx.entity().downgrade();
        let row_view = cx.entity().downgrade();
        div()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(
                switch_row(
                    Switch::new("preview.switch")
                        .checked(self.checked)
                        .aria_label("Airplane mode")
                        .on_checked_change(move |checked, _, _, cx| {
                            switch_view
                                .update(cx, |this, cx| {
                                    this.checked = checked;
                                    cx.notify();
                                })
                                .ok();
                        }),
                    if self.checked {
                        "Airplane mode on"
                    } else {
                        "Airplane mode"
                    },
                )
                .id("preview.switch.interactive-row")
                .cursor_pointer()
                .on_click(move |_, _, cx| {
                    row_view
                        .update(cx, |this, cx| {
                            this.checked = !this.checked;
                            cx.notify();
                        })
                        .ok();
                }),
            )
            .child(switch_row(
                Switch::new("preview.switch.checked")
                    .default_checked(true)
                    .aria_label("Notifications"),
                "Notifications",
            ))
            .child(switch_row(
                Switch::new("preview.switch.disabled")
                    .default_checked(true)
                    .disabled(true)
                    .aria_label("Disabled setting"),
                "Disabled",
            ))
            .child(switch_row(
                Switch::new("preview.switch.read-only")
                    .default_checked(true)
                    .read_only(true)
                    .aria_label("Read-only setting"),
                "Read only",
            ))
    }

    fn tabs_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        tabs(cx)
            .id("preview.tabs")
            .w_full()
            .max_w(px(420.0))
            .default_value(Some("account"))
            .child(
                tabs_list(cx)
                    .child(
                        tabs_trigger(TabsVariant::Default, cx)
                            .id("preview.tabs.account")
                            .value("account")
                            .child("Account"),
                    )
                    .child(
                        tabs_trigger(TabsVariant::Default, cx)
                            .id("preview.tabs.password")
                            .value("password")
                            .child("Password"),
                    ),
            )
            .child(
                tabs_content(cx)
                    .value("account")
                    .pt(px(12.0))
                    .child("Make changes to your account here."),
            )
            .child(
                tabs_content(cx)
                    .value("password")
                    .pt(px(12.0))
                    .child("Change your password here."),
            )
    }

    fn toast_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let manager = create_toast_manager::<()>();
        let add_toast = manager.clone();
        toast_provider("preview.toast")
            .manager(manager)
            .child_any(
                Button::new("preview.toast.trigger")
                    .on_click(move |_, _, cx| {
                        add_toast.add(
                            ToastOptions::new()
                                .title("Draft saved")
                                .description("All changes synced to your workspace."),
                            cx,
                        );
                    })
                    .child("Show toast"),
            )
            .child(toast_portal().child(toast_viewport("preview.toast.viewport", cx)))
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
                    .on_pressed_change(move |pressed, _, _, cx| {
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
                    .default_pressed(true)
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

    fn toggle_group_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let icon_color = UiTheme::read(cx).colors.foreground;
        ToggleGroup::new("preview.toggle-group")
            .aria_label("Text alignment")
            .default_value(["left"])
            .item(
                ToggleGroupItem::new("preview.toggle-group.left", "left")
                    .aria_label("Align left")
                    .child(
                        lucide(LucideIcon::TextAlignStart)
                            .size(px(16.0))
                            .text_color(icon_color),
                    ),
            )
            .item(
                ToggleGroupItem::new("preview.toggle-group.center", "center")
                    .aria_label("Align center")
                    .child(
                        lucide(LucideIcon::TextAlignCenter)
                            .size(px(16.0))
                            .text_color(icon_color),
                    ),
            )
            .item(
                ToggleGroupItem::new("preview.toggle-group.right", "right")
                    .aria_label("Align right")
                    .disabled(true)
                    .child(
                        lucide(LucideIcon::TextAlignEnd)
                            .size(px(16.0))
                            .text_color(icon_color),
                    ),
            )
    }

    fn toolbar_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let icon_color = theme.colors.foreground;
        let bold_view = cx.entity().downgrade();
        let italic_view = cx.entity().downgrade();
        let underline_view = cx.entity().downgrade();
        let copy_view = cx.entity().downgrade();
        toolbar(cx)
            .id("preview.toolbar")
            .aria_label("Formatting")
            .child(
                toolbar_group(cx)
                    .id("preview.toolbar.format")
                    .aria_label("Text style")
                    .child(
                        toolbar_button(cx)
                            .id("preview.toolbar.bold")
                            .aria_label("Bold")
                            .when(self.pressed, |button| button.bg(theme.colors.muted))
                            .on_click(move |_, _, cx| {
                                bold_view
                                    .update(cx, |this, cx| {
                                        this.pressed = !this.pressed;
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
                        toolbar_button(cx)
                            .id("preview.toolbar.italic")
                            .aria_label("Italic")
                            .when(self.italic, |button| button.bg(theme.colors.muted))
                            .on_click(move |_, _, cx| {
                                italic_view
                                    .update(cx, |this, cx| {
                                        this.italic = !this.italic;
                                        cx.notify();
                                    })
                                    .ok();
                            })
                            .child(
                                lucide(LucideIcon::Italic)
                                    .size(px(16.0))
                                    .text_color(icon_color),
                            ),
                    )
                    .child(
                        toolbar_button(cx)
                            .id("preview.toolbar.underline")
                            .aria_label("Underline")
                            .when(self.underline, |button| button.bg(theme.colors.muted))
                            .on_click(move |_, _, cx| {
                                underline_view
                                    .update(cx, |this, cx| {
                                        this.underline = !this.underline;
                                        cx.notify();
                                    })
                                    .ok();
                            })
                            .child(
                                lucide(LucideIcon::Underline)
                                    .size(px(16.0))
                                    .text_color(icon_color),
                            ),
                    ),
            )
            .child(toolbar_separator(cx).h(px(16.0)).w(px(1.0)))
            .child(
                toolbar_button(cx)
                    .id("preview.toolbar.copy")
                    .aria_label("Copy")
                    .on_click(move |_, _, cx| {
                        copy_view
                            .update(cx, |this, cx| {
                                this.count += 1;
                                cx.notify();
                            })
                            .ok();
                    })
                    .child(if self.count > 0 { "Copied" } else { "Copy" }),
            )
    }

    fn tooltip_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        tooltip_provider("preview.tooltip-provider").child(
            tooltip_root("preview.tooltip")
                .child(
                    tooltip_trigger("preview.tooltip.trigger").child(
                        Button::new("preview.tooltip.button")
                            .variant(ButtonVariant::Outline)
                            .child("Hover me"),
                    ),
                )
                .child(tooltip_portal().child(tooltip_positioner().child(
                    tooltip_popup("preview.tooltip.popup", cx).child_any("Add to library"),
                ))),
        )
    }
}

fn tree_row(name: &'static str, theme: &UiTheme) -> gpui::Div {
    div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .h(px(30.0))
        .px(px(10.0))
        .text_size(px(14.0))
        .child(
            lucide(LucideIcon::File)
                .size(px(14.))
                .text_color(theme.colors.muted_foreground),
        )
        .child(name)
}

fn tree_folder_mark(theme: &UiTheme) -> gpui::Svg {
    lucide(LucideIcon::Folder)
        .size(px(14.))
        .text_color(theme.colors.muted_foreground)
}

fn drawer_direction_button(
    id: &'static str,
    label: &'static str,
    direction: DrawerSwipeDirection,
    view: gpui::WeakEntity<Showcase>,
    handle: DrawerHandle<()>,
) -> Button {
    Button::new(id)
        .variant(ButtonVariant::Outline)
        .on_click(move |_, window, cx| {
            view.update(cx, |this, cx| {
                this.drawer_direction = direction;
                cx.notify();
            })
            .ok();
            handle.open_with_payload((), window, cx);
        })
        .child(label)
}

fn shortcut(value: &'static str, cx: &App) -> gpui::Div {
    div()
        .ml_auto()
        .font_family(UiTheme::read(cx).fonts.mono.clone())
        .text_size(px(12.0))
        .text_color(UiTheme::read(cx).colors.muted_foreground)
        .child(value)
}

fn dialog_field(label: &'static str, input: Input, cx: &App) -> gpui::Div {
    div()
        .flex()
        .items_center()
        .gap(px(12.0))
        .child(
            div()
                .w(px(80.0))
                .text_right()
                .text_size(px(14.0))
                .text_color(UiTheme::read(cx).colors.foreground)
                .child(label),
        )
        .child(div().flex_1().min_w(px(0.)).child(input))
}

fn popover_field(label: &'static str, input: Input, cx: &App) -> gpui::Div {
    div()
        .grid()
        .grid_cols(3)
        .items_center()
        .gap(px(8.0))
        .child(
            div()
                .text_size(px(14.0))
                .text_color(UiTheme::read(cx).colors.foreground)
                .child(label),
        )
        .child(div().col_span(2).child(input))
}

fn switch_row(switch: Switch, label: &'static str) -> gpui::Div {
    div()
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(switch)
        .child(label)
}

fn checkbox_row(checkbox: Checkbox, label: &'static str) -> gpui::Div {
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
