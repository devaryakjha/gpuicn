mod data;

use super::*;
use data::{Data, Store, Task};
use gpui::{ElementId, FontWeight, Pixels, Rgba, SharedString, rgb};
use gpuicn::{
    resizable::{PaneLimits, Resizable},
    sidebar::{
        Sidebar, SidebarItem, SidebarLayout, SidebarState, sidebar_group_label, sidebar_trigger,
    },
};

gpui::actions!(workspace, [Quit, OpenGallery]);

pub(super) fn launch(cx: &mut App) {
    UiTheme::set(cx, workspace_theme(UiTheme::read(cx).mode));
    cx.bind_keys([gpui::KeyBinding::new("cmd-q", Quit, None)]);
    cx.set_menus([gpui::Menu::new("gpuicn Workspace").items([
        gpui::MenuItem::action("Component Gallery", OpenGallery),
        gpui::MenuItem::action("Quit gpuicn Workspace", Quit),
    ])]);
    cx.on_window_closed(|cx, _| {
        if cx.windows().is_empty() {
            cx.quit();
        }
    })
    .detach();
    let path = std::env::var_os("GPUICN_WORKSPACE_DATA")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(std::env::var_os("HOME").expect("home directory"))
                .join("Library/Application Support/gpuicn Workspace/workspace.json")
        });
    let (store, data, error) = Store::open(path);
    let bounds = Bounds::centered(None, size(px(1240.), px(800.)), cx);
    let workspace = cx
        .open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(1000.), px(680.))),
                titlebar: Some(gpui::TitlebarOptions {
                    appears_transparent: true,
                    traffic_light_position: Some(gpui::point(px(20.), px(20.))),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| {
                window.set_window_title("gpuicn Workspace");
                let entity = cx.new(|cx| Workspace::new(store, data, error, cx));
                let weak = entity.downgrade();
                window.on_window_should_close(cx, move |_, cx| {
                    weak.update(cx, |this, cx| {
                        if this.details_changed() || this.unsaved {
                            this.pending = Some(Destination::Quit);
                            cx.notify();
                            false
                        } else {
                            true
                        }
                    })
                    .unwrap_or(true)
                });
                entity
            },
        )
        .expect("open workspace");
    cx.on_action(|_: &OpenGallery, cx| super::gallery::open(cx));
    cx.on_action(move |_: &Quit, cx| request_quit(workspace, cx));
    cx.activate(true);
}

fn request_quit(workspace: gpui::WindowHandle<Workspace>, cx: &mut App) {
    // Global key actions still run inside the active window's update.
    // Defer until that borrow ends so an update error cannot bypass the save guard.
    cx.defer(move |cx| {
        if workspace
            .update(cx, |this, window, cx| {
                this.navigate(Destination::Quit, cx);
                if this.pending.is_some() {
                    window.activate_window();
                }
            })
            .is_err()
        {
            cx.quit();
        }
    });
}

#[derive(Clone, Copy)]
enum Draft {
    Search,
    Task,
    Project,
    Message,
    Title,
    Note,
}

#[derive(Clone, Copy)]
enum Destination {
    Task(u64),
    Project(u64),
    AddTask,
    Quit,
}

struct Workspace {
    store: Store,
    data: Data,
    project: u64,
    selected: Option<u64>,
    mode: &'static str,
    filter: &'static str,
    search: String,
    task_draft: String,
    project_draft: String,
    message_drafts: std::collections::HashMap<u64, String>,
    chat_scroll: gpui::ScrollHandle,
    title_draft: String,
    note_draft: String,
    error: Option<String>,
    notice: String,
    last_deleted: Option<Task>,
    pending: Option<Destination>,
    input_epoch: u64,
    message_epoch: u64,
    unsaved: bool,
    list_width: Pixels,
    adding_task: bool,
    task_focus: gpui::FocusHandle,
    adding_project: bool,
    editing_note: bool,
    editing_title: bool,
    sidebar: SidebarState,
}

impl Workspace {
    fn new(store: Store, data: Data, error: Option<String>, cx: &mut Context<Self>) -> Self {
        let project = data.projects[0].id;
        let mut view = Self {
            store,
            data,
            project,
            selected: None,
            mode: "tasks",
            filter: "open",
            search: String::new(),
            task_draft: String::new(),
            project_draft: String::new(),
            message_drafts: Default::default(),
            chat_scroll: gpui::ScrollHandle::new(),
            title_draft: String::new(),
            note_draft: String::new(),
            error,
            notice: "Saved on this Mac".into(),
            last_deleted: None,
            pending: None,
            input_epoch: 0,
            message_epoch: 0,
            unsaved: false,
            list_width: px(560.),
            adding_task: false,
            task_focus: cx.focus_handle(),
            sidebar: SidebarState::default(),
            adding_project: false,
            editing_note: false,
            editing_title: false,
        };
        view.select_task(
            view.data
                .tasks
                .iter()
                .find(|t| t.project == project && !t.done)
                .map(|t| t.id),
        );
        view
    }

    fn persist(&mut self, message: &str, cx: &mut Context<Self>) {
        match self.store.save(&self.data) {
            Ok(()) => {
                self.unsaved = false;
                self.error = None;
                self.notice = message.into();
            }
            Err(error) => {
                self.unsaved = true;
                self.error = Some(format!("Changes are only in memory: {error}"));
            }
        }
        cx.notify();
    }

    fn select_task(&mut self, id: Option<u64>) {
        self.input_epoch += 1;
        self.message_epoch += 1;
        self.selected = id;
        self.editing_note = false;
        self.editing_title = false;
        let task = id.and_then(|id| self.data.tasks.iter().find(|task| task.id == id));
        self.title_draft = task.map_or(String::new(), |task| task.title.clone());
        self.note_draft = task.map_or(String::new(), |task| task.note.clone());
    }

    fn details_changed(&self) -> bool {
        self.selected
            .and_then(|id| self.data.tasks.iter().find(|t| t.id == id))
            .is_some_and(|t| t.title != self.title_draft || t.note != self.note_draft)
    }

    fn navigate(&mut self, destination: Destination, cx: &mut Context<Self>) {
        if self.details_changed() || (matches!(destination, Destination::Quit) && self.unsaved) {
            self.pending = Some(destination);
        } else {
            self.continue_to(destination, cx);
        }
        cx.notify();
    }

    fn continue_to(&mut self, destination: Destination, cx: &mut Context<Self>) {
        match destination {
            Destination::Task(id) => {
                if let Some(task) = self.data.tasks.iter().find(|t| t.id == id) {
                    self.project = task.project;
                }
                self.select_task(Some(id));
            }
            Destination::Project(id) => self.select_project(id, cx),
            Destination::AddTask => self.add_task(cx),
            Destination::Quit => cx.quit(),
        }
    }

