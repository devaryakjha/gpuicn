use std::borrow::Cow;

use gpui::{
    App, AppContext as _, Application, Bounds, Context, IntoElement, ParentElement as _, Render,
    Styled as _, Window, WindowBounds, WindowOptions, div, px, size,
};
use gpui_icons::LucideAssetSource;
use gpuicn::{
    Button, ButtonSize, ButtonVariant, Checkbox, ThemeMode, UiTheme,
    accordion::{
        accordion, accordion_content, accordion_header, accordion_item, accordion_trigger,
    },
    alert_dialog::{
        alert_dialog_action, alert_dialog_backdrop, alert_dialog_cancel, alert_dialog_description,
        alert_dialog_popup, alert_dialog_portal, alert_dialog_root, alert_dialog_title,
        alert_dialog_trigger, alert_dialog_viewport,
    },
    autocomplete::{
        autocomplete_empty, autocomplete_input, autocomplete_item, autocomplete_list,
        autocomplete_popup, autocomplete_portal, autocomplete_positioner, autocomplete_root,
    },
    avatar::{Avatar, AvatarSize},
    checkbox_group::{CheckboxGroup, CheckboxGroupItem},
    collapsible::{collapsible, collapsible_content, collapsible_trigger},
    combobox::{
        combobox_empty, combobox_input, combobox_input_group, combobox_item, combobox_list,
        combobox_popup, combobox_portal, combobox_positioner, combobox_root, combobox_trigger,
    },
    context_menu::{
        context_menu_item, context_menu_popup, context_menu_portal, context_menu_positioner,
        context_menu_root, context_menu_trigger,
    },
    dialog::{
        dialog_backdrop, dialog_close, dialog_description, dialog_popup, dialog_portal,
        dialog_root, dialog_title, dialog_trigger, dialog_viewport,
    },
    drawer::{
        drawer_backdrop, drawer_close, drawer_content, drawer_description, drawer_popup,
        drawer_portal, drawer_root, drawer_swipe_handle, drawer_title, drawer_trigger,
        drawer_viewport,
    },
    field::{FieldOrientation, field_control, field_description, field_label, field_root},
    fieldset::{FieldsetLegendVariant, fieldset_legend, fieldset_root},
    form::{FormSubmitAction, form},
    input::Input,
    menu::{menu_item, menu_popup, menu_portal, menu_positioner, menu_root, menu_trigger},
    menubar::{
        menubar, menubar_content, menubar_item, menubar_menu, menubar_portal, menubar_trigger,
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
        popover_close, popover_popup, popover_portal, popover_positioner, popover_root,
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
        select_icon, select_item, select_item_text, select_list, select_popup, select_portal,
        select_positioner, select_root, select_trigger, select_value,
    },
    separator::Separator,
    slider::Slider,
    switch::Switch,
    tabs::{TabsVariant, tabs, tabs_content, tabs_list, tabs_trigger},
    toast::{
        ToastClose, ToastContent, ToastDescription, ToastOptions, ToastRoot, ToastTitle,
        create_toast_manager, toast_portal, toast_provider, toast_viewport,
    },
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
    inline_js = "export function previewReady(){let sent=false;const send=()=>{if(sent)return;sent=true;document.documentElement.dataset.gpuicnReady='true';window.parent.postMessage({gpuicn:'preview-ready'},'*')};requestAnimationFrame(()=>requestAnimationFrame(send));setTimeout(send,1000)}"
)]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = previewReady)]
    fn preview_ready();
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

    let demo = requested_value("demo")
        .and_then(|value| Demo::parse(&value))
        .unwrap_or_default();
    let mode = match requested_value("theme").as_deref() {
        Some("dark") => ThemeMode::Dark,
        _ => ThemeMode::Light,
    };
    UiTheme::switch(cx, mode);

    let bounds = Bounds::centered(None, size(px(720.0), px(480.0)), cx);
    cx.open_window(
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
            })
        },
    )
    .expect("failed to open showcase window");
    #[cfg(target_family = "wasm")]
    preview_ready();
    #[cfg(target_family = "wasm")]
    cx.activate(true);
    #[cfg(not(target_family = "wasm"))]
    if std::env::var_os("IMAJHA_SHOWCASE_BACKGROUND").is_none() {
        cx.activate(true);
    }
}

