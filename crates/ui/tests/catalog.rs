//! Every component draws in both themes without panicking, and the stateful
//! ones answer a click. The registry lint requires each component to be named
//! in a test; this file covers the ones whose behavior is Kit's or purely visual.

use std::{cell::RefCell, rc::Rc};

use gpui_icons::LucideIcon;
use gpui_kit::{
    AnyElement, App, AppContext as _, Axis, Context, InteractiveElement as _, IntoElement,
    Modifiers, ParentElement, Render, Styled, TestAppContext, VisualTestContext, Window, div, px,
    rgb,
};
use gpuicn::{
    Button, ButtonVariant, UiTheme,
    alert_dialog::{alert_dialog, alert_dialog_action, alert_dialog_cancel},
    attachment::{Attachment, AttachmentStatus, attachment_group},
    avatar::{Avatar, AvatarSize},
    carousel::{
        Carousel, CarouselContent, CarouselItem, CarouselNext, CarouselPagination,
        CarouselPaginationItem, CarouselPrevious, CarouselState,
    },
    checkbox::Checkbox,
    checkbox_group::{CheckboxGroup, CheckboxGroupItem},
    collapsible::{collapsible, collapsible_content, collapsible_trigger},
    context_menu::context_menu,
    dialog::{DialogHandle, dialog_description, dialog_footer, dialog_popup, dialog_title},
    empty::{Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle},
    fieldset::{FieldsetLegendVariant, fieldset, fieldset_legend},
    form::form,
    input::{Input, InputState, TextareaState},
    input_group::{InputGroup, InputGroupAddon, InputGroupButton, InputGroupText},
    markdown::{markdown, selection_root},
    menu::{MenuItem, MenuState},
    menubar::Menubar,
    message::{Message, MessageHighlight, MessageStatus, Reaction, day_divider},
    meter::Meter,
    navigation_menu::navigation_menu,
    number_field::NumberField,
    otp_field::{OtpField, OtpState},
    popover::{popover, popover_description, popover_popup, popover_title, popover_trigger},
    preview_card::{preview_card, preview_card_popup},
    scroll_area::ScrollArea,
    separator::Separator,
    skeleton::Skeleton,
    spinner::Spinner,
    switch::Switch,
    tag::{Tag, TagVariant},
    textarea::textarea,
    title_bar::TitleBar,
    toggle::{Toggle, ToggleSize, ToggleVariant},
    toggle_group::{ToggleGroup, ToggleGroupItem},
    toolbar::{Toolbar, ToolbarButton},
};

type Build = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;

struct Probe(Build);
impl Render for Probe {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .p(px(16.))
            .bg(UiTheme::read(cx).colors.background)
            .child((self.0)(window, cx))
    }
}

/// Opens a window around `build`, draws it in light and dark mode, and leaves
/// it in light mode for interaction.
fn mount<E: IntoElement>(
    cx: &mut TestAppContext,
    build: impl Fn(&mut Window, &mut App) -> E + 'static,
) -> &mut VisualTestContext {
    cx.update(|cx| {
        gpuicn::init(cx);
        UiTheme::set(cx, UiTheme::neutral_light());
    });
    let build: Build = Rc::new(move |window, cx| build(window, cx).into_any_element());
    let (_, visual) = cx.add_window_view(move |window, _| {
        window.activate_window();
        Probe(build)
    });
    draw(visual);
    visual.update(|_, cx| UiTheme::set(cx, UiTheme::neutral_dark()));
    draw(visual);
    visual.update(|_, cx| UiTheme::set(cx, UiTheme::neutral_light()));
    draw(visual);
    visual
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.run_until_parked();
}

fn click(cx: &mut VisualTestContext, selector: &'static str) {
    let bounds = cx
        .debug_bounds(selector)
        .unwrap_or_else(|| panic!("{selector} is not on screen"));
    cx.simulate_click(bounds.center(), Modifiers::default());
    draw(cx);
}

fn probe(selector: &'static str) -> gpui_kit::Div {
    div().debug_selector(move || selector.into())
}

