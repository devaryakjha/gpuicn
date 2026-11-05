//! Right-to-left layouts mirror components, and built-in strings translate.

use gpui_icons::{LucideIcon, lucide};
use gpui_kit::{
    AppContext as _, Bounds, InteractiveElement as _, Context, Entity, IntoElement, ParentElement, Pixels, Render, Styled,
    TestAppContext, VisualTestContext, Window, div, px, size,
};
use gpuicn::{
    Button, Direction,
    drawer::{Drawer, DrawerContent, DrawerHandle, DrawerSide},
    input::{Input, InputState},
    locale,
    menu::{Menu, MenuItem, MenuState},
};

struct Gallery {
    menu: Entity<MenuState>,
    input: Entity<InputState>,
    drawer: DrawerHandle,
}

impl Render for Gallery {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(16.))
            .p(px(24.))
            .child(
                Button::new("save")
                    .child(
                        div()
                            .debug_selector(|| "save-icon".into())
                            .child(lucide(LucideIcon::Save).size(px(16.))),
                    )
                    .child(div().debug_selector(|| "save-label".into()).child("Save")),
            )
            .child(
                div()
                    .debug_selector(|| "menu-trigger".into())
                    .child(Menu::new(&self.menu, "Options")),
            )
            .child(
                div()
                    .w(px(240.))
                    .debug_selector(|| "input-frame".into())
                    .child(Input::new(&self.input).aria_label("Name")),
            )
            .child(
                Drawer::new("nav", &self.drawer)
                    .direction(DrawerSide::Left)
                    .content(DrawerContent::new("nav.panel", "Navigation").child("Inbox")),
            )
    }
}

fn open(direction: Direction, cx: &mut TestAppContext) -> (Entity<Gallery>, VisualTestContext) {
    cx.update(|cx| {
        gpuicn::init(cx);
        locale::set_direction(cx, direction);
    });
    let (view, cx) = cx.add_window_view(|window, cx| Gallery {
        menu: cx.new(|cx| {
            MenuState::new(
                [
                    MenuItem::new("copy", "Copy"),
                    MenuItem::submenu(
                        "share",
                        "Share",
                        [MenuItem::new("mail", "Mail"), MenuItem::new("link", "Link")],
                    ),
                ],
                cx,
            )
        }),
        input: cx.new(|cx| InputState::new(window, cx).default_value("abc")),
        drawer: DrawerHandle::new(false),
    });
    cx.simulate_resize(size(px(800.), px(600.)));
    let mut cx = cx.clone();
    draw(&mut cx);
    (view, cx)
}

fn draw(cx: &mut VisualTestContext) {
    for _ in 0..3 {
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.run_until_parked();
    }
}

fn bounds(cx: &mut VisualTestContext, selector: &'static str) -> Bounds<Pixels> {
    cx.debug_bounds(selector)
        .unwrap_or_else(|| panic!("{selector} is drawn"))
}

#[gpui_kit::test]
fn button_icons_lead_from_the_reading_start(cx: &mut TestAppContext) {
    let (_, mut ltr) = open(Direction::Ltr, cx);
    assert!(bounds(&mut ltr, "save-icon").left() < bounds(&mut ltr, "save-label").left());

    let mut rtl_app = TestAppContext::single();
    let (_, mut rtl) = open(Direction::Rtl, &mut rtl_app);
    let icon = bounds(&mut rtl, "save-icon");
    let label = bounds(&mut rtl, "save-label");
    assert!(icon.left() > label.left(), "{icon:?} should follow {label:?}");
}

#[gpui_kit::test]
fn menus_align_to_the_trigger_end_and_submenus_open_toward_the_start(cx: &mut TestAppContext) {
    let (view, mut cx) = open(Direction::Rtl, cx);
    cx.update(|window, cx| {
        let menu = view.read(cx).menu.clone();
        menu.update(cx, |menu, cx| menu.open(window, cx));
    });
    draw(&mut cx);
    let trigger = bounds(&mut cx, "menu-trigger");
    let popup = bounds(&mut cx, "menu-popup-0");
    assert!(
        (popup.right() - trigger.right()).abs() < px(1.),
        "{popup:?} should align its right edge with {trigger:?}"
    );

    // Down to Share, then the start-side arrow, which is Left in RTL.
    cx.simulate_keystrokes("down left");
    draw(&mut cx);
    let submenu = bounds(&mut cx, "menu-popup-1");
    assert!(
        submenu.right() <= popup.left() + px(1.),
        "{submenu:?} should open left of {popup:?}"
    );
    // Right goes back toward the parent.
    cx.simulate_keystrokes("right");
    draw(&mut cx);
    assert!(cx.debug_bounds("menu-popup-1").is_none());
}

#[gpui_kit::test]
fn a_left_drawer_opens_from_the_right_edge_in_rtl(cx: &mut TestAppContext) {
    let (view, mut cx) = open(Direction::Rtl, cx);
    cx.update(|window, cx| view.read(cx).drawer.clone().open(window, cx));
    // Let the presence animation settle.
    for _ in 0..40 {
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(16));
        draw(&mut cx);
    }
    let panel = bounds(&mut cx, "drawer-panel");
    assert!(
        (panel.right() - px(800.)).abs() < px(1.),
        "{panel:?} should touch the right edge"
    );
    assert!(panel.left() > px(400.));
}

#[gpui_kit::test]
fn the_input_caret_starts_at_the_right_and_arrows_follow_the_direction(cx: &mut TestAppContext) {
    let (view, mut cx) = open(Direction::Rtl, cx);
    let input = cx.update(|_, cx| view.read(cx).input.clone());
    cx.update(|window, cx| {
        input.update(cx, |input, cx| {
            input.focus(window, cx);
            input.set_selected_range(0..0, cx);
        })
    });
    draw(&mut cx);
    let frame = bounds(&mut cx, "input-frame");
    let caret = cx.update(|_, cx| {
        input
            .read(cx)
            .prepaint_caret_geometry()
            .expect("the caret is laid out")
            .cursor_bounds()
    });
    // "abc" is aligned to the right edge; offset 0 is its left end.
    assert!(
        caret.left() > frame.center().x,
        "{caret:?} should sit in the right half of {frame:?}"
    );
    // Left moves toward the end of the text in RTL.
    cx.simulate_keystrokes("left");
    cx.update(|_, cx| assert_eq!(input.read(cx).cursor(), 1));
    cx.simulate_keystrokes("right");
    cx.update(|_, cx| assert_eq!(input.read(cx).cursor(), 0));
}

#[gpui_kit::test]
fn built_in_strings_translate_without_forking(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpuicn::init(cx);
        locale::set_strings(cx, |text| match text {
            gpuicn::UiText::SearchEmoji => Some("ابحث عن رمز تعبيري".into()),
            _ => None,
        });
        assert_eq!(
            locale::text(cx, gpuicn::UiText::SearchEmoji),
            "ابحث عن رمز تعبيري"
        );
        assert_eq!(locale::text(cx, gpuicn::UiText::Close), "Close");
    });
}
