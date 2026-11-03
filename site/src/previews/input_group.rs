//! The input group catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn input_group_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let url = demo_input("input-group.url", "", "example.com", window, cx);
        let search = demo_input("input-group.search", "", "Search", window, cx);
        let message = window.use_keyed_state("input-group.message", cx, |window, cx| {
            TextareaState::new(window, cx)
                .rows(4)
                .placeholder("Write a message")
        });

        div()
            .w(px(420.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(
                InputGroup::new("url-group", InputGroupInput::new(&url))
                    .aria_label("Website address")
                    .addon(
                        InputGroupAddon::new("scheme")
                            .child(InputGroupText::new().child("https://")),
                    )
                    .addon(
                        InputGroupAddon::new("url-action")
                            .align(InputGroupAddonAlignment::InlineEnd)
                            .button(InputGroupButton::new("copy-url").label("Copy")),
                    ),
            )
            .child(
                InputGroup::new("search-group", InputGroupInput::new(&search))
                    .aria_label("Search")
                    .disabled(true)
                    .addon(
                        InputGroupAddon::new("search-status")
                            .align(InputGroupAddonAlignment::InlineEnd)
                            .child(InputGroupText::new().child("⌘ K")),
                    ),
            )
            .child(
                InputGroup::new("message-group", InputGroupTextarea::new(&message))
                    .aria_label("Message")
                    .addon(
                        InputGroupAddon::new("message-header")
                            .align(InputGroupAddonAlignment::BlockStart)
                            .child(InputGroupText::new().child("Message")),
                    )
                    .addon(
                        InputGroupAddon::new("message-toolbar")
                            .align(InputGroupAddonAlignment::BlockEnd)
                            .child(InputGroupText::new().child("Markdown supported"))
                            .button(InputGroupButton::new("send-message").label("Send")),
                    ),
            )
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.input_group_preview(window, cx).into_any_element()
}
