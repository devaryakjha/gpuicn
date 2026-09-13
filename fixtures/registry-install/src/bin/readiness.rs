#![allow(dead_code, unused_imports)]

#[path = "../ui/mod.rs"]
mod ui;

use std::borrow::Cow;

use gpui_icons::{LucideAssetSource, LucideIcon, lucide};
use gpui_kit::{
    App, AppContext as _, Bounds, Context, Entity, FontWeight, InteractiveElement as _,
    IntoElement, ParentElement as _, Render, Role, SharedString, StatefulInteractiveElement as _,
    Styled, Subscription, Window, WindowBounds, WindowOptions, div, prelude::FluentBuilder as _,
    px, rgb, size,
};
use ui::{
    button::{Button, ButtonVariant},
    dialog::{
        DialogHandle, dialog, dialog_action, dialog_close, dialog_description, dialog_footer,
        dialog_popup, dialog_title,
    },
    field::Field,
    form::form,
    input::{Input, InputEvent, InputState},
    number_field::NumberField,
    resizable::{PaneLimits, Resizable},
    select::{Select, SelectItem, SelectState},
    sidebar::{
        Sidebar, SidebarCollapsible, SidebarItem, SidebarLayout, SidebarState, sidebar_group_label,
        sidebar_is_mobile, sidebar_menu, sidebar_trigger,
    },
    switch::Switch,
    theme::UiTheme,
};