#[gpui_kit::test]
fn toggles_report_their_next_state(cx: &mut TestAppContext) {
    let log = Rc::new(RefCell::new(Vec::<String>::new()));
    let sink = log.clone();
    let cx = mount(cx, move |_, _| {
        let (checkbox, switch, toggle) = (sink.clone(), sink.clone(), sink.clone());
        div()
            .flex()
            .flex_col()
            .items_start()
            .gap(px(8.))
            .child(
                probe("checkbox").child(Checkbox::new("agree").aria_label("Agree").on_change(
                    move |next, _, _, _| checkbox.borrow_mut().push(format!("checkbox {next}")),
                )),
            )
            .child(
                Checkbox::new("partial")
                    .indeterminate(true)
                    .aria_label("Some"),
            )
            .child(
                probe("switch").child(
                    Switch::new("airplane")
                        .aria_label("Airplane mode")
                        .on_change(move |next, _, _, _| {
                            switch.borrow_mut().push(format!("switch {next}"))
                        }),
                ),
            )
            .child(
                Switch::new("locked")
                    .checked(true)
                    .disabled(true)
                    .aria_label("Locked"),
            )
            .child(
                probe("toggle").child(
                    Toggle::new("bold")
                        .aria_label("Bold")
                        .variant(ToggleVariant::Outline)
                        .size(ToggleSize::Sm)
                        .on_change(move |next, _, _, _| {
                            toggle.borrow_mut().push(format!("toggle {next}"))
                        })
                        .child("B"),
                ),
            )
            .child(
                Toggle::new("pressed")
                    .pressed(true)
                    .aria_label("Italic")
                    .child("I"),
            )
    });
    click(cx, "checkbox");
    click(cx, "switch");
    click(cx, "toggle");
    assert_eq!(
        *log.borrow(),
        ["checkbox true", "switch true", "toggle true"]
    );
}

#[gpui_kit::test]
fn groups_report_their_selected_values(cx: &mut TestAppContext) {
    let log = Rc::new(RefCell::new(Vec::<Vec<String>>::new()));
    let sink = log.clone();
    let cx = mount(cx, move |_, _| {
        let (checks, toggles) = (sink.clone(), sink.clone());
        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(
                probe("checks").child(
                    CheckboxGroup::new("notify")
                        .aria_label("Notifications")
                        .value(["email"])
                        .item(CheckboxGroupItem::new("email", "email").label("Email"))
                        .item(CheckboxGroupItem::new("sms", "sms").label("SMS"))
                        .item(
                            CheckboxGroupItem::new("push", "push")
                                .label("Push")
                                .disabled(true),
                        )
                        .on_change(move |values, _, _| {
                            checks
                                .borrow_mut()
                                .push(values.iter().map(|v| v.to_string()).collect())
                        }),
                ),
            )
            .child(
                ToggleGroup::new("align")
                    .aria_label("Alignment")
                    .joined(true)
                    .value(["left"])
                    .item(
                        ToggleGroupItem::new("left", "left")
                            .aria_label("Left")
                            .child("L"),
                    )
                    .item(
                        ToggleGroupItem::new("center", "center")
                            .aria_label("Center")
                            .child("C"),
                    )
                    .item(
                        ToggleGroupItem::new("right", "right")
                            .aria_label("Right")
                            .disabled(true)
                            .child("R"),
                    )
                    .on_change(move |values, _, _| {
                        toggles
                            .borrow_mut()
                            .push(values.iter().map(|v| v.to_string()).collect())
                    }),
            )
            .child(
                ToggleGroup::new("styles")
                    .aria_label("Styles")
                    .multiple(true)
                    .value(["bold", "italic"])
                    .item(
                        ToggleGroupItem::new("bold", "bold")
                            .aria_label("Bold")
                            .child("B"),
                    )
                    .item(
                        ToggleGroupItem::new("italic", "italic")
                            .aria_label("Italic")
                            .child("I"),
                    ),
            )
    });
    // The first item sits at the group's top-left corner; Tab then reaches the second.
    let group = cx.debug_bounds("checks").expect("group drawn");
    cx.simulate_click(
        group.origin + gpui_kit::point(px(10.), px(10.)),
        Modifiers::default(),
    );
    draw(cx);
    assert_eq!(
        log.borrow().last(),
        Some(&Vec::<String>::new()),
        "email unchecked"
    );
    // Three rows: the middle of the group is the second item.
    cx.simulate_click(
        group.origin + gpui_kit::point(px(10.), group.size.height / 2.),
        Modifiers::default(),
    );
    draw(cx);
    assert_eq!(
        log.borrow().last(),
        Some(&vec!["email".to_string(), "sms".to_string()]),
        "the group reports the next value set from its rendered value"
    );
}

