//! Small, compile-checked examples for the documentation's Usage sections.
#![allow(dead_code)]

// Installed components live at crate::ui; alias the library to check those imports.
use gpuicn as ui;

fn main() {}

mod accordion {
    use crate::ui::accordion::*;
    use gpui_kit::{App, IntoElement, ParentElement, Window};

    fn example(open: bool, window: &mut Window, cx: &mut App) -> impl IntoElement {
        accordion("faq", cx).child(
            accordion_item(cx)
                .open(open)
                .header(accordion_header(
                    accordion_trigger("shipping", open, false, cx).child("When will it arrive?"),
                ))
                .panel(accordion_content(
                    "shipping-content",
                    open,
                    "Within three working days.",
                    window,
                    cx,
                )),
        )
    }
}

mod alert_dialog {
    use crate::ui::alert_dialog::*;
    use crate::ui::dialog::*;
    use gpui_kit::{App, IntoElement, ParentElement, Window};

    fn example(handle: &DialogHandle, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let popup = dialog_popup("profile.popup", "Edit profile", cx)
            .child(dialog_title("profile.title", cx).child("Edit profile"))
            .child(dialog_action("profile.save", handle, cx).label("Save"));
        gpui_kit::div()
            .child(dialog_trigger("profile.open", handle, cx).label("Edit profile"))
            .child(alert_dialog("profile", handle, popup, window, cx))
    }
}

mod autocomplete {
    use crate::ui::autocomplete::*;
    use crate::ui::select::{SelectItem, SelectState};
    use gpui_kit::{App, IntoElement, Window};

    fn example(window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state("fruit", cx, |window, cx| {
            SelectState::new(
                [
                    SelectItem::new("apple", "Apple"),
                    SelectItem::new("pear", "Pear"),
                ],
                window,
                cx,
            )
        });
        autocomplete(&state)
            .aria_label("Fruit")
            .placeholder("Choose a fruit…")
    }
}

mod avatar {
    use crate::ui::avatar::Avatar;
    use gpui_kit::IntoElement;

    fn example() -> impl IntoElement {
        Avatar::new("profile.avatar")
            .image("https://raw.githubusercontent.com/devaryakjha/devaryakjha/6526e3d7415b2fb573ba3da4523b5c6948aa5d08/avatar.png")
            .aria_label("Arya")
            .fallback("AJ")
    }
}

mod button {
    use crate::ui::button::Button;
    use gpui_kit::IntoElement;

    fn example() -> impl IntoElement {
        Button::new("open-docs")
            .on_click(|_, _, cx| cx.open_url("https://ui.imajha.com"))
            .label("Open docs")
    }
}

mod checkbox {
    use crate::ui::checkbox::*;
    use gpui_kit::IntoElement;

    fn example(checked: bool) -> impl IntoElement {
        Checkbox::new("terms")
            .checked(checked)
            .aria_label("Accept terms")
            .on_change(|checked, _, _, _| println!("Accepted: {checked}"))
    }
}

mod checkbox_group {
    use crate::ui::checkbox_group::*;
    use gpui_kit::{App, IntoElement, Window};

    fn example(window: &mut Window, cx: &mut App) -> impl IntoElement {
        let value = window.use_keyed_state("notifications", cx, |_, _| {
            vec![gpui_kit::SharedString::from("email")]
        });
        CheckboxGroup::new("notifications")
            .aria_label("Notifications")
            .value(value.read(cx).clone())
            .item(CheckboxGroupItem::new("email", "email").label("Email"))
            .item(CheckboxGroupItem::new("push", "push").label("Push"))
            .on_change(move |next, _, cx| {
                value.update(cx, |value, cx| {
                    *value = next;
                    cx.notify();
                })
            })
    }
}

mod collapsible {
    use crate::ui::collapsible::*;
    use gpui_kit::{App, IntoElement, ParentElement, Window};

    fn example(open: bool, window: &mut Window, cx: &mut App) -> impl IntoElement {
        collapsible("projects-region", open, window, cx)
            .child(collapsible_trigger("projects", open, cx).child("Recent projects"))
            .content(collapsible_content(cx).child("Design system"))
    }
}

mod combobox {
    use crate::ui::combobox::*;
    use crate::ui::select::{SelectItem, SelectState};
    use gpui_kit::{App, IntoElement, Window};

    fn example(window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state("fruit", cx, |window, cx| {
            SelectState::new(
                [
                    SelectItem::new("apple", "Apple"),
                    SelectItem::new("pear", "Pear"),
                ],
                window,
                cx,
            )
        });
        combobox(&state)
            .aria_label("Fruit")
            .placeholder("Choose a fruit…")
    }
}

mod context_menu {
    use crate::ui::context_menu::*;
    use crate::ui::menu::*;
    use gpui_kit::{Entity, IntoElement, ParentElement};

