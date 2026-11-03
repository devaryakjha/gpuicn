//! The menu catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn menu_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sign_out = cx.listener(|this, _, _, cx| {
            this.demo_action = "Signed out".into();
            cx.notify();
        });
        let menu = window.use_keyed_state("account-menu", cx, |_, cx| {
            MenuState::new(
                [
                    MenuItem::label("account-heading", "My account"),
                    MenuItem::new("profile", "Profile"),
                    MenuItem::new("notifications", "Notifications").checked(true),
                    MenuItem::separator(),
                    MenuItem::submenu(
                        "invite",
                        "Invite",
                        [
                            MenuItem::new("email", "Email"),
                            MenuItem::new("message", "Message"),
                        ],
                    ),
                    MenuItem::new("billing", "Billing").disabled(true),
                    MenuItem::new("sign-out", "Sign out").on_click(sign_out),
                ],
                cx,
            )
        });
        Menu::new(&menu, "My account")
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.menu_preview(window, cx).into_any_element()
}