#[gpui_kit::test]
fn collapsible_reveals_its_content_on_open(cx: &mut TestAppContext) {
    let open = Rc::new(RefCell::new(false));
    let state = open.clone();
    let cx = mount(cx, move |window, cx| {
        let is_open = *state.borrow();
        let flip = state.clone();
        collapsible("projects", is_open, window, cx)
            .w(px(320.))
            .child(
                probe("trigger").child(
                    collapsible_trigger("disclosure", is_open, cx)
                        .accessibility_label("Toggle projects")
                        .child("Toggle")
                        .on_click(move |_, window, _| {
                            let next = !*flip.borrow();
                            *flip.borrow_mut() = next;
                            window.refresh();
                        }),
                ),
            )
            .child(div().child("Design system"))
            .content(
                collapsible_content(cx)
                    .child(probe("hidden").child("Documentation"))
                    .child(div().child("Website")),
            )
    });
    assert!(
        cx.debug_bounds("hidden").is_none(),
        "closed content is not laid out"
    );
    click(cx, "trigger");
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(400));
    draw(cx);
    assert!(*open.borrow());
    assert!(
        cx.debug_bounds("hidden").is_some(),
        "open content is laid out"
    );
}

#[gpui_kit::test]
fn alert_dialog_opens_from_its_trigger_and_cancels(cx: &mut TestAppContext) {
    let handle = DialogHandle::new(false);
    let shared = handle.clone();
    let cx = mount(cx, move |window, cx| {
        let handle = shared.clone();
        let popup = dialog_popup("alert.popup", "Delete account?", cx)
            .child(dialog_title("alert.title", cx).child("Delete account?"))
            .child(dialog_description("alert.description", cx).child("This cannot be undone."))
            .child(
                dialog_footer(cx)
                    .child(
                        probe("cancel").child(alert_dialog_cancel("alert.cancel").label("Cancel")),
                    )
                    .child(alert_dialog_action("alert.confirm").label("Continue")),
            );
        let open = handle.clone();
        div()
            .child(
                probe("open").child(
                    Button::new("alert.trigger")
                        .label("Delete account")
                        .on_click(move |_, window, cx| open.open(window, cx)),
                ),
            )
            .child(alert_dialog("alert", &handle, popup, window, cx))
    });
    assert!(!handle.is_open());
    click(cx, "open");
    assert!(handle.is_open(), "the trigger opens the alert");
    draw(cx);
    click(cx, "cancel");
    assert!(!handle.is_open(), "cancel closes it");
}

#[gpui_kit::test]
fn carousel_advances_through_its_slides(cx: &mut TestAppContext) {
    let state = cx.update(|cx| cx.new(|_| CarouselState::new(3).with_looping(true)));
    let shared = state.clone();
    let cx = mount(cx, move |_, _| {
        let state = shared.clone();
        probe("carousel").child(
            Carousel::new("tour", &state)
                .aria_label("Featured")
                .w(px(300.))
                .child(
                    CarouselContent::new(&state)
                        .h(px(120.))
                        .children((0..3).map(|index| {
                            CarouselItem::new(("slide", index), index, &state)
                                .child(div().size_full().child(format!("Slide {}", index + 1)))
                        })),
                )
                .child(CarouselPrevious::new(&state))
                .child(CarouselNext::new(&state))
                .child(CarouselPagination::new().children(
                    (0..3).map(|index| CarouselPaginationItem::new(("page", index), index, &state)),
                )),
        )
    });
    let selected = |cx: &mut VisualTestContext| cx.update(|_, cx| state.read(cx).selected_index());
    assert_eq!(selected(cx), Some(0));
    // The carousel is a tab stop; arrows move it while it has focus.
    cx.update(|window, cx| window.focus_next(cx));
    cx.simulate_keystrokes("right");
    draw(cx);
    assert_eq!(selected(cx), Some(1));
    cx.simulate_keystrokes("left left");
    draw(cx);
    assert_eq!(selected(cx), Some(2), "looping wraps backwards");
}