    fn example(state: &Entity<MenuState>) -> impl IntoElement {
        context_menu(
            state,
            "File actions",
            gpui_kit::div().child("Right-click here"),
        )
    }
}

mod dialog {
    use crate::ui::dialog::*;
    use gpui_kit::{App, IntoElement, ParentElement, Window};

    fn example(handle: &DialogHandle, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let popup = dialog_popup("profile.popup", "Edit profile", cx)
            .child(dialog_title("profile.title", cx).child("Edit profile"))
            .child(dialog_action("profile.save", handle, cx).label("Save"));
        gpui_kit::div()
            .child(dialog_trigger("profile.open", handle, cx).label("Edit profile"))
            .child(dialog("profile", handle, popup, window, cx))
    }
}

mod drawer {
    use crate::ui::drawer::*;
    use gpui_kit::{App, IntoElement, ParentElement};

    fn example(handle: &DrawerHandle, cx: &App) -> impl IntoElement {
        Drawer::new("profile", handle)
            .direction(DrawerSide::Bottom)
            .show_swipe_handle(true)
            .child(drawer_trigger("profile.open", handle).label("Edit profile"))
            .content(
                DrawerContent::new("profile.content", "Edit profile")
                    .child(
                        drawer_header(DrawerSide::Bottom, cx)
                            .child(drawer_title("profile.title", cx).child("Edit profile"))
                            .child(
                                drawer_description("profile.description", cx)
                                    .child("Make changes to your profile."),
                            ),
                    )
                    .child(drawer_body("profile.body", cx).child("Your profile fields go here."))
                    .child(
                        drawer_footer(cx)
                            .child(drawer_close("profile.cancel", handle).label("Done")),
                    ),
            )
    }
}

mod field {
    use crate::ui::{field::*, input::InputState};
    use gpui_kit::{Entity, IntoElement};

    fn example(name: &Entity<InputState>) -> impl IntoElement {
        Field::new("name", name)
            .label("Full name")
            .required(true)
            .description("Use the name on your account.")
    }
}

mod fieldset {
    use crate::ui::{field::Field, fieldset::*, input::InputState};
    use gpui_kit::Entity;
    use gpui_kit::{App, IntoElement, ParentElement, StatefulInteractiveElement};

    fn example(name: &Entity<InputState>, cx: &App) -> impl IntoElement {
        fieldset_root("shipping", cx)
            .aria_label("Shipping address")
            .child(fieldset_legend(FieldsetLegendVariant::Legend, cx).child("Shipping address"))
            .child(Field::new("name", name).label("Full name"))
    }
}

mod form {
    use crate::ui::{Button, field::Field, form::*, input::InputState};
    use gpui_kit::Entity;
    use gpui_kit::{App, IntoElement, ParentElement, StatefulInteractiveElement};

    fn example(email: &Entity<InputState>, cx: &App) -> impl IntoElement {
        // Validate in the owning view and share its submit handler with InputEvent::PressEnter.
        form("subscribe", cx)
            .aria_label("Subscribe")
            .child(Field::new("email", email).label("Email").required(true))
            .child(Button::new("subscribe.submit").label("Subscribe"))
    }
}

mod input {
    use crate::ui::input::*;
    use gpui_kit::{App, IntoElement, Window};

    fn example(window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state("email", cx, |window, cx| {
            InputState::new(window, cx).placeholder("you@example.com")
        });
        // Subscribe to InputEvent on state in the owning view to handle changes and Enter.
        Input::new(&state).aria_label("Email address")
    }
}

mod menu {
    use crate::ui::menu::*;
    use gpui_kit::{Entity, IntoElement};

    fn example(state: &Entity<MenuState>) -> impl IntoElement {
        Menu::new(state, "Actions")
    }
}

mod menubar {
    use crate::ui::menu::*;
    use crate::ui::menubar::*;
    use gpui_kit::{Entity, IntoElement};

    fn example(state: &Entity<MenuState>) -> impl IntoElement {
        Menubar::new("app-menu", "Application").menu(state, "File")
    }
}

mod meter {
    use crate::ui::meter::Meter;
    use gpui_kit::IntoElement;

    fn example() -> impl IntoElement {
        Meter::new("storage").value(68.).aria_label("Storage used")
    }
}

mod navigation_menu {
    use crate::ui::menu::*;
    use crate::ui::navigation_menu::*;
    use gpui_kit::{Entity, IntoElement};

    fn example(state: &Entity<MenuState>) -> impl IntoElement {
        navigation_menu("docs", "Documentation").menu(state, "Getting started")
    }
}

mod number_field {
    use crate::ui::{input::InputState, number_field::*};
    use gpui_kit::{App, IntoElement, Window};