#[derive(Clone, Copy, Default)]
enum Demo {
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
}

impl Render for Showcase {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(theme.colors.background)
            .text_color(theme.colors.foreground)
            .font_family(theme.fonts.body)
            .child(self.preview(cx))
    }
}

impl Showcase {
    fn preview(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        match self.demo {
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
            Demo::ToggleGroup => self.toggle_group_preview().into_any_element(),
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

    fn accordion_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        accordion(cx).child(
            accordion_item("first", cx)
                .child(accordion_header().child(accordion_trigger(cx).child("Is it styled?")))
                .child(
                    accordion_content(cx).child("Yes. This uses the Base GPUI disclosure runtime."),
                ),
        )
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
                                            .child_any("Menu"),
                                    ),
                            )
                            .child(autocomplete_empty(cx).child("No components found.")),
                    ),
                ),
            )
    }

    fn alert_dialog_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        alert_dialog_root("preview.alert-dialog")
            .child(alert_dialog_trigger("preview.alert-dialog.trigger", cx).child("Delete account"))
            .child(
                alert_dialog_portal().child(alert_dialog_backdrop()).child(
                    alert_dialog_viewport().child(
                        alert_dialog_popup("preview.alert-dialog.popup", "Confirm deletion", cx)
                            .child(
                                alert_dialog_title("preview.alert-dialog.title", cx)
                                    .child("Are you sure?"),
                            )
                            .child(
                                alert_dialog_description("preview.alert-dialog.description", cx)
                                    .child("This action cannot be undone."),
                            )
                            .child_any(
                                div()
                                    .flex()
                                    .justify_end()
                                    .gap(px(8.0))
                                    .child(
                                        alert_dialog_cancel("preview.alert-dialog.cancel", cx)
                                            .child("Cancel"),
                                    )
                                    .child(
                                        alert_dialog_action("preview.alert-dialog.action", cx)
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
        collapsible(cx)
            .w(px(320.0))
            .child(collapsible_trigger(cx).child("More details"))
            .child(collapsible_content(cx).child("This panel opens with Base GPUI state."))
    }

    fn checkbox_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity().downgrade();
        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .child(checkbox_row(
                Checkbox::new("preview.checkbox.interactive")
                    .checked(self.checked)
                    .aria_label("Accept terms")
                    .on_checked_change(move |checked, _, _, cx| {
                        view.update(cx, |this, cx| {
                            this.checked = checked;
                            cx.notify();
                        })
                        .ok();
                    }),
                if self.checked {
                    "Accepted"
                } else {
                    "Accept terms"
                },
            ))
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
        div()
            .flex()
            .gap(px(10.0))
            .child(
                CheckboxGroup::new("preview.checkbox-group")
                    .aria_label("Notifications")
                    .default_value(["updates"])
                    .all_values(["updates", "digest", "mentions"])
                    .item(
                        CheckboxGroupItem::new("preview.checkbox-group.updates", "updates")
                            .aria_label("Product updates"),
                    )
                    .item(
                        CheckboxGroupItem::new("preview.checkbox-group.digest", "digest")
                            .aria_label("Weekly digest"),
                    )
                    .item(
                        CheckboxGroupItem::new("preview.checkbox-group.mentions", "mentions")
                            .aria_label("Mentions"),
                    ),
            )
            .child(div().flex().flex_col().gap(px(12.0)).children([
                "Product updates",
                "Weekly digest",
                "Mentions",
            ]))
    }

    fn combobox_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        combobox_root::<&'static str>("preview.combobox")
            .item_to_string_value(|value| (*value).into())
            .w(px(240.0))
            .child(
                combobox_input_group(cx)
                    .child(
                        combobox_input("preview.combobox.input", cx)
                            .placeholder("Search fruits…")
                            .aria_label("Fruits"),
                    )
                    .child(combobox_trigger("preview.combobox.trigger", cx).child("⌄")),
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
            .child(
                context_menu_trigger("preview.context-menu.trigger")
                    .w(px(280.0))
                    .h(px(150.0))
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
                                    .child("Back"),
                            )
                            .child(
                                context_menu_item("preview.context-menu.reload", cx)
                                    .label("Reload")
                                    .child("Reload"),
                            ),
                    ),
                ),
            )
    }

    fn dialog_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        dialog_root("preview.dialog")
            .child(dialog_trigger("preview.dialog.trigger", cx).child("Open dialog"))
            .child(
                dialog_portal().child(dialog_backdrop()).child(
                    dialog_viewport().child(
                        dialog_popup("preview.dialog.popup", "Edit profile", cx)
                            .child(dialog_title("preview.dialog.title", cx).child("Edit profile"))
                            .child(
                                dialog_description("preview.dialog.description", cx)
                                    .child("Make changes to your profile, then close this dialog."),
                            )
                            .child(dialog_close("preview.dialog.close", cx)),
                    ),
                ),
            )
    }

    fn drawer_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        drawer_root("preview.drawer")
            .child(drawer_trigger("preview.drawer.trigger", cx).child("Open drawer"))
            .child(
                drawer_portal().child(drawer_backdrop()).child(
                    drawer_viewport().child(
                        drawer_popup("preview.drawer.popup", "Drawer", cx).child(
                            drawer_content(cx).child(drawer_swipe_handle(cx)).child(
                                div()
                                    .p(px(20.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(12.0))
                                    .child(
                                        drawer_title("preview.drawer.title", cx).child("Move goal"),
                                    )
                                    .child(
                                        drawer_description("preview.drawer.description", cx)
                                            .child("Choose a new project for this task."),
                                    )
                                    .child(drawer_close("preview.drawer.close", cx).child("Close")),
                            ),
                        ),
                    ),
                ),
            )
    }

    fn field_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        field_root("preview.field", FieldOrientation::Vertical, cx)
            .w(px(280.0))
            .name("username")
            .child(field_label(cx).text("Username"))
            .child(field_control("preview.field.control", cx).placeholder("e.g. ada"))
            .child(field_description(cx).child("Visible on your public profile."))
    }

    fn fieldset_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        fieldset_root("preview.fieldset", cx)
            .w(px(300.0))
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
        form("preview.form", cx)
            .w(px(280.0))
            .aria_label("Subscribe")
            .child(
                field_root("preview.form.email", FieldOrientation::Vertical, cx)
                    .name("email")
                    .child(field_label(cx).text("Email"))
                    .child(
                        field_control("preview.form.email.control", cx)
                            .required(true)
                            .placeholder("you@example.com"),
                    ),
            )
            .child(
                Button::new("preview.form.submit")
                    .on_click(|_, window, cx| {
                        window.dispatch_action(Box::new(FormSubmitAction), cx)
                    })
                    .child("Subscribe"),
            )
    }

    fn input_preview(&self) -> impl IntoElement {
        div().w(px(320.0)).child(
            Input::new("preview.input")
                .default_value("hello@example.com")
                .aria_label("Email address"),
        )
    }

    fn menu_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        menu_root::<()>("preview.menu")
            .child(menu_trigger("preview.menu.trigger", cx).child("Open menu"))
            .child(
                menu_portal().child(
                    menu_positioner().child(
                        menu_popup("preview.menu.popup", cx)
                            .child(menu_item("preview.menu.cut", cx).label("Cut").child("Cut"))
                            .child(
                                menu_item("preview.menu.copy", cx)
                                    .label("Copy")
                                    .child("Copy"),
                            )
                            .child(
                                menu_item("preview.menu.paste", cx)
                                    .label("Paste")
                                    .child("Paste"),
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
                                            .child("New File"),
                                    )
                                    .child(
                                        menubar_item("preview.menubar.open", cx)
                                            .label("Open")
                                            .child("Open…"),
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
            .w(px(320.0))
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
        div().w(px(160.0)).child(
            NumberField::new("preview.number-field")
                .default_value(4.0)
                .range(Some(0.0), Some(10.0)),
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
                            .child(navigation_menu_trigger(cx).child_any("Docs").child_any("⌄"))
                            .child(
                                navigation_menu_content(cx)
                                    .w(px(200.0))
                                    .flex()
                                    .flex_col()
                                    .child("Getting started")
                                    .child("Components")
                                    .child("Theming"),
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
        OtpField::new("preview.otp-field", 6).aria_label("Verification code")
    }

    fn popover_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        popover_root("preview.popover")
            .child(popover_trigger("preview.popover.trigger", cx).child("Open popover"))
            .child(
                popover_portal().child(
                    popover_positioner().child(
                        popover_popup("preview.popover.popup", "Dimensions", cx)
                            .child(popover_title(cx).child("Dimensions"))
                            .child_any(
                                Input::new("preview.popover.width")
                                    .default_value("100%")
                                    .aria_label("Width"),
                            )
                            .child(popover_close("preview.popover.close", cx).child("Close")),
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
                        preview_card_popup("preview.preview-card.popup", cx)
                            .child_any("gpuicn")
                            .child_any("Open-code GPUI components."),
                    ),
                ),
            )
    }

    fn progress_preview(&self) -> impl IntoElement {
        div()
            .w(px(320.0))
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
        div()
            .flex()
            .gap(px(10.0))
            .child(
                RadioGroup::new("preview.radio-group")
                    .aria_label("Interface density")
                    .default_value("comfortable")
                    .item(
                        RadioItem::new("preview.radio-group.compact", "compact")
                            .aria_label("Compact"),
                    )
                    .item(
                        RadioItem::new("preview.radio-group.comfortable", "comfortable")
                            .aria_label("Comfortable"),
                    )
                    .item(
                        RadioItem::new("preview.radio-group.spacious", "spacious")
                            .aria_label("Spacious"),
                    ),
            )
            .child(div().flex().flex_col().gap(px(8.0)).children([
                "Compact",
                "Comfortable",
                "Spacious",
            ]))
    }

    fn scroll_area_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        scroll_area(cx)
            .id("preview.scroll-area")
            .w(px(300.0))
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
                    .child(select_value(cx).placeholder("Theme"))
                    .child(select_icon(cx).child("⌄")),
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
                                        .child(select_item_text().text("Dark")),
                                ),
                        ),
                    ),
                ),
            )
    }

    fn separator_preview(&self) -> impl IntoElement {
        div()
            .w(px(320.0))
            .flex()
            .flex_col()
            .gap(px(12.0))
            .child("gpuicn")
            .child(Separator::new("preview.separator"))
            .child("Open-code components for GPUI")
    }

    fn slider_preview(&self) -> impl IntoElement {
        div().w(px(320.0)).child(
            Slider::new("preview.slider")
                .default_value(48.0)
                .aria_label("Volume"),
        )
    }

    fn switch_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity().downgrade();
        div()
            .flex()
            .items_center()
            .gap(px(10.0))
            .child(
                Switch::new("preview.switch")
                    .checked(self.checked)
                    .aria_label("Airplane mode")
                    .on_checked_change(move |checked, _, _, cx| {
                        view.update(cx, |this, cx| {
                            this.checked = checked;
                            cx.notify();
                        })
                        .ok();
                    }),
            )
            .child(if self.checked {
                "Airplane mode on"
            } else {
                "Airplane mode off"
            })
    }

    fn tabs_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        tabs(cx)
            .default_value(Some("account"))
            .child(
                tabs_list(cx)
                    .child(
                        tabs_trigger(TabsVariant::Default, cx)
                            .value("account")
                            .child("Account"),
                    )
                    .child(
                        tabs_trigger(TabsVariant::Default, cx)
                            .value("password")
                            .child("Password"),
                    ),
            )
            .child(
                tabs_content(cx)
                    .value("account")
                    .child("Make changes to your account here."),
            )
            .child(
                tabs_content(cx)
                    .value("password")
                    .child("Change your password here."),
            )
    }

    fn toast_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let manager = create_toast_manager::<()>();
        let add_toast = manager.clone();
        let theme = UiTheme::read(cx).clone();
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
            .child(
                toast_portal().child(toast_viewport("preview.toast.viewport").content_builder(
                    move |_facts| {
                        toast_root_from_theme(&theme).child(
                            toast_content_from_theme(&theme)
                                .child(toast_title_from_theme(&theme))
                                .child(toast_description_from_theme(&theme))
                                .child(toast_close_from_theme(&theme).child_any("Dismiss")),
                        )
                    },
                )),
            )
    }

    fn toggle_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity().downgrade();
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
            .child("Bold")
    }

    fn toggle_group_preview(&self) -> impl IntoElement {
        ToggleGroup::new("preview.toggle-group")
            .aria_label("Text alignment")
            .default_value(["left"])
            .item(
                ToggleGroupItem::new("preview.toggle-group.left", "left")
                    .aria_label("Align left")
                    .child("L"),
            )
            .item(
                ToggleGroupItem::new("preview.toggle-group.center", "center")
                    .aria_label("Align center")
                    .child("C"),
            )
            .item(
                ToggleGroupItem::new("preview.toggle-group.right", "right")
                    .aria_label("Align right")
                    .child("R"),
            )
    }

    fn toolbar_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
                            .child("B"),
                    )
                    .child(
                        toolbar_button(cx)
                            .id("preview.toolbar.italic")
                            .aria_label("Italic")
                            .child("I"),
                    )
                    .child(
                        toolbar_button(cx)
                            .id("preview.toolbar.underline")
                            .aria_label("Underline")
                            .child("U"),
                    ),
            )
            .child(toolbar_separator(cx).h(px(16.0)).w(px(1.0)))
            .child(
                toolbar_button(cx)
                    .id("preview.toolbar.copy")
                    .aria_label("Copy")
                    .child("Copy"),
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

fn toast_root_from_theme(theme: &UiTheme) -> ToastRoot<()> {
    ToastRoot::new()
        .w(px(320.0))
        .rounded(px(12.0))
        .border_1()
        .border_color(theme.colors.border)
        .bg(theme.colors.popover)
        .text_color(theme.colors.popover_foreground)
        .p(px(16.0))
}

fn toast_content_from_theme(_theme: &UiTheme) -> ToastContent<()> {
    ToastContent::new().flex().flex_col().gap(px(4.0))
}

fn toast_title_from_theme(theme: &UiTheme) -> ToastTitle<()> {
    ToastTitle::new()
        .text_color(theme.colors.popover_foreground)
        .font_weight(gpui::FontWeight::MEDIUM)
}

fn toast_description_from_theme(theme: &UiTheme) -> ToastDescription<()> {
    ToastDescription::new().text_color(theme.colors.muted_foreground)
}

fn toast_close_from_theme(theme: &UiTheme) -> ToastClose<()> {
    ToastClose::new()
        .aria_label("Close toast")
        .mt(px(8.0))
        .text_color(theme.colors.muted_foreground)
}

fn checkbox_row(checkbox: Checkbox, label: &'static str) -> impl IntoElement {
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