#[gpui_kit::test]
fn menus_mount_in_a_menubar_a_navigation_menu_and_a_context_area(cx: &mut TestAppContext) {
    let clicks = Rc::new(RefCell::new(0));
    let sink = clicks.clone();
    let cx = mount(cx, move |window, cx| {
        let sink = sink.clone();
        let file = window.use_keyed_state("menus.file", cx, |_, cx| {
            MenuState::new(
                [
                    MenuItem::new("new", "New"),
                    MenuItem::separator(),
                    MenuItem::new("quit", "Quit").disabled(true),
                ],
                cx,
            )
        });
        let docs = window.use_keyed_state("menus.docs", cx, |_, cx| {
            MenuState::new([MenuItem::new("intro", "Introduction").link()], cx)
        });
        let actions = window.use_keyed_state("menus.actions", cx, |_, cx| {
            MenuState::new(
                [MenuItem::new("copy", "Copy").on_click(move |_, _, _| *sink.borrow_mut() += 1)],
                cx,
            )
        });
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(
                probe("menubar").child(Menubar::new("app", "Application menu").menu(&file, "File")),
            )
            .child(navigation_menu("docs-nav", "Documentation").menu(&docs, "Getting started"))
            .child(context_menu(
                &actions,
                "File actions",
                probe("area")
                    .w(px(200.))
                    .h(px(80.))
                    .child("Right click here"),
            ))
    });
    let area = cx.debug_bounds("area").expect("context area drawn");
    cx.simulate_event(gpui_kit::MouseDownEvent {
        button: gpui_kit::MouseButton::Right,
        position: area.center(),
        modifiers: Modifiers::default(),
        click_count: 1,
        first_mouse: false,
    });
    cx.simulate_event(gpui_kit::MouseUpEvent {
        button: gpui_kit::MouseButton::Right,
        position: area.center(),
        modifiers: Modifiers::default(),
        click_count: 1,
    });
    draw(cx);
    cx.simulate_keystrokes("down enter");
    draw(cx);
    assert_eq!(*clicks.borrow(), 1, "the context menu's first item ran");
}

#[gpui_kit::test]
fn text_controls_mount_with_their_shared_states(cx: &mut TestAppContext) {
    let otp = Rc::new(RefCell::new(None));
    let otp_sink = otp.clone();
    let cx = mount(cx, move |window, cx| {
        let quantity = window.use_keyed_state("text.quantity", cx, |window, cx| {
            InputState::new(window, cx)
                .default_value("3")
                .min(0.)
                .max(20.)
                .step(1.)
        });
        let note = window.use_keyed_state("text.note", cx, |window, cx| {
            TextareaState::new(window, cx).default_value("Release notes")
        });
        let search = window.use_keyed_state("text.search", cx, |window, cx| {
            InputState::new(window, cx).placeholder("Search")
        });
        let code =
            window.use_keyed_state("text.code", cx, |window, cx| OtpState::new(6, window, cx));
        *otp_sink.borrow_mut() = Some(code.clone());
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .w(px(360.))
            .child(NumberField::new(&quantity).aria_label("Quantity"))
            .child(textarea(&note).aria_label("Notes"))
            .child(
                InputGroup::new("search-group", Input::new(&search))
                    .aria_label("Search")
                    .addon(
                        InputGroupAddon::new("search-leading")
                            .child(InputGroupText::new().child("@")),
                    )
                    .addon(
                        InputGroupAddon::new("search-trailing")
                            .align(gpuicn::input_group::InputGroupAddonAlignment::InlineEnd)
                            .button(InputGroupButton::new("go").label("Go")),
                    ),
            )
            .child(
                InputGroup::new("disabled-group", Input::new(&search))
                    .disabled(true)
                    .invalid(true),
            )
            .child(probe("otp").child(OtpField::new(&code).aria_label("Verification code")))
            .child(
                Toolbar::new("tools", "Formatting", cx)
                    .button(ToolbarButton::new("bold", "Bold", cx).child("Bold"))
                    .separator()
                    .input(&search, "Find text", false)
                    .button(
                        ToolbarButton::new("more", "More", cx)
                            .disabled(true)
                            .child("More"),
                    ),
            )
            .child(
                Toolbar::new("side", "Side tools", cx)
                    .axis(Axis::Vertical)
                    .button(ToolbarButton::new("up", "Up", cx).child("Up")),
            )
    });
    click(cx, "otp");
    cx.simulate_keystrokes("1 2 3");
    draw(cx);
    let code = otp.borrow().clone().expect("otp state created");
    assert_eq!(cx.update(|_, cx| code.read(cx).value().to_string()), "123");
}

#[gpui_kit::test]
fn forms_and_overlays_mount(cx: &mut TestAppContext) {
    let cx = mount(cx, move |window, cx| {
        let email = window.use_keyed_state("form.email", cx, |window, cx| {
            InputState::new(window, cx).placeholder("you@example.com")
        });
        form("signup", cx)
            .child(
                fieldset("contact", cx)
                    .child(fieldset_legend(FieldsetLegendVariant::Legend, cx).child("Contact"))
                    .child(fieldset_legend(FieldsetLegendVariant::Label, cx).child("Email"))
                    .child(Input::new(&email)),
            )
            .child(
                probe("popover-trigger").child(
                    popover("dimensions")
                        .trigger(popover_trigger("dimensions.trigger", cx).child("Dimensions"))
                        .content(|_, _, cx| {
                            popover_popup("dimensions.popup", "Dimensions", cx)
                                .child(popover_title(cx).child("Dimensions"))
                                .child(popover_description(cx).child("Set the layer size."))
                                .child(probe("popover-body").child("Width"))
                        }),
                ),
            )
            .child(
                preview_card("profile")
                    .trigger(
                        Button::new("profile.trigger")
                            .variant(ButtonVariant::Link)
                            .label("@gpuicn"),
                    )
                    .content(|_, _, cx| preview_card_popup("profile.popup", cx).child("gpuicn")),
            )
    });
    assert!(cx.debug_bounds("popover-body").is_none());
    click(cx, "popover-trigger");
    assert!(
        cx.debug_bounds("popover-body").is_some(),
        "the popover opened"
    );
}

