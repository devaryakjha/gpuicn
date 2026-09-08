//! Local native review surface for the first desktop component batch.
//! Run: cargo run -p gpuicn --example desktop --features native-fixture,gpui_platform/runtime_shaders

use gpui::{
    App, AppContext as _, Bounds, Context, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, ScrollStrategy, StatefulInteractiveElement as _, Styled,
    UniformListScrollHandle, Window, WindowBounds, WindowOptions, canvas, div,
    prelude::FluentBuilder as _, px, size, uniform_list,
};
use gpui_icons::{LucideAssetSource, LucideIcon, lucide};
use gpuicn::{
    Button, ButtonSize, ButtonVariant, ThemeMode, UiTheme,
    dialog::*,
    input::Input,
    resizable::{PaneLimits, Resizable},
    sidebar::{Sidebar, SidebarItem, sidebar_group_label},
};
use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    rc::Rc,
    time::Instant,
};

fn main() {
    gpui_platform::application()
        .with_assets(LucideAssetSource)
        .run(|cx: &mut App| {
            gpuicn::init(cx);
            cx.text_system()
                .add_fonts(vec![
                    Cow::Borrowed(include_bytes!(
                        "../../../site/assets/fonts/Geist-Regular.ttf"
                    )),
                    Cow::Borrowed(include_bytes!(
                        "../../../site/assets/fonts/Geist-Medium.ttf"
                    )),
                    Cow::Borrowed(include_bytes!(
                        "../../../site/assets/fonts/GeistMono-Regular.ttf"
                    )),
                ])
                .expect("load bundled Geist fonts");
            UiTheme::set(cx, UiTheme::neutral_light());
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                        None,
                        size(px(1080.), px(740.)),
                        cx,
                    ))),
                    ..Default::default()
                },
                |window, cx| {
                    window.set_window_title("gpuicn — desktop component review");
                    let rows = Rc::new(Cell::new(0));
                    let content = cx.new(|_| Content {
                        rows: rows.clone(),
                        scroll: UniformListScrollHandle::new(),
                    });
                    cx.new(|_| Desktop {
                        collapsed: false,
                        personalized: false,
                        sidebar_width: px(240.),
                        content_height: px(400.),
                        selected: "Changes",
                        resize_count: 0,
                        content,
                        rows,
                        measured: 0,
                        benchmark: std::env::args()
                            .any(|arg| arg == "--benchmark")
                            .then(|| Rc::new(RefCell::new(Benchmark::default()))),
                    })
                },
            )
            .expect("open native review window");
            cx.activate(true);
        });
}
struct Desktop {
    collapsed: bool,
    personalized: bool,
    sidebar_width: gpui::Pixels,
    content_height: gpui::Pixels,
    selected: &'static str,
    resize_count: usize,
    content: Entity<Content>,
    rows: Rc<Cell<usize>>,
    measured: usize,
    benchmark: Option<Rc<RefCell<Benchmark>>>,
}
impl Render for Desktop {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let started = Instant::now();
        if let Some(benchmark) = &self.benchmark {
            let frame = benchmark.borrow().cpu_ms.len();
            if frame < 660 {
                let entity = cx.entity();
                window.on_next_frame(move |_, cx| {
                    entity.update(cx, |this, cx| {
                        this.sidebar_width = px(180. + (frame % 180) as f32);
                        this.collapsed = (frame / 120) % 2 == 1;
                        this.content_height = px(200. + (frame % 240) as f32);
                        this.content.update(cx, |content, cx| {
                            content
                                .scroll
                                .scroll_to_item((frame * 137) % 99_950, ScrollStrategy::Top);
                            cx.notify();
                        });
                        cx.notify();
                    })
                });
            } else {
                benchmark.borrow().save(self.rows.get());
                cx.quit();
            }
        }
        let theme = UiTheme::read(cx).clone();
        let collapsed = self.collapsed;
        let icon = |name| {
            lucide(name)
                .size(theme.space(4.))
                .text_color(theme.colors.foreground)
        };
        let collapse = Button::new("lab.collapse")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Icon)
            .aria_label(if collapsed {
                "Expand sidebar"
            } else {
                "Collapse sidebar"
            })
            .on_click(cx.listener(|this, _, _, cx| {
                this.collapsed = !this.collapsed;
                cx.notify();
            }))
            .child(icon(if collapsed {
                LucideIcon::PanelLeftOpen
            } else {
                LucideIcon::PanelLeftClose
            }));
        let mut sidebar = Sidebar::new("lab.sidebar", "Workspace navigation")
            .header(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(collapse)
                    .when(!collapsed, |el| {
                        el.child(
                            div()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .child("gpuicn"),
                        )
                    }),
            )
            .footer(
                SidebarItem::new("lab.settings", "Settings")
                    .collapsed(collapsed)
                    .selected(self.selected == "Settings")
                    .icon(icon(LucideIcon::Settings))
                    .on_activate(cx.listener(|this, _, _, cx| {
                        this.selected = "Settings";
                        cx.notify();
                    })),
            );
        if !collapsed {
            sidebar = sidebar.child(sidebar_group_label("Workspace", cx));
        }
        for (id, label, glyph, disabled) in [
            ("changes", "Changes", LucideIcon::FileDiff, false),
            ("history", "History", LucideIcon::Clock, false),
            ("branches", "Branches", LucideIcon::GitBranch, false),
            (
                "pull-requests",
                "Pull requests",
                LucideIcon::GitPullRequest,
                false,
            ),
            (
                "long-name",
                "A repository with a deliberately long name",
                LucideIcon::FolderGit2,
                false,
            ),
            ("releases", "Releases", LucideIcon::Package, true),
        ] {
            sidebar = sidebar.child(
                SidebarItem::new(format!("lab.nav.{id}"), label)
                    .selected(self.selected == label)
                    .disabled(disabled)
                    .collapsed(collapsed)
                    .icon(icon(glyph))
                    .on_activate(cx.listener(move |this, _, _, cx| {
                        this.selected = label;
                        cx.notify();
                    })),
            );
        }
        let status = div().id("lab.status").size_full().overflow_y_scroll().p(px(20.))
            .flex().flex_col().gap(px(12.))
            .child(div().font_weight(gpui::FontWeight::SEMIBOLD).child("Try the native interactions"))
            .child(format!("Destination: {}", self.selected))
            .child(format!("Preferred sidebar: {:.0}px · top pane: {:.0}px · resize events: {}",
                f32::from(self.sidebar_width), f32::from(self.content_height), self.resize_count))
            .child("Tab to a separator. Use arrows to resize; Shift makes larger steps. Home/End reach the limits.")
            .child("Narrow the window: panes retain their minimum size and the group scrolls. Collapse restores your preferred sidebar width.")
            .child(Button::new("lab.measure").aria_label("Inspect rendered range").variant(ButtonVariant::Outline)
                .on_click(cx.listener(|this, _, _, cx| { this.measured = this.rows.get(); cx.notify(); }))
                .child("Inspect rendered range"))
            .child(format!("Largest row range requested: {} / 100,000 (render count, not a frame-time benchmark)", self.measured));
        let split = Resizable::new(
            "lab.content-split",
            "Resize list and details",
            self.content_height,
            self.content.clone(),
            status,
            cx.listener(|this, value: &gpui::Pixels, _, cx| {
                this.content_height = *value;
                this.resize_count += 1;
                cx.notify();
            }),
        )
        .vertical()
        .first_limits(PaneLimits::new(px(140.), px(900.)))
        .second_limits(PaneLimits::new(px(180.), px(900.)));
        let workspace = div()
            .size_full()
            .flex()
            .flex_col()
            .min_w_0()
            .min_h_0()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(12.))
                    .px(px(20.))
                    .h(px(56.))
                    .flex_shrink_0()
                    .border_b_1()
                    .border_color(theme.colors.border)
                    .child(
                        div()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Component review"),
                    )
                    .child(
                        Button::new("lab.theme")
                            .aria_label("Switch theme")
                            .variant(ButtonVariant::Outline)
                            .on_click(cx.listener(|this, _, _, cx| {
                                let next = if UiTheme::read(cx).mode == ThemeMode::Light {
                                    ThemeMode::Dark
                                } else {
                                    ThemeMode::Light
                                };
                                set_review_theme(next, this.personalized, cx);
                                this.content.update(cx, |_, cx| cx.notify());
                                cx.notify();
                            }))
                            .child("Switch theme"),
                    )
                    .child(
                        Button::new("lab.personalize")
                            .aria_label("Toggle personalized theme")
                            .variant(ButtonVariant::Outline)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.personalized = !this.personalized;
                                let mode = UiTheme::read(cx).mode;
                                set_review_theme(mode, this.personalized, cx);
                            }))
                            .child("Personalize"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(theme.space(2.))
                            .child(
                                gpuicn::switch::Switch::new("lab.motion")
                                    .aria_label("Reduce motion")
                                    .checked(theme.motion.reduced)
                                    .on_checked_change(|checked, _, _, cx| {
                                        let mut theme = UiTheme::read(cx).clone();
                                        theme.motion.reduced = checked;
                                        UiTheme::set(cx, theme);
                                    }),
                            )
                            .child("Reduce motion"),
                    ),
            )
            .child(div().flex_1().min_h_0().min_w_0().child(split));
        let collapse_progress = gpuicn::theme::transition_value(
            "lab.sidebar-collapse",
            if collapsed { 1. } else { 0. },
            theme.motion.normal,
            window,
            cx,
        );
        let rail = theme.space(12.);
        let width = self.sidebar_width * (1. - collapse_progress) + rail * collapse_progress;
        let limits = if collapse_progress > 0. {
            PaneLimits::new(width, width)
        } else {
            PaneLimits::new(px(180.), px(360.))
        };
        div()
            .relative()
            .size_full()
            .bg(theme.colors.background)
            .text_color(theme.colors.foreground)
            .font_family(theme.fonts.body)
            .text_size(px(14.))
            .on_key_down(|event, window, cx| {
                if event.keystroke.key == "tab" {
                    if event.keystroke.modifiers.shift {
                        window.focus_prev(cx);
                    } else {
                        window.focus_next(cx);
                    }
                    cx.stop_propagation();
                }
            })
            .child(
                Resizable::new(
                    "lab.sidebar-split",
                    "Resize navigation",
                    width,
                    sidebar,
                    workspace,
                    cx.listener(|this, value: &gpui::Pixels, _, cx| {
                        if !this.collapsed && this.sidebar_width != *value {
                            this.sidebar_width = *value;
                            this.resize_count += 1;
                            cx.notify();
                        }
                    }),
                )
                .first_limits(limits)
                .second_limits(PaneLimits::new(px(460.), px(4000.))),
            )
            .when_some(self.benchmark.clone(), |el, benchmark| {
                el.child(
                    canvas(
                        |_, _, _| (),
                        move |_, _, _, _| {
                            let now = Instant::now();
                            let mut sample = benchmark.borrow_mut();
                            sample.cpu_ms.push(started.elapsed().as_secs_f64() * 1000.);
                            if let Some(previous) = sample.previous.replace(now) {
                                sample
                                    .interval_ms
                                    .push(now.duration_since(previous).as_secs_f64() * 1000.);
                            }
                        },
                    )
                    .absolute()
                    .size_full(),
                )
            })
    }
}
struct Content {
    rows: Rc<Cell<usize>>,
    scroll: UniformListScrollHandle,
}
impl Render for Content {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let count = self.rows.clone();
        div().size_full().flex().flex_col().min_h_0().min_w_0()
            .child(div().flex().items_center().gap(px(12.)).p(px(16.)).flex_shrink_0()
                .child(div().w(px(300.)).child(Input::new("lab.note").aria_label("Persistent test text").placeholder("Type here; resizing should preserve this text")))
                .child(dialog_root("lab.dialog")
                    .child(dialog_trigger("lab.dialog.trigger", cx).aria_label("Open dialog").child("Open dialog"))
                    .child(dialog_portal().child(dialog_backdrop(cx)).child(dialog_viewport(cx).child(
                        dialog_popup("lab.dialog.popup", "Native focus check", cx)
                            .child_any(dialog_title("lab.dialog.title", cx).child("Native focus check"))
                            .child_any(Input::new("lab.dialog.input").aria_label("Dialog test input").placeholder("Tab through this dialog"))
                            .child_any(dialog_close("lab.dialog.close", cx).aria_label("Close dialog").child("Close")),
                    )))))
            .child(div().flex_1().min_h_0().overflow_hidden().child(
                uniform_list("lab.rows", 100_000, move |range, _, _| {
                    count.set(count.get().max(range.len()));
                    range.map(|index| div().h(px(28.)).px(px(16.)).flex().items_center()
                        .bg(if index % 2 == 0 { theme.colors.background } else { theme.colors.muted.opacity(0.35) })
                        .child(format!("Row {:06} — synthetic content for resizing and scroll checks", index + 1)))
                        .collect::<Vec<_>>()
                }).track_scroll(&self.scroll).size_full(),
            ))
    }
}