    fn save_details(&mut self, cx: &mut Context<Self>) -> bool {
        if self.selected.is_some()
            && (self.title_draft.trim().is_empty()
                || self.title_draft.len() > 500
                || self.note_draft.len() > 10_000)
        {
            self.error =
                Some("Enter a title up to 500 bytes and a note up to 10,000 bytes.".into());
            cx.notify();
            return false;
        }
        let was_unsaved = self.unsaved;
        let previous = self
            .selected
            .and_then(|id| self.data.tasks.iter().find(|task| task.id == id))
            .cloned();
        if let Some(task) = self
            .selected
            .and_then(|id| self.data.tasks.iter_mut().find(|task| task.id == id))
        {
            task.title = self.title_draft.trim().into();
            task.note = self.note_draft.trim().into();
        }
        self.persist("Task updated", cx);
        if self.unsaved {
            // A failed Save must leave drafts editable and discardable.
            if let Some(previous) = previous
                && let Some(task) = self
                    .data
                    .tasks
                    .iter_mut()
                    .find(|task| task.id == previous.id)
            {
                *task = previous;
            }
            self.unsaved = was_unsaved;
            return false;
        }
        self.select_task(self.selected);
        true
    }

    fn select_project(&mut self, id: u64, cx: &mut Context<Self>) {
        self.project = id;
        self.chat_scroll.scroll_to_bottom();
        self.search.clear();
        self.select_task(
            self.data
                .tasks
                .iter()
                .find(|t| t.project == id && !t.done)
                .map(|t| t.id),
        );
        cx.notify();
    }

    fn epoch(&self, field: Draft) -> u64 {
        if matches!(field, Draft::Message) {
            self.message_epoch
        } else {
            self.input_epoch
        }
    }

    fn input(
        &self,
        id: impl Into<ElementId>,
        label: &'static str,
        value: &str,
        field: Draft,
        cx: &mut Context<Self>,
    ) -> gpui::Div {
        let project = self.project;
        let epoch = self.epoch(field);
        let id: ElementId = id.into();
        let theme = UiTheme::read(cx).clone();
        let input = Input::new((id, epoch.to_string()))
            .aria_label(label)
            .placeholder(label)
            .value(value.to_owned())
            .h(px(36.))
            .bg(theme.colors.card)
            .when(matches!(field, Draft::Search | Draft::Task), |input| {
                input.h(px(32.))
            })
            .when(matches!(field, Draft::Task), |input| {
                input.focus_handle(self.task_focus.clone())
            })
            .auto_focus(matches!(
                field,
                Draft::Task | Draft::Project | Draft::Note | Draft::Title | Draft::Message
            ))
            .when(matches!(field, Draft::Title), |input| {
                input
                    .h(px(44.))
                    .text_size(px(22.))
                    .font_weight(FontWeight::MEDIUM)
            })
            .on_submit(
                cx.listener(move |this, value: &SharedString, _, cx| match field {
                    Draft::Task => {
                        this.task_draft = value.to_string();
                        this.navigate(Destination::AddTask, cx);
                    }
                    Draft::Project => {
                        this.project_draft = value.to_string();
                        this.create_project(cx);
                    }
                    Draft::Message => {
                        this.message_drafts.insert(project, value.to_string());
                        this.send_message(cx);
                    }
                    Draft::Title => {
                        this.title_draft = value.to_string();
                        this.save_details(cx);
                    }
                    Draft::Note => {
                        this.note_draft = value.to_string();
                        this.save_details(cx);
                    }
                    Draft::Search => {}
                }),
            )
            .on_change(cx.listener(move |this, value: &SharedString, _, cx| {
                let draft = match field {
                    Draft::Search => &mut this.search,
                    Draft::Task => &mut this.task_draft,
                    Draft::Project => &mut this.project_draft,
                    Draft::Message => this.message_drafts.entry(project).or_default(),
                    Draft::Title => &mut this.title_draft,
                    Draft::Note => &mut this.note_draft,
                };
                *draft = value.to_string();
                cx.notify();
            }));
        div()
            .w_full()
            .debug_selector(move || label.to_owned())
            .child(input)
    }

    fn add_task(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.data.add_task(self.project, &self.task_draft) {
            self.task_draft.clear();
            self.adding_task = false;
            self.filter = "open";
            self.select_task(Some(id));
            self.persist("Task added", cx);
        } else {
            self.error = Some("Enter a task title between 1 and 500 bytes.".into());
            cx.notify();
        }
    }

    fn complete(&mut self, id: u64, done: bool, cx: &mut Context<Self>) {
        if let Some(task) = self.data.tasks.iter_mut().find(|t| t.id == id) {
            task.done = done;
            self.persist(
                if done {
                    "Task completed"
                } else {
                    "Task reopened"
                },
                cx,
            );
        }
    }

    fn create_project(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.data.add_project(&self.project_draft) {
            self.project_draft.clear();
            self.adding_project = false;
            self.input_epoch += 1;
            self.navigate(Destination::Project(id), cx);
            self.persist("Project created", cx);
        } else {
            self.error = Some("Choose a unique project name between 1 and 120 bytes.".into());
            cx.notify();
        }
    }

    fn delete_task(&mut self, id: u64, cx: &mut Context<Self>) {
        if let Some(index) = self.data.tasks.iter().position(|task| task.id == id) {
            self.last_deleted = Some(self.data.tasks.remove(index));
            self.select_task(None);
            self.persist("Task deleted", cx);
        }
    }

