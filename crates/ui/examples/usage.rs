//! Small, compile-checked examples for the documentation's Usage sections.
#![allow(dead_code)]

// Installed components live at crate::ui; alias the library to check those imports.
use gpuicn as ui;

fn main() {}

mod accordion {
    use crate::ui::accordion::*;
    use gpui::{App, IntoElement, ParentElement};

    fn example(cx: &App) -> impl IntoElement {
        accordion(cx).id("faq").child(
            accordion_item("shipping", cx)
                .id("faq.shipping")
                .child(
                    accordion_header().child(
                        accordion_trigger(cx)
                            .id("faq.shipping.trigger")
                            .child("When will it arrive?"),
                    ),
                )
                .child(accordion_content(cx).child("Within three working days.")),
        )
    }
}

mod alert_dialog {
    use crate::ui::alert_dialog::*;
    use gpui::{App, IntoElement, ParentElement};

    fn example(cx: &App) -> impl IntoElement {
        alert_dialog_root("discard")
            .child(alert_dialog_trigger("discard.trigger", cx).child("Discard changes"))
            .child(
                alert_dialog_portal().child(alert_dialog_backdrop()).child(
                    alert_dialog_viewport().child(
                        alert_dialog_popup("discard.popup", "Discard changes?", cx)
                            .child(
                                alert_dialog_title("discard.title", cx).child("Discard changes?"),
                            )
                            .child(
                                alert_dialog_description("discard.description", cx)
                                    .child("Your unsaved edits will be lost."),
                            )
                            .child(alert_dialog_cancel("discard.cancel", cx).child("Keep editing"))
                            .child(alert_dialog_action("discard.confirm", cx).child("Discard")),
                    ),
                ),
            )
    }
}

mod autocomplete {
    use crate::ui::autocomplete::*;
    use gpui::{App, IntoElement};

    fn example(cx: &App) -> impl IntoElement {
        autocomplete_root::<&str>("search")
            .child(
                autocomplete_input("search.input", cx)
                    .aria_label("Search components")
                    .placeholder("Search…"),
            )
            .child(
                autocomplete_portal().child(
                    autocomplete_positioner().child(
                        autocomplete_popup(cx).child(
                            autocomplete_list().child(
                                autocomplete_item("search.button", cx)
                                    .value("button")
                                    .label("Button")
                                    .child_any("Button"),
                            ),
                        ),
                    ),
                ),
            )
    }
}

mod avatar {
    use crate::ui::avatar::Avatar;
    use gpui::{IntoElement, ParentElement};

    fn example() -> impl IntoElement {
        Avatar::new("profile.avatar").child("AJ")
    }
}

mod button {
    use crate::ui::button::Button;
    use gpui::{IntoElement, ParentElement};

    fn example() -> impl IntoElement {
        Button::new("open-docs")
            .on_click(|_, _, cx| cx.open_url("https://ui.imajha.com"))
            .child("Open docs")
    }
}

mod checkbox {
    use crate::ui::checkbox::Checkbox;
    use gpui::{IntoElement, ParentElement, Styled, div, px};

    fn example() -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap(px(8.))
            .child(Checkbox::new("terms").aria_label("Accept terms"))
            .child("Accept terms")
    }
}

mod checkbox_group {
    use crate::ui::checkbox_group::{CheckboxGroup, CheckboxGroupItem};
    use gpui::IntoElement;

    fn example() -> impl IntoElement {
        CheckboxGroup::new("notifications")
            .aria_label("Notifications")
            .default_value(["updates"])
            .all_values(["updates", "digest"])
            .item(
                CheckboxGroupItem::new("notifications.updates", "updates")
                    .label("Product updates")
                    .aria_label("Product updates"),
            )
            .item(
                CheckboxGroupItem::new("notifications.digest", "digest")
                    .label("Weekly digest")
                    .aria_label("Weekly digest"),
            )
    }
}

mod collapsible {
    use crate::ui::collapsible::*;
    use gpui::{App, IntoElement, ParentElement};

    fn example(cx: &App) -> impl IntoElement {
        collapsible(cx)
            .id("details")
            .child(
                collapsible_trigger(cx)
                    .id("details.trigger")
                    .child("Show details"),
            )
            .child(collapsible_content(cx).child("Your order has shipped."))
    }
}

mod combobox {
    use crate::ui::combobox::*;
    use gpui::{App, IntoElement};

    fn example(cx: &App) -> impl IntoElement {
        combobox_root::<&str>("fruit")
            .child(
                combobox_input("fruit.input", cx)
                    .aria_label("Fruit")
                    .placeholder("Choose a fruit…"),
            )
            .child(
                combobox_portal().child(
                    combobox_positioner().child(
                        combobox_popup(cx).child(
                            combobox_list().child(
                                combobox_item("fruit.apple", cx)
                                    .value("apple")
                                    .label("Apple")
                                    .child_any("Apple"),
                            ),
                        ),
                    ),
                ),
            )
    }
}