fn main() {
    gpui_kit::platform::application()
        .with_assets(LucideAssetSource)
        .run(|cx: &mut App| {
            ui::theme::init(cx);
            cx.text_system()
                .add_fonts(vec![
                    Cow::Borrowed(include_bytes!("../../assets/fonts/Geist-Regular.ttf")),
                    Cow::Borrowed(include_bytes!("../../assets/fonts/Geist-Medium.ttf")),
                    Cow::Borrowed(include_bytes!("../../assets/fonts/GeistMono-Regular.ttf")),
                ])
                .expect("load bundled Geist fonts");
            UiTheme::set(cx, UiTheme::neutral_light());

            let bounds = Bounds::centered(None, size(px(1080.), px(700.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| {
                    window.set_window_title("gpuicn adoption consumer");
                    cx.new(|cx| AdoptionConsumer::new(window, cx))
                },
            )
            .expect("open adoption consumer");
            cx.activate(true);
        });
}

#[derive(Clone)]
struct Record {
    id: usize,
    name: String,
    owner: &'static str,
    status: &'static str,
}

struct AdoptionConsumer {
    page: usize,
    sidebar: SidebarState,
    pane_width: gpui_kit::Pixels,
    dark: bool,
    custom_palette: bool,
    compact: bool,
    large_text: bool,
    long_content: bool,
    disable_fields: bool,
    workspace_name: Entity<InputState>,
    region: Entity<SelectState>,
    seats: Entity<InputState>,
    saved_settings: String,
    settings_errors: [bool; 3],
    filter: Entity<InputState>,
    edit_name: Entity<InputState>,
    edit_record: Option<usize>,
    editor: DialogHandle,
    records: Vec<Record>,
    selected_record: usize,
    _subscriptions: Vec<Subscription>,
}

impl AdoptionConsumer {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let workspace_name = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value("Northwind workspace")
                .placeholder("Workspace name")
        });
        let region = cx.new(|cx| {
            SelectState::new(
                [
                    SelectItem::new("us", "United States"),
                    SelectItem::new("eu", "Europe"),
                    SelectItem::new("apac", "Asia Pacific"),
                ],
                window,
                cx,
            )
        });
        region.update(cx, |state, cx| {
            state.set_value(Some(SharedString::from("eu")), window, cx)
        });
        let seats = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value("12")
                .min(1.)
                .max(200.)
                .step(1.)
        });
        let filter = cx.new(|cx| InputState::new(window, cx).placeholder("Filter records…"));
        let edit_name = cx.new(|cx| InputState::new(window, cx).placeholder("Record name"));
        let subscriptions = [&filter, &edit_name]
            .into_iter()
            .map(|input| {
                cx.subscribe(input, |_, _, event, cx| {
                    if matches!(event, InputEvent::Change) {
                        cx.notify();
                    }
                })
            })
            .collect();

        Self {
            page: 0,
            sidebar: SidebarState::default(),
            pane_width: px(280.),
            dark: false,
            custom_palette: false,
            compact: false,
            large_text: false,
            long_content: false,
            disable_fields: false,
            workspace_name,
            region,
            seats,
            saved_settings: "Saved: Northwind workspace · Europe · 12 seats".into(),
            settings_errors: [false; 3],
            filter,
            edit_name,
            edit_record: None,
            editor: DialogHandle::new(false),
            records: vec![
                Record {
                    id: 101,
                    name: "Desktop shell".into(),
                    owner: "Mira",
                    status: "Ready",
                },
                Record {
                    id: 205,
                    name: "Billing settings".into(),
                    owner: "Sam",
                    status: "Review",
                },
                Record {
                    id: 309,
                    name: "Search results".into(),
                    owner: "Lee",
                    status: "Draft",
                },
                Record {
                    id: 412,
                    name: "Customer profile".into(),
                    owner: "Noor",
                    status: "Ready",
                },
            ],
            selected_record: 0,
            _subscriptions: subscriptions,
        }
    }

    fn apply_theme(&self, cx: &mut App) {
        let mut theme = if self.dark {
            UiTheme::neutral_dark()
        } else {
            UiTheme::neutral_light()
        };
        if self.custom_palette {
            // Application theme definition: one brand role feeds all component states.
            let brand = rgb(if self.dark { 0xa78bfa } else { 0x7c3aed });
            theme.colors.primary = brand;
            theme.colors.primary_foreground = rgb(0xffffff);
            theme.colors.ring = brand;
            theme.colors.sidebar_primary = brand;
            theme.colors.sidebar_primary_foreground = rgb(0xffffff);
        }
        theme.spacing.unit = px(if self.compact { 3. } else { 4. });
        theme.text_scale = if self.large_text { 1.25 } else { 1. };
        UiTheme::set(cx, theme);
    }

    fn save_settings(&mut self, cx: &mut Context<Self>) {
        let name = self.workspace_name.read(cx).value().to_string();
        let seats = self.seats.read(cx).value().to_string();
        let region = self
            .region
            .read(cx)
            .value()
            .map(|value| match value.as_ref() {
                "us" => "United States",
                "apac" => "Asia Pacific",
                _ => "Europe",
            })
            .unwrap_or("No region");
        self.settings_errors = [
            name.trim().is_empty(),
            self.region.read(cx).value().is_none(),
            seats
                .parse::<u32>()
                .map_or(true, |seats| !(1..=200).contains(&seats)),
        ];
        if !self.settings_errors.iter().any(|error| *error) {
            self.saved_settings = format!("Saved: {name} · {region} · {seats} seats");
        }
        cx.notify();
    }

    fn begin_edit(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.edit_record = Some(index);
        let name = self.records[index].name.clone();
        self.edit_name
            .update(cx, |input, cx| input.set_value(name, window, cx));
        self.editor.open(window, cx);
        cx.notify();
    }

    fn save_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let name = self.edit_name.read(cx).value().to_string();
        if name.trim().is_empty() {
            return;
        }
        if let Some(index) = self.edit_record {
            self.records[index].name = name;
        }
        self.editor.close(window, cx);
        cx.notify();
    }

    fn review_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = UiTheme::read(cx).clone();
        let compact = cx.listener(|this, value: &bool, _, cx| {
            this.compact = *value;
            this.apply_theme(cx);
            cx.notify();
        });
        let custom_palette = cx.listener(|this, value: &bool, _, cx| {
            this.custom_palette = *value;
            this.apply_theme(cx);
            cx.notify();
        });
        let large_text = cx.listener(|this, value: &bool, _, cx| {
            this.large_text = *value;
            this.apply_theme(cx);
            cx.notify();
        });
        let long_content = cx.listener(|this, value: &bool, _, cx| {
            this.long_content = *value;
            cx.notify();
        });
        let disable_fields = cx.listener(|this, value: &bool, _, cx| {
            this.disable_fields = *value;
            cx.notify();
        });
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(t.space(2.))
            .child(
                Button::new("review.theme")
                    .variant(ButtonVariant::Outline)
                    .label(if self.dark {
                        "Light theme"
                    } else {
                        "Dark theme"
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.dark = !this.dark;
                        this.apply_theme(cx);
                    })),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(t.space(1.5))
                    .child(
                        Switch::new("review.compact")
                            .checked(self.compact)
                            .aria_label("Compact spacing")
                            .on_change(move |value, _, window, cx| compact(&value, window, cx)),
                    )
                    .child("Compact"),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(t.space(1.5))
                    .child(
                        Switch::new("review.custom-palette")
                            .checked(self.custom_palette)
                            .aria_label("Custom palette")
                            .on_change(move |value, _, window, cx| {
                                custom_palette(&value, window, cx)
                            }),
                    )
                    .child("Custom palette"),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(t.space(1.5))
                    .child(
                        Switch::new("review.large-text")
                            .checked(self.large_text)
                            .aria_label("Larger text")
                            .on_change(move |value, _, window, cx| large_text(&value, window, cx)),
                    )
                    .child("Larger text"),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(t.space(1.5))
                    .child(
                        Switch::new("review.long-content")
                            .checked(self.long_content)
                            .aria_label("Long content")
                            .on_change(move |value, _, window, cx| {
                                long_content(&value, window, cx)
                            }),
                    )
                    .child("Long content"),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(t.space(1.5))
                    .child(
                        Switch::new("review.disabled-fields")
                            .checked(self.disable_fields)
                            .aria_label("Disable settings fields")
                            .on_change(move |value, _, window, cx| {
                                disable_fields(&value, window, cx)
                            }),
                    )
                    .child("Disabled fields"),
            )
    }

    fn settings(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = UiTheme::read(cx).clone();
        let long_description = if self.long_content {
            "Used in navigation, notifications, exports, shared invitations, and narrow review windows where translated copy may wrap onto several lines."
        } else {
            "Shown throughout the application."
        };
        form("settings.form", cx)
            .max_w(t.space(120.))
            .child(
                Field::new("settings.name", &self.workspace_name)
                    .label("Workspace name")
                    .description(long_description)
                    .required(true)
                    .disabled(self.disable_fields)
                    .when(self.settings_errors[0], |field| {
                        field.error("Enter a workspace name.")
                    }),
            )
            .child(
                Field::from_control("settings.region", Select::new(&self.region))
                    .label("Data region")
                    .description(
                        "The saved value starts as Europe and must remain visible on first render.",
                    )
                    .required(true)
                    .disabled(self.disable_fields)
                    .when(self.settings_errors[1], |field| {
                        field.error("Choose a data region.")
                    }),
            )
            .child(
                Field::from_control("settings.seats", NumberField::new(&self.seats))
                    .label("Seats")
                    .description("Between 1 and 200.")
                    .disabled(self.disable_fields)
                    .when(self.settings_errors[2], |field| {
                        field.error("Enter a whole number from 1 to 200.")
                    }),
            )
            .child(
                Button::new("settings.save")
                    .label("Save settings")
                    .on_click(cx.listener(|this, _, _, cx| this.save_settings(cx))),
            )
            .child(
                div()
                    .id("settings.saved")
                    .role(Role::Status)
                    .aria_label(self.saved_settings.clone())
                    .text_color(t.colors.muted_foreground)
                    .child(self.saved_settings.clone()),
            )
    }

    fn records(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = UiTheme::read(cx).clone();
        let query = self.filter.read(cx).value().to_lowercase();
        let mut rows = div().flex().flex_col().gap(t.space(2.));
        let mut visible = 0;
        for (index, record) in self.records.iter().enumerate() {
            if !record.name.to_lowercase().contains(&query)
                && !record.owner.to_lowercase().contains(&query)
            {
                continue;
            }
            visible += 1;
            rows = rows.child(
                div()
                    .id(("record", record.id))
                    .flex()
                    .items_center()
                    .gap(t.space(3.))
                    .rounded(t.radius.lg)
                    .border_1()
                    .border_color(t.colors.border)
                    .p(t.space(3.))
                    .child(div().flex_1().min_w_0().child(record.name.clone()))
                    .child(
                        div()
                            .text_color(t.colors.muted_foreground)
                            .child(format!("{} · {}", record.owner, record.status)),
                    )
                    .child(
                        Button::new(("record.edit", record.id))
                            .variant(ButtonVariant::Outline)
                            .label("Edit…")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.begin_edit(index, window, cx)
                            })),
                    ),
            );
        }
        if visible == 0 {
            rows = rows.child(
                div()
                    .text_color(t.colors.muted_foreground)
                    .child("No records match this filter."),
            );
        }
        let editor = self.editor.clone();
        let edit_empty = self.edit_name.read(cx).value().trim().is_empty();
        let popup = dialog_popup("record.editor.popup", "Edit record", cx)
            .child(dialog_close("record.editor.close", cx))
            .child(dialog_title("record.editor.title", cx).child("Edit record"))
            .child(
                dialog_description("record.editor.description", cx)
                    .child("Change the row name, then save or press Escape to cancel."),
            )
            .child(Field::new("record.editor.name", &self.edit_name).label("Name"))
            .child(
                dialog_footer(cx)
                    .child(dialog_action("record.editor.cancel", &editor, cx).label("Cancel"))
                    .child(
                        Button::new("record.editor.save")
                            .label("Save")
                            .disabled(edit_empty)
                            .on_click(
                                cx.listener(|this, _, window, cx| this.save_edit(window, cx)),
                            ),
                    ),
            );
        div()
            .flex()
            .flex_col()
            .gap(t.space(4.))
            .child(Input::new(&self.filter).aria_label("Filter records"))
            .child(rows)
            .child(dialog("record.editor", &editor, popup, window, cx))
    }

    fn split_workspace(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = UiTheme::read(cx).clone();
        let projects = ["Desktop app", "Documentation", "Release tooling"];
        let mut list = div()
            .size_full()
            .flex()
            .flex_col()
            .p(t.space(3.))
            .gap(t.space(1.));
        for (index, project) in projects.into_iter().enumerate() {
            list = list.child(
                SidebarItem::new(("project", index), project)
                    .selected(index == self.selected_record)
                    .on_activate(cx.listener(move |this, _, _, cx| {
                        this.selected_record = index;
                        cx.notify();
                    })),
            );
        }
        let project = projects[self.selected_record];
        let detail = div()
            .id("workspace.detail")
            .size_full()
            .overflow_y_scroll()
            .p(t.space(5.))
            .flex()
            .flex_col()
            .gap(t.space(3.))
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_size(t.text(20.))
                    .child(project),
            )
            .child("This detail pane keeps the selected project visible while the list width changes.")
            .when(self.long_content, |detail| {
                detail.child("A deliberately longer paragraph checks wrapping, hierarchy, selection, and readable spacing when the application window becomes narrow or the text scale increases. The split remains keyboard adjustable through its separator.")
            });
        Resizable::new(
            "workspace.split",
            "Resize project list",
            self.pane_width,
            list,
            detail,
            cx.listener(|this, width, _, cx| {
                this.pane_width = *width;
                cx.notify();
            }),
        )
        .size_full()
        .first_limits(PaneLimits::new(t.space(40.), t.space(90.)))
        .second_limits(PaneLimits::new(t.space(64.), t.space(180.)))
    }
}