    fn tasks(&self, cx: &mut Context<Self>) -> gpui::Div {
        let theme = UiTheme::read(cx).clone();
        let c = theme.colors;
        let query = self.search.trim().to_lowercase();
        let tasks: Vec<_> = self
            .data
            .tasks
            .iter()
            .filter(|task| task.project == self.project)
            .filter(|task| match self.filter {
                "open" => !task.done,
                "done" => task.done,
                _ => true,
            })
            .filter(|task| {
                task.title.to_lowercase().contains(&query)
                    || task.note.to_lowercase().contains(&query)
            })
            .collect();
        let mut rows = div().flex().flex_col().p(px(8.));
        for task in &tasks {
            let id = task.id;
            let selected = self.selected == Some(id);
            let view = cx.entity().downgrade();
            let row_view = view.clone();
            rows = rows.child(
                div()
                    .relative()
                    .w_full()
                    .h(px(64.))
                    .child(
                        Toggle::new(("workspace.task", id))
                            .aria_label(format!("Open {}", task.title))
                            .pressed(selected)
                            .w_full()
                            .h_full()
                            .pl(px(44.))
                            .pr(px(14.))
                            .justify_start()
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .flex()
                                    .flex_col()
                                    .gap(px(7.))
                                    .child(
                                        div()
                                            .truncate()
                                            .text_size(px(14.))
                                            .font_weight(FontWeight::MEDIUM)
                                            .when(task.done, |el| {
                                                el.text_color(c.muted_foreground).line_through()
                                            })
                                            .child(task.title.clone()),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(px(8.))
                                            .text_size(px(11.))
                                            .font_weight(FontWeight::NORMAL)
                                            .text_color(c.muted_foreground)
                                            .child(
                                                div()
                                                    .font_family(theme.fonts.mono.clone())
                                                    .child(format!("TASK-{id:03}")),
                                            )
                                            .child(div().size(px(3.)).rounded_full().bg(c.border))
                                            .child(if task.done {
                                                "Completed"
                                            } else if task.priority {
                                                "High priority"
                                            } else {
                                                "To do"
                                            }),
                                    ),
                            )
                            .when(task.priority && !task.done, |el| {
                                el.child(icon(LucideIcon::Flag, 14., c.muted_foreground))
                            })
                            .on_pressed_change(move |_, _, _, cx| {
                                let _ = row_view.update(cx, |this, cx| {
                                    this.navigate(Destination::Task(id), cx)
                                });
                            }),
                    )
                    .child(
                        div().absolute().left(px(14.)).top(px(24.)).child(
                            Checkbox::new(("workspace.done", id))
                                .checked(task.done)
                                .aria_label(format!("Complete {}", task.title))
                                .on_checked_change(move |done, _, _, cx| {
                                    let _ = view.update(cx, |this, cx| this.complete(id, done, cx));
                                }),
                        ),
                    ),
            );
        }
        if tasks.is_empty() {
            rows = rows.child(
                div()
                    .py(px(64.))
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(12.))
                    .child(icon(
                        if query.is_empty() {
                            LucideIcon::CircleCheck
                        } else {
                            LucideIcon::Search
                        },
                        28.,
                        c.muted_foreground,
                    ))
                    .child(
                        div()
                            .font_weight(FontWeight::MEDIUM)
                            .child(if query.is_empty() {
                                "No tasks in this view"
                            } else {
                                "No matching tasks"
                            }),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(c.muted_foreground)
                            .child(if query.is_empty() {
                                "Add a task or choose another filter."
                            } else {
                                "Try another search or clear the filter."
                            }),
                    ),
            );
        }
        let filters = div().flex().items_center().gap(px(2.)).children(
            [("open", "To do"), ("done", "Done"), ("all", "All")]
                .into_iter()
                .map(|(value, label)| {
                    Button::new(format!("workspace.filter.{value}"))
                        .label(label)
                        .size(ButtonSize::Sm)
                        .aria_label(format!("Show {label} tasks"))
                        .variant(if self.filter == value {
                            ButtonVariant::Secondary
                        } else {
                            ButtonVariant::Ghost
                        })
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.filter = value;
                            cx.notify();
                        }))
                }),
        );
        let list = div()
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .bg(c.card)
            .rounded(px(12.))
            .border_1()
            .border_color(c.border)
            .overflow_hidden()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px(px(18.))
                    .h(px(58.))
                    .flex_shrink_0()
                    .border_b_1()
                    .border_color(c.border)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .font_weight(FontWeight::MEDIUM)
                            .child("Tasks")
                            .child(tag(tasks.len().to_string(), c.muted_foreground, c.muted)),
                    )
                    .child(filters),
            )
            .child(div().px(px(18.)).py(px(12.)).child(self.input(
                "workspace.search",
                "Search tasks",
                &self.search,
                Draft::Search,
                cx,
            )))
            .child(
                div().flex_1().min_h_0().overflow_hidden().child(
                    scroll_area(cx)
                        .id("workspace.task-scroll")
                        .size_full()
                        .child(
                            scroll_area_viewport(cx)
                                .child(scroll_area_content(cx).w_full().child(rows)),
                        )
                        .child(
                            scroll_area_scrollbar(ScrollAreaOrientation::Vertical, cx)
                                .child(scroll_area_thumb(cx)),
                        ),
                ),
            )
            .child(
                div()
                    .px(px(18.))
                    .py(px(10.))
                    .flex_shrink_0()
                    .border_t_1()
                    .border_color(c.border)
                    .when(self.adding_task, |footer| {
                        footer.child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .child(div().flex_1().min_w_0().child(self.input(
                                    "workspace.new-task",
                                    "Add a task…",
                                    &self.task_draft,
                                    Draft::Task,
                                    cx,
                                )))
                                .child(
                                    Button::new("workspace.add-task")
                                        .aria_label("Add task")
                                        .label("Add")
                                        .disabled(self.task_draft.trim().is_empty())
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.navigate(Destination::AddTask, cx)
                                        })),
                                )
                                .child(
                                    Button::new("workspace.close-composer")
                                        .aria_label("Close task composer")
                                        .variant(ButtonVariant::Ghost)
                                        .size(ButtonSize::Icon)
                                        .child(icon(LucideIcon::X, 16., c.muted_foreground))
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.adding_task = false;
                                            cx.notify();
                                        })),
                                ),
                        )
                    })
                    .when(!self.adding_task, |footer| {
                        footer.child(
                            Button::new("workspace.new-task-inline")
                                .aria_label("Open task composer")
                                .variant(ButtonVariant::Ghost)
                                .justify_start()
                                .w_full()
                                .child(icon(LucideIcon::Plus, 15., c.muted_foreground))
                                .label("Add a task")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.adding_task = !this.adding_task;
                                    cx.notify();
                                })),
                        )
                    }),
            );
        div().size_full().p(px(20.)).child(
            Resizable::new(
                "workspace.task-split",
                "Resize tasks and details",
                self.list_width,
                list,
                self.details(cx),
                cx.listener(|this, value, _, cx| {
                    this.list_width = *value;
                    cx.notify();
                }),
            )
            .first_limits(PaneLimits::new(px(340.), px(1000.)))
            .second_limits(PaneLimits::new(px(300.), px(600.))),
        )
    }

    fn details(&self, cx: &mut Context<Self>) -> gpui::Div {
        let theme = UiTheme::read(cx).clone();
        let c = theme.colors;
        let mut content = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(c.card)
            .rounded(px(12.))
            .border_1()
            .border_color(c.border)
            .overflow_hidden();
        if let Some(task) = self
            .selected
            .and_then(|id| self.data.tasks.iter().find(|task| task.id == id))
        {
            let id = task.id;
            let done = task.done;
            let menu_complete = cx.entity().downgrade();
            let menu_delete = menu_complete.clone();
            let changed = self.details_changed();
            content = content.child(div().h(px(58.)).px(px(20.)).flex_shrink_0().flex().items_center().justify_between().border_b_1().border_color(c.border)
                .child(div().font_family(theme.fonts.mono.clone()).text_size(px(11.)).text_color(c.muted_foreground).child(format!("TASK-{id:03}")))
                .child(menu_root::<()>("workspace.task-menu")
                    .child(menu_trigger("workspace.task-menu-trigger", cx).aria_label("Task actions").child(icon(LucideIcon::Ellipsis, 16., c.muted_foreground)))
                    .child(menu_portal().child(menu_positioner(cx).child(menu_popup("workspace.task-menu-popup", cx)
                        .child(menu_item("workspace.menu.complete", cx).child(if done { "Reopen task" } else { "Mark complete" })
                            .on_click(move |_, cx| { let _ = menu_complete.update(cx, |this, cx| this.complete(id, !done, cx)); }))
                        .child(menu_separator(cx))
                        .child(menu_item("workspace.menu.delete", cx).child("Delete task")
                            .on_click(move |_, cx| { let _ = menu_delete.update(cx, |this, cx| this.delete_task(id, cx)); })))))))
                .child(div().flex_1().min_h_0().overflow_hidden().child(scroll_area(cx).id("workspace.details-scroll").size_full()
                    .child(scroll_area_viewport(cx).child(scroll_area_content(cx).w_full().child(
                        div().p(px(24.)).flex().flex_col().gap(px(22.))
                            .when(self.editing_title, |el| el.child(self.input(("workspace.title", id), "Task title", &self.title_draft, Draft::Title, cx)))
                            .when(!self.editing_title, |el| el.child(div().flex().items_start().gap(px(8.))
                                .child(div().flex_1().min_w_0().text_size(px(24.)).line_height(px(31.)).font_weight(FontWeight::MEDIUM).child(task.title.clone()))
                                .child(Button::new("workspace.edit-title").aria_label("Edit task title").variant(ButtonVariant::Ghost).size(ButtonSize::IconSm)
                                    .child(icon(LucideIcon::Pencil, 13., c.muted_foreground))
                                    .on_click(cx.listener(|this, _, _, cx| { this.editing_title = true; cx.notify(); })))))
                            .child(div().flex().items_center().justify_between().text_size(px(12.))
                                .child(div().text_color(c.muted_foreground).child("Status"))
                                .child(Button::new("workspace.detail-complete").aria_label(if done { "Reopen task" } else { "Mark task complete" }).variant(ButtonVariant::Secondary).size(ButtonSize::Sm)
                                    .child(icon(if done { LucideIcon::CircleCheck } else { LucideIcon::Circle }, 13., if done { c.chart_2 } else { c.muted_foreground }))
                                    .child(if done { "Completed" } else { "To do" })
                                    .on_click(cx.listener(move |this, _, _, cx| this.complete(id, !done, cx)))))
                            .child(div().flex().items_center().justify_between().text_size(px(12.))
                                .child(div().text_color(c.muted_foreground).child("Priority"))
                                .child(Button::new("workspace.priority").aria_label("Toggle task priority").variant(ButtonVariant::Ghost).size(ButtonSize::Sm)
                                    .child(icon(LucideIcon::Flag, 13., if task.priority { c.primary } else { c.muted_foreground }))
                                    .child(if task.priority { "High priority" } else { "Normal" })
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        if let Some(task) = this.data.tasks.iter_mut().find(|t| t.id == id) { task.priority = !task.priority; }
                                        this.persist("Priority updated", cx);
                                    }))))
                            .child(div().flex().items_center().justify_between().text_size(px(12.))
                                .child(div().text_color(c.muted_foreground).child("Assigned to"))
                                .child(div().flex().items_center().gap(px(8.)).child(Avatar::new("workspace.assignee").size(AvatarSize::Sm).child("Y")).child("You")))
                            .child(Separator::new("workspace.notes-separator"))
                            .child(div().flex().flex_col().gap(px(12.))
                                .child(div().flex().items_center().justify_between().font_weight(FontWeight::MEDIUM).text_size(px(12.)).child("Notes")
                                    .child(Button::new("workspace.edit-note").aria_label("Edit task note").variant(ButtonVariant::Ghost).size(ButtonSize::Sm)
                                        .child(icon(LucideIcon::Pencil, 13., c.muted_foreground)).label("Edit")
                                        .on_click(cx.listener(|this, _, _, cx| { this.editing_note = !this.editing_note; cx.notify(); }))))
                                .child(div().text_size(px(13.)).line_height(px(22.)).text_color(c.muted_foreground)
                                    .child(if self.note_draft.is_empty() { "A good place for the details, links, and little things worth remembering.".into() } else { self.note_draft.clone() }))
                                .when(self.editing_note, |el| el.child(self.input(("workspace.note", id), "Add a note", &self.note_draft, Draft::Note, cx))))
                            .when(changed, |el| el.child(Button::new("workspace.save-task").aria_label("Save task changes").label("Save changes")
                                .disabled(self.title_draft.trim().is_empty()).on_click(cx.listener(|this, _, _, cx| { this.save_details(cx); }))))
                    )))
                    .child(scroll_area_scrollbar(ScrollAreaOrientation::Vertical, cx).child(scroll_area_thumb(cx)))))
                .child(div().px(px(24.)).py(px(16.)).flex().items_center().gap(px(8.)).border_t_1().border_color(c.border)
                    .child(icon(if changed { LucideIcon::Circle } else { LucideIcon::Check }, 13., c.muted_foreground))
                    .child(div().text_size(px(11.)).text_color(c.muted_foreground).child(if changed || self.unsaved { "Unsaved changes" } else { "Saved on this Mac" })));
        } else {
            content = content
                .items_center()
                .justify_center()
                .gap(px(12.))
                .p(px(28.))
                .child(icon(LucideIcon::MousePointer2, 28., c.muted_foreground))
                .child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .child("The details live here"),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(c.muted_foreground)
                        .child("Choose a task to take a closer look."),
                );
        }
        content
    }

    fn send_message(&mut self, cx: &mut Context<Self>) {
        if self
            .data
            .send(
                self.project,
                self.message_drafts
                    .get(&self.project)
                    .map(String::as_str)
                    .unwrap_or_default(),
            )
            .is_some()
        {
            self.message_drafts.remove(&self.project);
            self.chat_scroll.scroll_to_bottom();
            self.persist("Message saved", cx);
        } else {
            self.error = Some("Messages must be between 1 and 10,000 bytes.".into());
            cx.notify();
        }
    }

    fn chat(&self, cx: &mut Context<Self>) -> gpui::Div {
        let c = UiTheme::read(cx).colors;
        let messages: Vec<_> = self
            .data
            .messages
            .iter()
            .filter(|m| m.project == self.project)
            .collect();
        let mut feed = div().flex().flex_col().gap(px(28.)).p(px(32.));
        feed = feed.child(div().py(px(16.)).flex().flex_col().gap(px(12.))
            .child(div().size(px(48.)).rounded(px(14.)).bg(c.accent).flex().items_center().justify_center().child(icon(LucideIcon::MessagesSquare, 24., c.primary)))
            .child(div().text_size(px(24.)).font_weight(FontWeight::MEDIUM).child("Every idea starts somewhere."))
            .child(div().max_w(px(440.)).text_size(px(13.)).line_height(px(22.)).text_color(c.muted_foreground).child("Keep the thinking close to the work. Leave yourself an update, collect an idea, or note what comes next."))
            .child(div().flex().items_center().gap(px(6.)).text_size(px(11.)).text_color(c.muted_foreground)
                .child(icon(LucideIcon::LockKeyhole, 12., c.muted_foreground)).child("Only you · Stored on this Mac")));
        if !messages.is_empty() {
            feed = feed.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .text_size(px(11.))
                    .text_color(c.muted_foreground)
                    .child(div().flex_1().h(px(1.)).bg(c.border))
                    .child("Project conversation")
                    .child(div().flex_1().h(px(1.)).bg(c.border)),
            );
        }
        for message in messages {
            feed = feed.child(
                div()
                    .flex()
                    .items_start()
                    .gap(px(12.))
                    .child(Avatar::new(("workspace.avatar", message.id)).child("Y"))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    .text_size(px(13.))
                                    .font_weight(FontWeight::MEDIUM)
                                    .child("You")
                                    .child(tag("Local note", c.muted_foreground, c.muted)),
                            )
                            .child(
                                div()
                                    .rounded(px(12.))
                                    .rounded_tl(px(2.))
                                    .bg(c.muted)
                                    .p(px(16.))
                                    .text_size(px(14.))
                                    .line_height(px(23.))
                                    .child(message.body.clone()),
                            ),
                    ),
            );
        }
        let composer = div()
            .p(px(20.))
            .flex()
            .flex_col()
            .gap(px(10.))
            .border_t_1()
            .border_color(c.border)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .child(
                        div().flex_1().min_w_0().child(
                            self.input(
                                "workspace.compose",
                                "Write a local message…",
                                self.message_drafts
                                    .get(&self.project)
                                    .map(String::as_str)
                                    .unwrap_or_default(),
                                Draft::Message,
                                cx,
                            ),
                        ),
                    )
                    .child(
                        Button::new("workspace.send")
                            .aria_label("Send local message")
                            .h(px(36.))
                            .child(icon(LucideIcon::ArrowUp, 16., c.primary_foreground))
                            .label("Send")
                            .disabled(
                                self.message_drafts
                                    .get(&self.project)
                                    .is_none_or(|draft| draft.trim().is_empty()),
                            )
                            .on_click(cx.listener(|this, _, _, cx| this.send_message(cx))),
                    ),
            )
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(c.muted_foreground)
                    .child("Enter to send. Messages stay private to this workspace."),
            );
        div().size_full().p(px(20.)).child(div().size_full().flex().gap(px(20.))
            .child(div().flex_1().min_w_0().h_full().bg(c.card).border_1().border_color(c.border).rounded(px(12.)).overflow_hidden().flex().flex_col()
                .child(div().relative().flex_1().min_h_0().overflow_hidden()
                    .child(div().id(("workspace.messages", self.project)).size_full().overflow_y_scroll().track_scroll(&self.chat_scroll).child(feed.w_full()))
                    .child(gpuicn::scroll_area::scroll_area_scrollbar_for("workspace.chat-scrollbar", &self.chat_scroll, cx)))
                .child(composer))
            .child(div().w(px(230.)).flex_shrink_0().p(px(12.)).flex().flex_col().gap(px(16.))
                .child(div().text_size(px(12.)).font_weight(FontWeight::MEDIUM).child("A little context"))
                .child(div().text_size(px(13.)).line_height(px(22.)).text_color(c.muted_foreground).child("A quiet space for the notes that don't fit in a task."))
                .child(Separator::new("workspace.chat-separator"))
                .child(div().flex().items_center().gap(px(8.)).text_size(px(12.)).child(icon(LucideIcon::HardDrive, 15., c.muted_foreground)).child("Available offline"))
                .child(div().flex().items_center().gap(px(8.)).text_size(px(12.)).child(icon(LucideIcon::UserRound, 15., c.muted_foreground)).child("One participant: you"))
                .child(div().text_size(px(12.)).line_height(px(20.)).text_color(c.muted_foreground).child("This is local chat. It doesn't connect to other people or generate replies."))))
    }
}