mod context_menu {
    use crate::ui::context_menu::*;
    use gpui::{App, IntoElement, ParentElement, Styled, px};

    fn example(cx: &App) -> impl IntoElement {
        context_menu_root::<()>("file-menu")
            .child(
                context_menu_trigger("file-menu.trigger")
                    .p(px(24.))
                    .child("Right-click here"),
            )
            .child(
                context_menu_portal().child(
                    context_menu_positioner().child(
                        context_menu_popup("file-menu.popup", cx).child(
                            context_menu_item("file-menu.copy", cx)
                                .label("Copy")
                                .child("Copy"),
                        ),
                    ),
                ),
            )
    }
}

mod dialog {
    use crate::ui::dialog::*;
    use gpui::{App, IntoElement, ParentElement};

    fn example(cx: &App) -> impl IntoElement {
        dialog_root("welcome")
            .child(dialog_trigger("welcome.trigger", cx).child("Open dialog"))
            .child(
                dialog_portal().child(dialog_backdrop()).child(
                    dialog_viewport().child(
                        dialog_popup("welcome.popup", "Welcome", cx)
                            .child(dialog_title("welcome.title", cx).child("Welcome"))
                            .child(
                                dialog_description("welcome.description", cx)
                                    .child("Your workspace is ready."),
                            )
                            .child(dialog_action("welcome.done", cx).child("Done")),
                    ),
                ),
            )
    }
}

mod drawer {
    use crate::ui::drawer::*;
    use gpui::{App, IntoElement, ParentElement};

    fn example(cx: &App) -> impl IntoElement {
        drawer_root("details")
            .child(drawer_trigger("details.trigger", cx).child("Show details"))
            .child(
                drawer_portal().child(drawer_backdrop()).child(
                    drawer_viewport().child(
                        drawer_popup("details.popup", "Order details", cx)
                            .child(drawer_title("details.title", cx).child("Order details"))
                            .child(
                                drawer_description("details.description", cx)
                                    .child("Your order has shipped."),
                            )
                            .child(drawer_close("details.close", cx).child("Close")),
                    ),
                ),
            )
    }
}

mod field {
    use crate::ui::field::*;
    use gpui::{App, IntoElement, ParentElement};

    fn example(cx: &App) -> impl IntoElement {
        field_root("username", FieldOrientation::Vertical, cx)
            .name("username")
            .child(field_label(cx).text("Username"))
            .child(field_control("username.input", cx).placeholder("ada"))
            .child(field_description(cx).child("Visible on your profile."))
    }
}

mod fieldset {
    use crate::ui::{field::*, fieldset::*};
    use gpui::{App, IntoElement, ParentElement};

    fn example(cx: &App) -> impl IntoElement {
        fieldset_root("shipping", cx)
            .aria_label("Shipping address")
            .child(fieldset_legend(FieldsetLegendVariant::Legend, cx).child("Shipping address"))
            .child_any(
                field_root("shipping.city", FieldOrientation::Vertical, cx)
                    .name("city")
                    .child(field_label(cx).text("City"))
                    .child(field_control("shipping.city.input", cx).placeholder("Bengaluru")),
            )
    }
}

mod form {
    use crate::ui::{button::Button, field::*, form::*};
    use gpui::{App, IntoElement, ParentElement};

    fn example(cx: &App) -> impl IntoElement {
        form("subscribe", cx)
            .child(
                field_root("email", FieldOrientation::Vertical, cx)
                    .name("email")
                    .child(field_label(cx).text("Email"))
                    .child(field_control("email.input", cx).required(true))
                    .child(field_error(cx).child("Enter your email address.")),
            )
            .child(
                Button::new("subscribe.submit")
                    .child("Subscribe")
                    .on_click(|_, window, cx| {
                        window.dispatch_action(Box::new(FormSubmitAction), cx)
                    }),
            )
    }
}

mod input {
    use crate::ui::input::Input;
    use gpui::IntoElement;

    fn example() -> impl IntoElement {
        Input::new("email")
            .aria_label("Email address")
            .placeholder("you@example.com")
    }
}

mod menu {
    use crate::ui::menu::*;
    use gpui::{App, IntoElement, ParentElement};

    fn example(cx: &App) -> impl IntoElement {
        menu_root::<()>("account")
            .child(menu_trigger("account.trigger", cx).child("Account"))
            .child(
                menu_portal().child(
                    menu_positioner().child(
                        menu_popup("account.popup", cx).child(
                            menu_item("account.profile", cx)
                                .label("Profile")
                                .child("Profile"),
                        ),
                    ),
                ),
            )
    }
}

