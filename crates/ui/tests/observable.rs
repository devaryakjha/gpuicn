//! Interactive components register for GPUI Kit's element queries with the
//! role and accessible name they give assistive technology, so an application
//! can audit names in a headless test.
use gpui_kit::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, Role, Styled,
    TestAppContext, Window, div, px, test::TestQueryExt as _,
};
use gpuicn::{
    button::Button,
    checkbox::Checkbox,
    command::{Command, CommandItem, CommandState},
    composer::{Composer, ComposerState},
    input::{Input, InputState, TextareaState},
    menu::{Menu, MenuItem, MenuState},
    popover::hover_popover,
    select::{Select, SelectItem, SelectState},
    sidebar::{Sidebar, SidebarItem},
    switch::Switch,
    tabs::{Tab, Tabs},
    textarea::textarea,
    toggle::Toggle,
};

struct Screen {
    input: Entity<InputState>,
    notes: Entity<TextareaState>,
    composer: Entity<ComposerState>,
    select: Entity<SelectState>,
    menu: Entity<MenuState>,
    command: Entity<CommandState>,
}

impl Render for Screen {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w(px(600.))
            .child(Button::new("save").label("Save"))
            .child(Button::new("icon-only").child("+"))
            .child(Toggle::new("bold").aria_label("Bold").child("B"))
            .child(Checkbox::new("terms").aria_label("Accept terms"))
            .child(Switch::new("wifi").aria_label("Wi-Fi"))
            .child(Select::new(&self.select).aria_label("Fruit"))
            .child(Menu::new(&self.menu, "Message actions"))
            .child(
                Tabs::new("settings")
                    .aria_label("Settings")
                    .selected("account")
                    .item(Tab::new("account", "account", "Account"))
                    .item(Tab::new("billing", "billing", "Billing")),
            )
            .child(Input::new(&self.input).aria_label("Search"))
            .child(textarea(&self.notes).aria_label("Notes"))
            .child(Composer::new(&self.composer))
            .child(
                hover_popover("profile", "Priya Nair, online")
                    .trigger("Priya")
                    .content(|_, _| div()),
            )
            .child(
                Sidebar::new("nav", "Channels")
                    .child(SidebarItem::new("general", "general"))
                    .child(SidebarItem::new("random", "random")),
            )
            .child(Command::new(&self.command))
    }
}

#[gpui_kit::test]
fn interactive_components_report_their_role_and_name(cx: &mut TestAppContext) {
    cx.update(gpuicn::init);
    let (_, cx) = cx.add_window_view(|window, cx| Screen {
        input: cx.new(|cx| InputState::new(window, cx)),
        notes: cx.new(|cx| TextareaState::new(window, cx)),
        composer: cx.new(|cx| ComposerState::new(4, window, cx)),
        select: cx.new(|cx| SelectState::new([SelectItem::new("apple", "Apple")], window, cx)),
        menu: cx.new(|cx| MenuState::new([MenuItem::new("reply", "Reply")], cx)),
        command: cx
            .new(|cx| CommandState::new([CommandItem::new("general", "#general")], window, cx)),
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.update(|window, _| {
        let named = |label: &str, role: Role| {
            let element = window.find_by_label(label);
            assert_eq!(element.role(), Some(role), "{label}");
        };
        named("Save", Role::Button);
        named("Bold", Role::Button);
        named("Accept terms", Role::CheckBox);
        named("Wi-Fi", Role::Switch);
        named("Fruit", Role::ComboBox);
        named("Message actions", Role::Button);
        named("Account", Role::Tab);
        named("Search", Role::TextInput);
        named("Notes", Role::TextInput);
        named("Message", Role::TextInput);
        named("Priya Nair, online", Role::Button);
        named("general", Role::Button);
        named("#general", Role::ListBoxOption);

        // An icon-only button without an accessible name is still reported,
        // with no label, which is what an audit looks for.
        let icon_only = window
            .elements()
            .into_iter()
            .find(|element| element.path().last() == Some(&"icon-only".into()))
            .expect("the icon-only button is observed");
        assert_eq!(icon_only.role(), Some(Role::Button));
        assert_eq!(icon_only.label(), None);
    });
}