fn icon(name: LucideIcon, size: f32, color: Rgba) -> gpui::Svg {
    lucide(name)
        .size(px(size))
        .flex_shrink_0()
        .text_color(color)
}

fn tag(label: impl Into<SharedString>, foreground: Rgba, background: Rgba) -> gpui::Div {
    div()
        .px(px(6.))
        .py(px(2.))
        .rounded(px(5.))
        .bg(background)
        .text_color(foreground)
        .text_size(px(11.))
        .font_weight(FontWeight::NORMAL)
        .child(label.into())
}

pub(super) fn workspace_theme(mode: ThemeMode) -> UiTheme {
    let mut theme = if mode == ThemeMode::Dark {
        UiTheme::neutral_dark()
    } else {
        UiTheme::neutral_light()
    };
    let c = &mut theme.colors;
    c.background = rgb(if mode == ThemeMode::Dark {
        0x000000
    } else {
        0xffffff
    });
    c.foreground = rgb(if mode == ThemeMode::Dark {
        0xffffff
    } else {
        0x000000
    });
    c.primary = c.foreground;
    c.primary_foreground = c.background;
    c.ring = c.foreground;
    c.destructive = c.foreground;
    c.chart_2 = c.foreground;
    c.sidebar_foreground = c.muted_foreground;
    c.sidebar_primary = c.primary;
    c.sidebar_primary_foreground = c.primary_foreground;
    c.sidebar_ring = c.ring;
    theme
}