mod menubar {
    use crate::ui::{menu::menu_positioner, menubar::*};
    use gpui::{App, IntoElement, ParentElement};

    fn example(cx: &App) -> impl IntoElement {
        menubar("app-menu", cx)
            .aria_label("Application menu")
            .child(
                menubar_menu::<()>("file")
                    .child(menubar_trigger("file.trigger", cx).child("File"))
                    .child(
                        menubar_portal().child(
                            menu_positioner().child(
                                menubar_content("file.popup", cx).child(
                                    menubar_item("file.new", cx)
                                        .label("New file")
                                        .child("New file"),
                                ),
                            ),
                        ),
                    ),
            )
    }
}

mod meter {
    use crate::ui::meter::Meter;
    use gpui::IntoElement;

    fn example() -> impl IntoElement {
        Meter::new("storage").value(68.).aria_label("Storage used")
    }
}

mod navigation_menu {
    use crate::ui::navigation_menu::*;
    use gpui::{App, IntoElement, ParentElement};

    fn example(cx: &App) -> impl IntoElement {
        navigation_menu::<&str>(cx)
            .id("navigation")
            .aria_label("Main navigation")
            .child(
                navigation_menu_list().child(
                    navigation_menu_item()
                        .value("docs")
                        .child(navigation_menu_trigger(cx).child_any("Docs"))
                        .child(
                            navigation_menu_content(cx)
                                .child(navigation_menu_link::<&str>(cx).child("Getting started")),
                        ),
                ),
            )
            .child(
                navigation_menu_portal().child(
                    navigation_menu_positioner()
                        .child(navigation_menu_popup(cx).child(navigation_menu_viewport(cx))),
                ),
            )
    }
}

mod number_field {
    use crate::ui::number_field::NumberField;
    use gpui::IntoElement;

    fn example() -> impl IntoElement {
        NumberField::new("quantity")
            .default_value(1.)
            .range(Some(1.), Some(10.))
    }
}

mod otp_field {
    use crate::ui::otp_field::OtpField;
    use gpui::IntoElement;

    fn example() -> impl IntoElement {
        OtpField::new("verification", 6).aria_label("Verification code")
    }
}

mod popover {
    use crate::ui::popover::*;
    use gpui::{App, IntoElement, ParentElement};

    fn example(cx: &App) -> impl IntoElement {
        popover_root("help")
            .child(popover_trigger("help.trigger", cx).child("Help"))
            .child(
                popover_portal().child(
                    popover_positioner().child(
                        popover_popup("help.popup", "Help", cx)
                            .child(popover_title(cx).child("Need a hand?"))
                            .child(
                                popover_description(cx)
                                    .child("Contact support from your account settings."),
                            ),
                    ),
                ),
            )
    }
}

mod preview_card {
    use crate::ui::preview_card::*;
    use gpui::{App, IntoElement, ParentElement};

    fn example(cx: &App) -> impl IntoElement {
        preview_card_root("profile")
            .child(preview_card_trigger("profile.trigger").child("@ada"))
            .child(preview_card_portal().child(preview_card_positioner().child(
                preview_card_popup("profile.popup", cx).child_any("Ada — software engineer"),
            )))
    }
}

mod progress {
    use crate::ui::progress::Progress;
    use gpui::IntoElement;

    fn example() -> impl IntoElement {
        Progress::new("upload").value(64.).label("Uploading…")
    }
}

mod radio_group {
    use crate::ui::radio_group::{RadioGroup, RadioItem};
    use gpui::IntoElement;

    fn example() -> impl IntoElement {
        RadioGroup::new("density")
            .aria_label("Density")
            .default_value("compact")
            .item(RadioItem::new("density.compact", "compact").label("Compact"))
            .item(RadioItem::new("density.comfortable", "comfortable").label("Comfortable"))
    }
}

mod scroll_area {
    use crate::ui::scroll_area::*;
    use gpui::{App, IntoElement, ParentElement, Styled, div, px};

    fn example(cx: &App) -> impl IntoElement {
        scroll_area(cx)
            .id("releases")
            .h(px(160.))
            .child(
                scroll_area_viewport(cx)
                    .id("releases.viewport")
                    .aria_label("Releases")
                    .child(scroll_area_content(cx).children(
                        (1..=20).map(|version| div().child(format!("Version {version}"))),
                    )),
            )
            .child(
                scroll_area_scrollbar(ScrollAreaOrientation::Vertical, cx)
                    .id("releases.scrollbar")
                    .child(scroll_area_thumb(cx)),
            )
    }
}

mod select {
    use crate::ui::select::*;
    use gpui::{App, IntoElement};