// Native baseline only: render-to-canvas-paint CPU time and paint intervals.
// These exclude GPU completion and input latency. Discard 60 warmup frames.
#[derive(Default)]
struct Benchmark {
    cpu_ms: Vec<f64>,
    interval_ms: Vec<f64>,
    previous: Option<Instant>,
}
impl Benchmark {
    fn save(&self, rows: usize) {
        let cpu = &self.cpu_ms[60..];
        let intervals = &self.interval_ms[60..];
        let percentile = |values: &[f64], fraction: f64| {
            let mut values = values.to_vec();
            values.sort_by(f64::total_cmp);
            values[((values.len() - 1) as f64 * fraction).round() as usize]
        };
        let report = format!(
            "{{\n  \"cpu_ms\": {:?},\n  \"paint_interval_ms\": {:?},\n  \"rendered_rows_max\": {},\n  \"cpu_p50_p95_p99_ms\": [{}, {}, {}],\n  \"interval_p50_p95_p99_ms\": [{}, {}, {}]\n}}\n",
            cpu,
            intervals,
            rows,
            percentile(cpu, 0.5),
            percentile(cpu, 0.95),
            percentile(cpu, 0.99),
            percentile(intervals, 0.5),
            percentile(intervals, 0.95),
            percentile(intervals, 0.99),
        );
        std::fs::write("/tmp/gpuicn-desktop-benchmark.json", report).expect("write benchmark");
    }
}

fn review_theme(mode: ThemeMode, personalized: bool) -> UiTheme {
    let mut theme = match mode {
        ThemeMode::Light => UiTheme::neutral_light(),
        ThemeMode::Dark => UiTheme::neutral_dark(),
    };
    if personalized {
        theme.radius = gpuicn::theme::UiRadius::new(px(4.));
        theme.spacing.unit = px(4.5);
        theme.colors.primary = gpui::rgb(0x2563eb);
        theme.colors.primary_foreground = gpui::rgb(0xffffff);
        theme.colors.ring = theme.colors.primary;
        theme.colors.sidebar_ring = theme.colors.primary;
        theme.colors.sidebar_accent = match mode {
            ThemeMode::Light => gpui::rgb(0xdbeafe),
            ThemeMode::Dark => gpui::rgb(0x172554),
        };
        theme.colors.sidebar_accent_foreground = theme.colors.foreground;
    }
    theme
}

fn set_review_theme(mode: ThemeMode, personalized: bool, cx: &mut App) {
    let motion = UiTheme::read(cx).motion;
    let mut theme = review_theme(mode, personalized);
    theme.motion = motion;
    UiTheme::set(cx, theme);
}