impl Render for AdoptionConsumer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = UiTheme::read(cx).clone();
        let mobile = sidebar_is_mobile(window);
        let collapsed = self
            .sidebar
            .icon_collapsed(mobile, SidebarCollapsible::Icon);
        let pages = [
            ("Settings", LucideIcon::Settings),
            ("Records", LucideIcon::Folder),
            ("Workspace", LucideIcon::LayoutDashboard),
        ];
        let mut menu = sidebar_menu();
        for (index, (label, icon)) in pages.into_iter().enumerate() {
            menu = menu.child(
                SidebarItem::new(("journey", index), label)
                    .icon(
                        lucide(icon)
                            .size(t.space(4.))
                            .text_color(t.colors.sidebar_foreground),
                    )
                    .collapsed(collapsed)
                    .selected(index == self.page)
                    .on_activate(cx.listener(move |this, _, _, cx| {
                        this.page = index;
                        this.sidebar.mobile_open = false;
                        cx.notify();
                    })),
            );
        }
        let navigation = Sidebar::new("consumer.navigation", "Readiness journeys")
            .header(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .when(!collapsed, |header| header.child("gpuicn consumer"))
                    .when(collapsed, |header| header.child("g")),
            )
            .when(!collapsed, |navigation| {
                navigation.child(sidebar_group_label("Journeys", cx))
            })
            .child(menu)
            .footer(
                div()
                    .text_size(t.text(12.))
                    .text_color(t.colors.sidebar_foreground)
                    .when(!collapsed, |footer| footer.child("Local registry branch")),
            );

        let toggle = cx.listener(move |this, _, _, cx| {
            this.sidebar.toggle(mobile);
            cx.notify();
        });
        let page = match self.page {
            1 => self.records(window, cx).into_any_element(),
            2 => self.split_workspace(cx).into_any_element(),
            _ => self.settings(cx).into_any_element(),
        };
        let content = div()
            .size_full()
            .min_w_0()
            .min_h_0()
            .flex()
            .flex_col()
            .bg(t.colors.background)
            .text_color(t.colors.foreground)
            .font_family(t.fonts.body.clone())
            .child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .gap(t.space(3.))
                    .border_b_1()
                    .border_color(t.colors.border)
                    .p(t.space(3.))
                    .child(sidebar_trigger("consumer.sidebar.toggle", toggle, cx))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(pages[self.page].0),
                    )
                    .child(div().flex_1())
                    .child(self.review_toolbar(cx)),
            )
            .child(
                div()
                    .id("consumer.page")
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .overflow_y_scroll()
                    .p(t.space(5.))
                    .child(page),
            );

        SidebarLayout::new(
            "consumer.layout",
            self.sidebar,
            mobile,
            navigation,
            content,
            cx.listener(|this, state: &SidebarState, _, cx| {
                this.sidebar = *state;
                cx.notify();
            }),
        )
        .collapsible(SidebarCollapsible::Icon)
        .rail(true)
    }
}