    fn example(window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state("quantity", cx, |window, cx| {
            InputState::new(window, cx)
                .default_value("3")
                .min(0.)
                .max(20.)
                .step(1.)
        });
        NumberField::new(&state).aria_label("Quantity")
    }
}

mod otp_field {
    use crate::ui::otp_field::*;
    use gpui_kit::{App, IntoElement, Window};

    fn example(window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state("code", cx, |window, cx| OtpState::new(6, window, cx));
        OtpField::new(&state).aria_label("Verification code")
    }
}

mod popover {
    use crate::ui::popover::*;
    use gpui_kit::{App, IntoElement, ParentElement};

    fn example(cx: &App) -> impl IntoElement {
        popover("details")
            .trigger(popover_trigger("details.open", cx).child("Details"))
            .content(|_, _, cx| {
                popover_popup("details.popup", "Details", cx).child("Account details")
            })
    }
}

mod preview_card {
    use crate::ui::{Button, preview_card::*};
    use gpui_kit::{IntoElement, ParentElement};

    fn example() -> impl IntoElement {
        preview_card("profile")
            .trigger(Button::new("profile.link").label("@gpuicn"))
            .content(|_, _, cx| {
                preview_card_popup("profile.popup", cx).child("Native GPUI components")
            })
    }
}

mod progress {
    use crate::ui::progress::Progress;
    use gpui_kit::IntoElement;

    fn example() -> impl IntoElement {
        Progress::new("upload").value(64.).label("Uploading…")
    }
}

mod radio_group {
    use crate::ui::radio_group::*;
    use gpui_kit::{App, IntoElement, Window};

    fn example(window: &mut Window, cx: &mut App) -> impl IntoElement {
        let value = window.use_keyed_state("density", cx, |_, _| {
            gpui_kit::SharedString::from("compact")
        });
        RadioGroup::new("density")
            .aria_label("Density")
            .value(value.read(cx).clone())
            .item(RadioItem::new("compact", "compact").label("Compact"))
            .item(RadioItem::new("comfortable", "comfortable").label("Comfortable"))
            .on_change(move |next, _, cx| {
                value.update(cx, |value, cx| {
                    *value = next;
                    cx.notify();
                })
            })
    }
}

mod scroll_area {
    use crate::ui::scroll_area::*;
    use gpui_kit::{IntoElement, ParentElement, Styled};

    fn example() -> impl IntoElement {
        ScrollArea::new("tags")
            .aria_label("Tags")
            .h(gpui_kit::px(200.))
            .children((0..50).map(|i| gpui_kit::div().child(format!("Tag {i}"))))
    }
}

mod select {
    use crate::ui::select::*;
    use crate::ui::select::{SelectItem, SelectState};
    use gpui_kit::{App, IntoElement, Window};

    fn example(window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state("fruit", cx, |window, cx| {
            SelectState::new(
                [
                    SelectItem::new("apple", "Apple"),
                    SelectItem::new("pear", "Pear"),
                ],
                window,
                cx,
            )
        });
        Select::new(&state)
            .aria_label("Fruit")
            .placeholder("Choose a fruit…")
    }
}

mod separator {
    use crate::ui::separator::Separator;
    use gpui_kit::IntoElement;

    fn example() -> impl IntoElement {
        Separator::new("section-divider")
    }
}

mod slider {
    use crate::ui::slider::*;
    use gpui_kit::{App, IntoElement, Window};

    fn example(window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state("volume", cx, |_, _cx| {
            SliderState::new().min(0.).max(100.).default_value(50.)
        });
        Slider::new(&state).aria_label("Volume")
    }
}

mod switch {
    use crate::ui::switch::*;
    use gpui_kit::IntoElement;

    fn example(checked: bool) -> impl IntoElement {
        Switch::new("airplane")
            .checked(checked)
            .aria_label("Airplane mode")
            .on_change(|checked, _, _, _| println!("Airplane mode: {checked}"))
    }
}

mod tabs {
    use crate::ui::tabs::*;
    use gpui_kit::{App, IntoElement, ParentElement, Window};

    fn example(window: &mut Window, cx: &mut App) -> impl IntoElement {
        let selected = window.use_keyed_state("settings", cx, |_, _| {
            gpui_kit::SharedString::from("account")
        });
        let value = selected.read(cx).clone();
        gpui_kit::div()
            .child(
                Tabs::new("settings")
                    .selected(value.clone())
                    .item(Tab::new("account", "account", "Account"))
                    .item(Tab::new("password", "password", "Password"))
                    .on_change(move |value, _, cx| {
                        selected.update(cx, |selected, cx| {
                            *selected = value;
                            cx.notify();
                        })
                    }),
            )
            .child(
                tabs_content("settings.panel", value.clone(), cx).child(if value == "account" {
                    "Account settings"
                } else {
                    "Password settings"
                }),
            )
    }
}