#[gpui_kit::test]
fn static_components_draw_in_both_themes(cx: &mut TestAppContext) {
    let cx = mount(cx, move |_, cx| {
        let theme = UiTheme::read(cx).clone();
        selection_root("page")
            .flex()
            .flex_col()
            .gap(px(12.))
            .w(px(480.))
            .child(TitleBar::new().child("Matty").on_close(|_, _, _| {}))
            .child(
                div()
                    .flex()
                    .gap(px(8.))
                    .child(Avatar::new("a1").fallback("AJ").size(AvatarSize::Sm))
                    .child(Avatar::new("a2").fallback("CN").aria_label("Claude"))
                    .child(Avatar::new("a3").fallback("XL").size(AvatarSize::Lg))
                    .child(Spinner::new("spin").label("Loading"))
                    .child(Spinner::new("tinted").color(rgb(0x3b82f6)))
                    .child(Skeleton::new("skeleton").w(px(80.)).h(px(16.))),
            )
            .child(Separator::new("rule").horizontal())
            .child(
                div()
                    .flex()
                    .gap(px(8.))
                    .h(px(24.))
                    .child(Tag::new("t1", "Design"))
                    .child(Tag::new("t2", "Bug").variant(TagVariant::Destructive).icon(LucideIcon::Bug))
                    .child(Tag::new("t3", "Live").dot(rgb(0x22c55e)).on_remove(|_, _, _| {}))
                    .child(Separator::new("vrule").vertical()),
            )
            .child(Meter::new("storage").value(0.6).aria_label("Storage"))
            .child(Meter::new("battery").range(0., 100.).value(42.).aria_label("Battery"))
            .child(
                Empty::new()
                    .header(
                        EmptyHeader::new()
                            .media(EmptyMedia::new().child("📭"))
                            .title(EmptyTitle::new().child("No messages"))
                            .description(EmptyDescription::new().child("Start a conversation.")),
                    )
                    .content(EmptyContent::new().child(Button::new("empty.action").label("New message"))),
            )
            .child(markdown(
                "md",
                "**Release day.** `v0.9.0` is tagged.\n\n- Rows\n- Lists\n\n```rust\nlet x = 1;\n```\n\n| A | B |\n| --- | --- |\n| 1 | 2 |",
                cx,
            ))
            .child(day_divider("day", "Today", cx))
            .child(
                Message::new("post")
                    .avatar(Avatar::new("post.avatar").fallback("AJ"))
                    .author("Arya")
                    .timestamp("10:42 AM")
                    .highlight(MessageHighlight::Mention)
                    .status(MessageStatus::Failed)
                    .on_retry(|_, _, _| {})
                    .reactions([Reaction::new("🎉", 3, true)])
                    .child("Shipping 0.9.0")
                    .child(attachment_group(
                        "files",
                        [
                            Attachment::new("plan", "plan.pdf")
                                .meta("PDF · 2.4 MB")
                                .status(AttachmentStatus::Uploading(0.4))
                                .on_remove(|_, _, _| {}),
                            Attachment::new("broken", "notes.txt")
                                .icon(LucideIcon::FileText)
                                .status(AttachmentStatus::Failed)
                                .on_retry(|_, _, _| {}),
                        ],
                        cx,
                    )),
            )
            .child(Message::new("follow").continuation(true).gutter_time("10:43").child("And the docs."))
            .child(
                ScrollArea::new("versions")
                    .aria_label("Versions")
                    .w(px(240.))
                    .h(px(120.))
                    .children((1..=30).map(|i| div().px(px(12.)).py(px(8.)).child(format!("Version 1.0.{i}")))),
            )
            .child(ScrollArea::new("wide").axis(Axis::Horizontal).w(px(240.)).h(px(40.)).child(div().w(px(900.)).child("Wide")))
            .child(div().text_color(theme.colors.muted_foreground).child("End"))
    });
    assert!(cx.debug_bounds("missing").is_none());
}