impl Render for Workspace {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let c = theme.colors;
        let project_name = self
            .data
            .projects
            .iter()
            .find(|p| p.id == self.project)
            .map(|p| p.name.clone())
            .unwrap_or_default();
        let count = self
            .data
            .tasks
            .iter()
            .filter(|t| t.project == self.project)
            .count();
        let done = self
            .data
            .tasks
            .iter()
            .filter(|t| t.project == self.project && t.done)
            .count();
        let dark = theme.mode == ThemeMode::Dark;
        let mut sidebar = Sidebar::new("workspace.navigation", "Workspace projects")
            .content_padding(px(12.))
            .header(
                div()
                    .pt(px(42.))
                    .px(px(8.))
                    .pb(px(18.))
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    // Two open panel outlines, with no enclosing app-icon tile.
                    .child(
                        div()
                            .relative()
                            .size(px(34.))
                            .flex_shrink_0()
                            .child(
                                div()
                                    .absolute()
                                    .top(px(3.))
                                    .right_0()
                                    .w(px(27.))
                                    .h(px(25.))
                                    .rounded(px(4.))
                                    .border_4()
                                    .border_color(rgb(0xf0783e)),
                            )
                            .child(
                                div()
                                    .absolute()
                                    .bottom_0()
                                    .left_0()
                                    .w(px(27.))
                                    .h(px(25.))
                                    .rounded(px(4.))
                                    .border_3()
                                    .border_color(c.foreground)
                                    .bg(c.sidebar),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(3.))
                            .child(
                                div()
                                    .text_size(px(16.))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(c.sidebar_accent_foreground)
                                    .child("gpuicn"),
                            )
                            .child(
                                div()
                                    .text_size(px(11.))
                                    .text_color(c.sidebar_foreground)
                                    .child("Your everyday workspace"),
                            ),
                    ),
            )
            .child(
                div()
                    .px(px(8.))
                    .pb(px(22.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(div().size(px(6.)).rounded_full().bg(c.foreground))
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(c.sidebar_foreground)
                            .child("Local workspace"),
                    ),
            )
            .child(sidebar_group_label("PROJECTS", cx));
        for project in &self.data.projects {
            let id = project.id;
            let open = self
                .data
                .tasks
                .iter()
                .filter(|t| t.project == id && !t.done)
                .count();
            sidebar = sidebar.child(
                SidebarItem::new(("workspace.project", id), project.name.clone())
                    .h(px(40.))
                    .icon(icon(
                        LucideIcon::Layers,
                        16.,
                        if self.project == id {
                            c.primary
                        } else {
                            c.sidebar_foreground
                        },
                    ))
                    .trailing(
                        div()
                            .text_size(px(11.))
                            .text_color(c.sidebar_foreground)
                            .child(open.to_string()),
                    )
                    .selected(self.project == id)
                    .on_activate(cx.listener(move |this, _, _, cx| {
                        this.navigate(Destination::Project(id), cx)
                    })),
            );
        }
        sidebar = sidebar.child(
            div().pt(px(12.)).child(
                Button::new("workspace.show-project-composer")
                    .aria_label("New project")
                    .variant(ButtonVariant::Ghost)
                    .w_full()
                    .justify_start()
                    .text_color(c.sidebar_foreground)
                    .child(icon(LucideIcon::Plus, 15., c.sidebar_foreground))
                    .label("New project")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.adding_project = !this.adding_project;
                        cx.notify();
                    })),
            ),
        );
        if self.adding_project {
            sidebar = sidebar.child(
                div()
                    .pt(px(8.))
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(self.input(
                        "workspace.project-name",
                        "New project name",
                        &self.project_draft,
                        Draft::Project,
                        cx,
                    ))
                    .child(
                        Button::new("workspace.add-project")
                            .aria_label("Create project")
                            .label("Create project")
                            .disabled(self.project_draft.trim().is_empty())
                            .on_click(cx.listener(|this, _, _, cx| this.create_project(cx))),
                    ),
            );
        }
        sidebar = sidebar.footer(div().px(px(8.)).pb(px(12.)).flex().flex_col().gap(px(18.))
            .when(!self.data.intro_dismissed, |footer| footer.child(
                div().p(px(14.)).rounded(px(10.)).border_1().border_color(c.sidebar_border).flex().flex_col().gap(px(8.))
                    .child(div().flex().items_center().justify_between().gap(px(4.))
                        .child(div().text_size(px(12.)).font_weight(FontWeight::MEDIUM).text_color(c.sidebar_accent_foreground).child("A space to make things."))
                        .child(Button::new("workspace.dismiss-intro").aria_label("Dismiss introduction")
                            .variant(ButtonVariant::Ghost).size(ButtonSize::IconXs).flex_shrink_0()
                            .child(icon(LucideIcon::X, 14., c.sidebar_foreground))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.data.intro_dismissed = true;
                                this.persist("Introduction dismissed", cx);
                            }))))
                    .child(div().text_size(px(11.)).line_height(px(18.)).text_color(c.sidebar_foreground).child("Your projects, tasks, and ideas.\nA native app built with gpuicn."))))
            .child(Button::new("workspace.gallery").aria_label("Open component gallery").variant(ButtonVariant::Ghost).w_full().justify_start().label("Component gallery").on_click(|_, _, cx| super::gallery::open(cx)))
            .child(Button::new("workspace.theme").aria_label("Toggle workspace theme").variant(ButtonVariant::Ghost).w_full().justify_start().text_color(c.sidebar_foreground)
                .child(icon(if dark { LucideIcon::Sun } else { LucideIcon::Moon }, 15., c.sidebar_foreground))
                .child(if dark { "Light appearance" } else { "Dark appearance" })
                .on_click(cx.listener(move |_, _, _, cx| { UiTheme::set(cx, workspace_theme(if dark { ThemeMode::Light } else { ThemeMode::Dark })); })))
            .child(div().flex().items_center().gap(px(10.))
                .child(Avatar::new("workspace.profile").size(AvatarSize::Sm).child("Y"))
                .child(div().text_size(px(12.)).text_color(c.sidebar_accent_foreground).child("Your workspace"))
                .child(icon(LucideIcon::LockKeyhole, 12., c.sidebar_foreground))));
        let view = cx.entity().downgrade();
        let content = div()
            .size_full()
            .min_w_0()
            .min_h_0()
            .bg(c.background)
            .flex()
            .flex_col()
            .child(
                div()
                    .id("workspace.titlebar")
                    .window_control_area(gpui::WindowControlArea::Drag)
                    .h(px(52.))
                    .flex_shrink_0()
                    .px(px(28.))
                    .when(!self.sidebar.open, |el| el.pl(px(100.)))
                    .border_b_1()
                    .border_color(c.border)
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .text_size(px(12.))
                    .child(
                        sidebar_trigger(
                            "workspace.sidebar-toggle",
                            cx.listener(|this, _, _, cx| {
                                this.sidebar.toggle(false);
                                cx.notify();
                            }),
                            cx,
                        )
                        .aria_label(if self.sidebar.open {
                            "Hide sidebar"
                        } else {
                            "Show sidebar"
                        }),
                    )
                    .child(div().text_color(c.muted_foreground).child("Projects"))
                    .child(icon(LucideIcon::ChevronRight, 12., c.muted_foreground))
                    .child(project_name.clone())
                    .child(div().flex_1())
                    .child(icon(LucideIcon::HardDrive, 13., c.muted_foreground))
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(c.muted_foreground)
                            .child("On your Mac"),
                    ),
            )
            .child(
                div()
                    .px(px(28.))
                    .pt(px(28.))
                    .pb(px(24.))
                    .flex()
                    .items_center()
                    .gap(px(18.))
                    .child(
                        div()
                            .size(px(54.))
                            .flex_shrink_0()
                            .rounded(px(15.))
                            .bg(c.accent)
                            .border_1()
                            .border_color(c.primary.opacity(0.16))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(icon(LucideIcon::Layers, 26., c.primary)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .child(
                                div()
                                    .text_size(px(30.))
                                    .font_weight(FontWeight::MEDIUM)
                                    .truncate()
                                    .child(project_name),
                            )
                            .child(
                                div()
                                    .text_size(px(12.))
                                    .text_color(c.muted_foreground)
                                    .flex()
                                    .items_center()
                                    .gap(px(12.))
                                    .child(format!("{count} tasks  ·  {done} complete"))
                                    .child(
                                        div().w(px(96.)).child(
                                            Progress::new("workspace.progress")
                                                .label("Project completion")
                                                .value(if count == 0 {
                                                    0.
                                                } else {
                                                    done as f64 / count as f64 * 100.
                                                }),
                                        ),
                                    ),
                            ),
                    )
                    .child(
                        Button::new("workspace.new-task-header")
                            .aria_label("Create a new task")
                            .h(px(36.))
                            .child(icon(LucideIcon::Plus, 16., c.primary_foreground))
                            .label("New task")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.mode = "tasks";
                                this.adding_task = true;
                                this.task_focus.focus(window, cx);
                                cx.notify();
                            })),
                    ),
            )
            .child(
                tabs(cx)
                    .id("workspace.tabs")
                    .value(Some(self.mode))
                    .gap(px(0.))
                    .flex_1()
                    .min_h_0()
                    .on_value_change(move |mode, _, cx| {
                        if let Some(mode) = mode {
                            let _ = view.update(cx, |this, cx| {
                                this.mode = mode;
                                if *mode == "chat" {
                                    this.chat_scroll.scroll_to_bottom();
                                }
                                cx.notify();
                            });
                        }
                    })
                    .child(
                        tabs_list_with_variant(TabsVariant::Line, cx)
                            .mx(px(28.))
                            .child(
                                tabs_trigger(TabsVariant::Line, cx)
                                    .id("workspace.tasks-tab")
                                    .aria_label("Tasks")
                                    .value("tasks")
                                    .child(icon(LucideIcon::ListTodo, 15., c.muted_foreground))
                                    .child("Tasks"),
                            )
                            .child(
                                tabs_trigger(TabsVariant::Line, cx)
                                    .id("workspace.chat-tab")
                                    .aria_label("Local chat")
                                    .value("chat")
                                    .child(icon(
                                        LucideIcon::MessagesSquare,
                                        15.,
                                        c.muted_foreground,
                                    ))
                                    .child("Local chat"),
                            ),
                    )
                    .child(
                        tabs_content(cx)
                            .value("tasks")
                            .flex_1()
                            .min_h_0()
                            .child(self.tasks(cx)),
                    )
                    .child(
                        tabs_content(cx)
                            .value("chat")
                            .flex_1()
                            .min_h_0()
                            .child(self.chat(cx)),
                    ),
            )
            .child(
                div()
                    .px(px(24.))
                    .pb(px(12.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(icon(
                        if self.error.is_some() {
                            LucideIcon::CircleAlert
                        } else {
                            LucideIcon::Check
                        },
                        12.,
                        if self.error.is_some() {
                            c.destructive
                        } else {
                            c.muted_foreground
                        },
                    ))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_size(px(11.))
                            .text_color(if self.error.is_some() {
                                c.destructive
                            } else {
                                c.muted_foreground
                            })
                            .child(self.error.clone().unwrap_or_else(|| {
                                if self.details_changed() {
                                    "Unsaved task edits".into()
                                } else {
                                    self.notice.clone()
                                }
                            })),
                    )
                    .when(self.unsaved, |el| {
                        el.child(
                            Button::new("workspace.retry-save")
                                .variant(ButtonVariant::Outline)
                                .size(ButtonSize::Sm)
                                .label("Retry save")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.persist("Saved on this Mac", cx)
                                })),
                        )
                    })
                    .when(self.last_deleted.is_some(), |el| {
                        el.child(
                            Button::new("workspace.undo")
                                .aria_label("Undo task deletion")
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::Sm)
                                .label("Undo deletion")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    if let Some(task) = this.last_deleted.take() {
                                        let id = task.id;
                                        this.data.tasks.push(task);
                                        this.navigate(Destination::Task(id), cx);
                                        this.persist("Task restored", cx);
                                    }
                                })),
                        )
                    }),
            );
        let main = SidebarLayout::new(
            "workspace.layout",
            self.sidebar,
            false,
            sidebar,
            content,
            cx.listener(|this, state: &SidebarState, _, cx| {
                this.sidebar = *state;
                cx.notify();
            }),
        )
        .width(px(224.))
        .text_color(c.foreground);
        let modal = cx.entity().downgrade();
        let save_title = if self.unsaved {
            "Save workspace changes?"
        } else {
            "Save task changes?"
        };
        let save_description = if self.unsaved {
            "Some changes could not be saved. Retry saving before you quit, or discard changes that are only in memory."
        } else {
            "You have edits to this task. Save them before moving on, or discard them."
        };
        dialog_root("workspace.unsaved")
            .open(self.pending.is_some())
            .on_open_change(move |open, _, _, cx| {
                if !open {
                    let _ = modal.update(cx, |this, cx| {
                        this.pending = None;
                        cx.notify();
                    });
                }
            })
            .child_any(main)
            .child(
                dialog_portal().child(dialog_backdrop(cx)).child(
                    dialog_viewport(cx).child(
                        dialog_popup("workspace.unsaved.popup", save_title, cx)
                            .child(dialog_title("workspace.unsaved.title", cx).child(save_title))
                            .child(
                                dialog_description("workspace.unsaved.description", cx)
                                    .child(save_description),
                            )
                            .child_any(
                                div()
                                    .flex()
                                    .justify_end()
                                    .gap(px(8.))
                                    .child(
                                        Button::new("workspace.unsaved.cancel")
                                            .aria_label("Keep editing task")
                                            .variant(ButtonVariant::Outline)
                                            .label("Keep editing")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.pending = None;
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Button::new("workspace.unsaved.discard")
                                            .aria_label("Discard task edits")
                                            .variant(ButtonVariant::Ghost)
                                            .label("Discard")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.select_task(this.selected);
                                                if let Some(next) = this.pending.take() {
                                                    this.continue_to(next, cx);
                                                }
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Button::new("workspace.unsaved.save")
                                            .aria_label("Save task and continue")
                                            .label("Save and continue")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                if this.save_details(cx)
                                                    && let Some(next) = this.pending.take()
                                                {
                                                    this.continue_to(next, cx);
                                                }
                                                cx.notify();
                                            })),
                                    ),
                            ),
                    ),
                ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Modifiers, TestAppContext, VisualTestContext};

    #[gpui::test]
    fn cmd_q_from_focused_dirty_input_opens_save_confirmation(cx: &mut TestAppContext) {
        let temp = tempfile::tempdir().unwrap();
        let (store, data, error) = Store::open(temp.path().join("workspace.json"));
        cx.update(|cx| {
            gpuicn::init(cx);
            UiTheme::set(cx, workspace_theme(ThemeMode::Light));
            cx.bind_keys([gpui::KeyBinding::new("cmd-q", Quit, None)]);
        });
        let window = cx.add_window(|_, cx| {
            let mut view = Workspace::new(store, data, error, cx);
            view.editing_title = true;
            view
        });
        cx.update(move |cx| {
            cx.on_action(move |_: &Quit, cx| request_quit(window, cx));
        });
        let mut visual = VisualTestContext::from_window(window.into(), cx);
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_keystrokes("cmd-a");
        visual.simulate_input("Unsaved native title");
        visual.simulate_keystrokes("cmd-q");
        cx.run_until_parked();
        assert!(
            cx.read_window(&window, |view, cx| {
                let view = view.read(cx);
                view.details_changed() && matches!(view.pending, Some(Destination::Quit))
            })
            .unwrap()
        );
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let mut tab_stops = Vec::new();
        for _ in 0..4 {
            visual.simulate_keystrokes("tab");
            tab_stops.push(visual.update(|window, cx| {
                window.draw(cx).clear(cx);
                window.focused(cx)
            }));
        }
        assert_eq!(
            tab_stops[0], tab_stops[3],
            "forward wrap skips the popup itself"
        );
        visual.simulate_keystrokes("shift-tab");
        assert_eq!(
            visual.update(|window, cx| window.focused(cx)),
            tab_stops[2],
            "reverse wrap goes to the last button"
        );
        visual.simulate_keystrokes("tab");
        visual.simulate_keystrokes("enter");
        cx.run_until_parked();
        assert!(
            cx.read_window(&window, |view, cx| view.read(cx).pending.is_none())
                .unwrap(),
            "Tab must wrap over the three dialog buttons so Enter keeps editing"
        );
    }

    #[gpui::test]
    fn failed_save_stays_unsaved_and_can_be_retried(cx: &mut TestAppContext) {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("workspace.json");
        let (store, data, error) = Store::open(path.clone());
        // Force a real filesystem error, then remove it before retrying.
        std::fs::create_dir(&path).unwrap();
        let view = cx.new(|cx| Workspace::new(store, data, error, cx));
        view.update(cx, |this, cx| {
            this.title_draft = "Saved after recovery".into();
            assert!(!this.save_details(cx));
            assert!(this.details_changed());
            assert_eq!(this.data.tasks[0].title, "Make a great first impression");
            this.complete(this.selected.unwrap(), true, cx);
            assert!(this.unsaved);
            this.navigate(Destination::Quit, cx);
            assert!(matches!(this.pending, Some(Destination::Quit)));
        });
        std::fs::remove_dir(&path).unwrap();
        view.update(cx, |this, cx| {
            assert!(this.save_details(cx));
            assert!(!this.unsaved);
            assert!(this.error.is_none());
        });
        let (_, saved, error) = Store::open(path);
        assert!(error.is_none());
        assert_eq!(saved.tasks[0].title, "Saved after recovery");
    }

    #[test]
    fn typing_navigation_and_chat_preserve_real_saved_content() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("workspace.json");
        let (store, data, error) = Store::open(path.clone());
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            gpuicn::init(cx);
            UiTheme::set(cx, workspace_theme(ThemeMode::Light));
        });
        let window = cx.add_window(|_, cx| Workspace::new(store, data, error, cx));
        let mut visual = VisualTestContext::from_window(window.into(), &cx);
        visual.simulate_resize(size(px(1240.), px(800.)));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        // The header toggle must work as a control, including in the drag area.
        visual.simulate_click(gpui::point(px(266.), px(26.)), Modifiers::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(
            !cx.read_window(&window, |view, cx| view.read(cx).sidebar.open)
                .unwrap()
        );
        visual.simulate_keystrokes("enter");
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(
            cx.read_window(&window, |view, cx| view.read(cx).sidebar.open)
                .unwrap()
        );
        for open in [false, true] {
            visual.simulate_keystrokes("cmd-b");
            visual.update(|window, cx| window.draw(cx).clear(cx));
            assert_eq!(
                cx.read_window(&window, |view, cx| view.read(cx).sidebar.open)
                    .unwrap(),
                open
            );
        }
        window
            .update(&mut cx, |this, _, cx| {
                this.adding_task = true;
                cx.notify();
            })
            .unwrap();
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let input = visual.debug_bounds("Add a task…").unwrap();
        visual.simulate_click(input.center(), Modifiers::default());
        // The shell shortcut must also work while editing, without typing a b.
        for open in [false, true] {
            visual.simulate_keystrokes("cmd-b");
            visual.update(|window, cx| window.draw(cx).clear(cx));
            assert_eq!(
                cx.read_window(&window, |view, cx| view.read(cx).sidebar.open)
                    .unwrap(),
                open
            );
        }
        for character in "Review the native workspace".chars() {
            visual.simulate_input(&character.to_string());
            visual.update(|window, cx| window.draw(cx).clear(cx));
        }
        cx.run_until_parked();
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(
            cx.read_window(&window, |view, cx| view.read(cx).task_draft.clone())
                .unwrap(),
            "Review the native workspace"
        );
        // New task must return focus to an already-open composer without resetting it.
        let search = visual.debug_bounds("Search tasks").unwrap();
        visual.simulate_click(search.center(), Modifiers::default());
        visual.simulate_click(gpui::point(px(1150.), px(107.)), Modifiers::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_input("!");
        assert_eq!(
            cx.read_window(&window, |view, cx| view.read(cx).task_draft.clone())
                .unwrap(),
            "Review the native workspace!"
        );
        visual.simulate_keystrokes("backspace");
        visual.simulate_keystrokes("enter");
        cx.run_until_parked();
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let (_, saved, error) = Store::open(path.clone());
        assert!(error.is_none());
        assert_eq!(
            saved.tasks.last().unwrap().title,
            "Review the native workspace"
        );
        assert!(
            cx.read_window(&window, |view, cx| view.read(cx).task_draft.is_empty())
                .unwrap()
        );

        // Navigation must not silently discard an edited task.
        window
            .update(&mut cx, |this, _, cx| {
                this.editing_title = true;
                cx.notify();
            })
            .unwrap();
        visual.update(|window, cx| window.draw(cx).clear(cx));
        // Opening the title editor must focus it without a second click.
        visual.simulate_keystrokes("cmd-a");
        visual.simulate_input("Edited task");
        cx.run_until_parked();
        window
            .update(&mut cx, |this, _, cx| {
                this.navigate(Destination::Project(2), cx)
            })
            .unwrap();
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(
            cx.read_window(&window, |view, cx| view.read(cx).pending.is_some())
                .unwrap()
        );
        assert_eq!(
            cx.read_window(&window, |view, cx| view.read(cx).project)
                .unwrap(),
            1
        );
        window
            .update(&mut cx, |this, _, cx| {
                assert!(this.save_details(cx));
                let next = this.pending.take().unwrap();
                this.continue_to(next, cx);
                this.mode = "chat";
                cx.notify();
            })
            .unwrap();
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let input = visual.debug_bounds("Write a local message…").unwrap();
        visual.simulate_click(input.center(), Modifiers::default());
        for character in "Check the links before launch".chars() {
            visual.simulate_input(&character.to_string());
            visual.update(|window, cx| window.draw(cx).clear(cx));
        }
        cx.run_until_parked();
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_keystrokes("enter");
        cx.run_until_parked();
        visual.update(|window, cx| window.draw(cx).clear(cx));
        // Sending a message must leave the new composer ready to type.
        visual.simulate_input("A second message");
        cx.run_until_parked();
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_keystrokes("enter");
        cx.run_until_parked();
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let (_, saved, error) = Store::open(path);
        assert!(error.is_none());
        assert_eq!(saved.messages.last().unwrap().body, "A second message");
        assert_eq!(saved.tasks.last().unwrap().title, "Edited task");
        assert_eq!(
            saved.messages[saved.messages.len() - 2].body,
            "Check the links before launch"
        );
        assert_eq!(saved.messages.last().unwrap().project, 2);
        // Finish the upstream scrollbar fade timer after closing its window.
        visual.update(|window, _| window.remove_window());
        cx.executor()
            .advance_clock(std::time::Duration::from_secs(3));
        cx.run_until_parked();
    }
}