mod toast {
    use crate::ui::{Button, toast::*};
    use gpui_kit::{App, IntoElement, ParentElement, Window};

    fn example(window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state("notifications", cx, |_, cx| ToastState::new(cx));
        let notify = state.clone();
        gpui_kit::div()
            .child(Button::new("save").label("Save").on_click(move |_, _, cx| {
                notify.update(cx, |state, cx| {
                    state.push(
                        "saved",
                        "Saved",
                        "Your changes were saved.",
                        Some(std::time::Duration::from_secs(5)),
                        cx,
                    )
                });
            }))
            .child(state)
    }
}

mod toggle {
    use crate::ui::toggle::*;
    use gpui_kit::{IntoElement, ParentElement};

    fn example(pressed: bool) -> impl IntoElement {
        Toggle::new("bold")
            .aria_label("Bold")
            .pressed(pressed)
            .child("B")
            .on_change(|pressed, _, _, _| println!("Bold: {pressed}"))
    }
}

mod toggle_group {
    use crate::ui::toggle_group::*;
    use gpui_kit::{App, IntoElement, ParentElement, Window};

    fn example(window: &mut Window, cx: &mut App) -> impl IntoElement {
        let value =
            window.use_keyed_state("styles", cx, |_, _| Vec::<gpui_kit::SharedString>::new());
        ToggleGroup::new("styles")
            .aria_label("Text styles")
            .multiple(true)
            .value(value.read(cx).clone())
            .item(
                ToggleGroupItem::new("bold", "bold")
                    .aria_label("Bold")
                    .child("B"),
            )
            .item(
                ToggleGroupItem::new("italic", "italic")
                    .aria_label("Italic")
                    .child("I"),
            )
            .on_change(move |next, _, cx| {
                value.update(cx, |value, cx| {
                    *value = next;
                    cx.notify();
                })
            })
    }
}

mod toolbar {
    use crate::ui::toolbar::*;
    use gpui_kit::{App, IntoElement, ParentElement};

    fn example(cx: &App) -> impl IntoElement {
        Toolbar::new("formatting", "Formatting", cx)
            .button(ToolbarButton::new("bold", "Bold", cx).child("B"))
            .button(ToolbarButton::new("italic", "Italic", cx).child("I"))
    }
}

mod tooltip {
    use crate::ui::tooltip::*;
    use gpui_kit::{App, IntoElement, Window};

    fn example(window: &mut Window, cx: &mut App) -> impl IntoElement {
        tooltip(
            "save-tooltip",
            "Save changes",
            gpui_kit::base::Button::new("save").accessibility_label("Save"),
            window,
            cx,
        )
    }
}

mod resizable {
    use crate::ui::resizable::{PaneLimits, Resizable};
    use gpui_kit::{IntoElement, Pixels, div, px};

    fn example() -> impl IntoElement {
        // In a Render implementation, pass caller-owned size and a cx.listener
        // that saves the new size and calls cx.notify(). This sample stays fixed.
        Resizable::new(
            "panes",
            "Resize navigation",
            px(240.),
            div(),
            div(),
            |_: &Pixels, _, _| {},
        )
        .first_limits(PaneLimits::new(px(180.), px(360.)))
        .second_limits(PaneLimits::new(px(300.), px(2000.)))
    }
}

mod sidebar {
    use crate::ui::sidebar::{Sidebar, SidebarItem, sidebar_group_label};
    use gpui_kit::{App, IntoElement, ParentElement};

    fn example(cx: &App) -> impl IntoElement {
        Sidebar::new("navigation", "Workspace navigation")
            .header("My workspace")
            .child(sidebar_group_label("Project", cx))
            .child(SidebarItem::new("changes", "Changes").selected(true))
            .child(SidebarItem::new("history", "History").on_activate(|_, _, _| {}))
            .footer(SidebarItem::new("settings", "Settings"))
    }
}

mod virtual_list {
    use crate::ui::virtual_list::{ListItem, ListSelectionMode, VirtualList, VirtualListState};
    use gpui_kit::{IntoElement, ParentElement, div};

    // Store this once as `list: VirtualListState` in your view, not in render.
    fn create_list() -> VirtualListState {
        let state = VirtualListState::new(vec![
            ListItem::new("readme", "README.md"),
            ListItem::new("main", "src/main.rs"),
        ])
        .expect("unique application IDs");
        state.set_selection_mode(ListSelectionMode::Multiple);
        state
    }

    fn example(state: &VirtualListState) -> impl IntoElement {
        VirtualList::new("files", "Files", state.clone(), |row, _, _| {
            div().child(row.item.label.clone()).into_any_element()
        })
    }
}