    fn example(cx: &App) -> impl IntoElement {
        select_root::<&str>("theme")
            .default_value(Some("system"))
            .item_to_string_value(|value| (*value).into())
            .child(
                select_trigger("theme.trigger", cx)
                    .aria_label("Theme")
                    .child(select_value(cx)),
            )
            .child(
                select_portal().child(
                    select_positioner().child(
                        select_popup(cx).child(
                            select_list()
                                .child(
                                    select_item("theme.system", cx)
                                        .value("system")
                                        .label("System")
                                        .child(select_item_text().text("System")),
                                )
                                .child(
                                    select_item("theme.dark", cx)
                                        .value("dark")
                                        .label("Dark")
                                        .child(select_item_text().text("Dark")),
                                ),
                        ),
                    ),
                ),
            )
    }
}

mod separator {
    use crate::ui::separator::Separator;
    use gpui::IntoElement;

    fn example() -> impl IntoElement {
        Separator::new("section-divider")
    }
}

mod slider {
    use crate::ui::slider::Slider;
    use gpui::IntoElement;

    fn example() -> impl IntoElement {
        Slider::new("volume")
            .aria_label("Volume")
            .default_value(50.)
    }
}

mod switch {
    use crate::ui::switch::Switch;
    use gpui::{IntoElement, ParentElement, Styled, div, px};

    fn example() -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap(px(8.))
            .child(
                Switch::new("notifications")
                    .aria_label("Notifications")
                    .default_checked(true),
            )
            .child("Notifications")
    }
}

mod tabs {
    use crate::ui::tabs::*;
    use gpui::{App, IntoElement, ParentElement};

    fn example(cx: &App) -> impl IntoElement {
        tabs(cx)
            .id("settings")
            .default_value(Some("account"))
            .child(
                tabs_list(cx)
                    .child(
                        tabs_trigger(TabsVariant::Default, cx)
                            .id("settings.account")
                            .value("account")
                            .child("Account"),
                    )
                    .child(
                        tabs_trigger(TabsVariant::Default, cx)
                            .id("settings.password")
                            .value("password")
                            .child("Password"),
                    ),
            )
            .child(tabs_content(cx).value("account").child("Account settings"))
            .child(
                tabs_content(cx)
                    .value("password")
                    .child("Password settings"),
            )
    }
}

mod toast {
    use crate::ui::{button::Button, toast::*};
    use gpui::{App, IntoElement, ParentElement};

    fn example(cx: &App) -> impl IntoElement {
        let manager = create_toast_manager::<()>();
        let notifications = manager.clone();
        toast_provider("notifications")
            .manager(manager)
            .child_any(Button::new("save").child("Save").on_click(move |_, _, cx| {
                notifications.add(
                    ToastOptions::new()
                        .title("Saved")
                        .description("Your changes are saved."),
                    cx,
                );
            }))
            .child(toast_portal().child(toast_viewport("notifications.viewport", cx)))
    }
}

mod toggle {
    use crate::ui::toggle::Toggle;
    use gpui::{IntoElement, ParentElement};

    fn example() -> impl IntoElement {
        Toggle::new("bold").aria_label("Bold").child("B")
    }
}

mod toggle_group {
    use crate::ui::toggle_group::{ToggleGroup, ToggleGroupItem};
    use gpui::{IntoElement, ParentElement};

    fn example() -> impl IntoElement {
        ToggleGroup::new("alignment")
            .aria_label("Text alignment")
            .default_value(["left"])
            .item(
                ToggleGroupItem::new("alignment.left", "left")
                    .aria_label("Align left")
                    .child("Left"),
            )
            .item(
                ToggleGroupItem::new("alignment.center", "center")
                    .aria_label("Align center")
                    .child("Center"),
            )
    }
}

mod toolbar {
    use crate::ui::toolbar::*;
    use gpui::{App, IntoElement, ParentElement};

    fn example(cx: &App) -> impl IntoElement {
        toolbar(cx)
            .id("formatting")
            .aria_label("Formatting")
            .child(
                toolbar_button(cx)
                    .id("formatting.bold")
                    .aria_label("Bold")
                    .child("B"),
            )
            .child(
                toolbar_button(cx)
                    .id("formatting.italic")
                    .aria_label("Italic")
                    .child("I"),
            )
    }
}

mod tooltip {
    use crate::ui::{button::Button, tooltip::*};
    use gpui::{App, IntoElement, ParentElement};

    fn example(cx: &App) -> impl IntoElement {
        tooltip_provider("tooltips").child(
            tooltip_root("save-hint")
                .child(
                    tooltip_trigger("save-hint.trigger").child(Button::new("save").child("Save")),
                )
                .child(
                    tooltip_portal().child(tooltip_positioner().child(
                        tooltip_popup("save-hint.popup", cx).child_any("Save your changes"),
                    )),
                ),
        )
    }
}
