//! The sidebar catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn sidebar_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        match self.sidebar_example.as_str() {
            "mail" => self.sidebar_mail_preview(window, cx).into_any_element(),
            "docs" => self.sidebar_docs_preview(window, cx).into_any_element(),
            _ => self
                .sidebar_application_preview(window, cx)
                .into_any_element(),
        }
    }

    fn sidebar_application_preview(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        use gpui_kit::FontWeight;
        use gpuicn::sidebar::*;
        let t = UiTheme::read(cx).clone();
        let s = t.spacing.unit;
        let example = self.sidebar_example.as_str();
        let mobile = self.sidebar_mobile || example == "mobile";
        let mode = if example == "loading" {
            SidebarCollapsible::None
        } else {
            SidebarCollapsible::Icon
        };
        let collapsed = self.sidebar_state.icon_collapsed(mobile, mode);
        let variant = match example {
            "workspace" => SidebarVariant::Inset,
            "floating" => SidebarVariant::Floating,
            _ => SidebarVariant::Sidebar,
        };
        let right = example == "floating";
        let labels: &[(&str, LucideIcon)] = match example {
            "floating" => &[
                ("Overview", LucideIcon::LayoutDashboard),
                ("Activity", LucideIcon::Activity),
                ("Files", LucideIcon::Folder),
                ("Settings", LucideIcon::Settings),
            ],
            _ => &[
                ("Overview", LucideIcon::House),
                ("Projects", LucideIcon::Folder),
                ("Inbox", LucideIcon::Inbox),
                ("Team", LucideIcon::Users),
            ],
        };
        let selected = self.sidebar_selected.min(labels.len() - 1);
        let title = labels[selected].0;
        let workspace = if self.sidebar_workspace == 0 {
            "Acme Studio"
        } else {
            "Personal workspace"
        };
        let toggle = cx.listener(move |this, _, _, cx| {
            this.sidebar_state.toggle(mobile);
            cx.notify();
        });
        let items = ["Acme Studio", "Personal workspace"]
            .into_iter()
            .enumerate()
            .map(|(index, label)| {
                let view = cx.entity().downgrade();
                MenuItem::new(label, label).on_click(move |_, _, cx| {
                    let _ = view.update(cx, |this, cx| {
                        this.sidebar_workspace = index;
                        cx.notify();
                    });
                })
            })
            .collect::<Vec<_>>();
        let state = window
            .use_keyed_state("sidebar.team.state", cx, |_, cx| {
                cx.new(|cx| MenuState::new(items, cx))
            })
            .read(cx)
            .clone();
        let team_menu = sidebar_dropdown(&state, "Switch workspace", collapsed, cx).trigger(
            div()
                .flex_row_in(t.direction)
                .w_full()
                .items_center()
                .gap(s * 2_f32)
                .child(
                    div()
                        .size(s * 8_f32)
                        .flex_shrink_0()
                        .rounded(t.radius.lg)
                        .bg(t.colors.sidebar_primary)
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            lucide(LucideIcon::Command)
                                .size(s * 4_f32)
                                .text_color(t.colors.sidebar_primary_foreground),
                        ),
                )
                .when(!collapsed, |el| {
                    el.child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .truncate()
                                    .font_weight(FontWeight::MEDIUM)
                                    .line_height(px(17.5) * t.text_scale)
                                    .child(workspace),
                            )
                            .child(
                                div()
                                    .truncate()
                                    .text_size(px(12.) * t.text_scale)
                                    .line_height(px(16.) * t.text_scale)
                                    .child("Enterprise"),
                            ),
                    )
                    .child(
                        lucide(LucideIcon::ChevronsUpDown)
                            .size(s * 4_f32)
                            .text_color(t.colors.sidebar_foreground),
                    )
                }),
        );
        let header = div().flex().flex_col().gap(s * 2_f32).child(team_menu);
        let mut menu = sidebar_menu();
        let loading = example == "loading" && !self.sidebar_loaded;
        if loading {
            for _ in 0..5 {
                menu = menu.child(sidebar_menu_skeleton(!collapsed, cx));
            }
        } else {
            for (index, (label, icon)) in labels.iter().enumerate() {
                let expandable = index == 1 && example == "workspace";
                let item = SidebarItem::new(("sidebar.nav", index), (*label).to_owned())
                    .icon(
                        lucide(*icon)
                            .size(s * 4_f32)
                            .text_color(t.colors.sidebar_foreground),
                    )
                    .collapsed(collapsed)
                    .selected(selected == index)
                    .when(expandable, |el| el.expanded(self.sidebar_nested))
                    .when(index == 1 && example == "workspace" && !collapsed, |el| {
                        el.trailing(
                            lucide(if self.sidebar_nested {
                                LucideIcon::ChevronDown
                            } else {
                                LucideIcon::ChevronRight
                            })
                            .size(s * 4_f32)
                            .text_color(t.colors.sidebar_foreground),
                        )
                    })
                    .when(*label == "Inbox" && !collapsed, |el| {
                        el.trailing(sidebar_badge("12", cx))
                    })
                    .on_activate(cx.listener(move |this, _, _, cx| {
                        this.sidebar_selected = index;
                        if expandable {
                            this.sidebar_nested = !this.sidebar_nested;
                        } else {
                            this.sidebar_state.mobile_open = false;
                        }
                        cx.notify();
                    }));
                menu = menu.child(item);
                if example == "workspace" && index == 1 && !collapsed && self.sidebar_nested {
                    menu = menu.child(
                        sidebar_menu_sub(cx)
                            .child(
                                SidebarItem::new("sidebar.nested.website", "Website")
                                    .size(SidebarItemSize::Small)
                                    .on_activate(cx.listener(|this, _, _, cx| {
                                        this.sidebar_note = "Website project opened".into();
                                        cx.notify();
                                    })),
                            )
                            .child(
                                SidebarItem::new("sidebar.nested.mobile", "Mobile app")
                                    .size(SidebarItemSize::Small)
                                    .on_activate(cx.listener(|this, _, _, cx| {
                                        this.sidebar_note = "Mobile app project opened".into();
                                        cx.notify();
                                    })),
                            ),
                    );
                }
            }
        }
        let mut navigation = Sidebar::new("sidebar.navigation", "Main navigation")
            .roving_focus(true)
            .header(header)
            .when(!collapsed, |el| {
                el.child(sidebar_group_label("Platform", cx))
            })
            .child(menu);
        if !collapsed {
            navigation = navigation.child(div().h(s * 4_f32).flex_shrink_0()).child(
                div()
                    .flex_row_in(t.direction)
                    .items_center()
                    .justify_between()
                    .child(sidebar_group_label("Projects", cx))
                    .child(
                        sidebar_menu_action("sidebar.add", "Add project", cx)
                            .child(
                                lucide(LucideIcon::Plus)
                                    .size(s * 4_f32)
                                    .text_color(t.colors.sidebar_foreground),
                            )
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.sidebar_projects += 1;
                                this.sidebar_note =
                                    format!("Created project {}", this.sidebar_projects);
                                cx.notify();
                            })),
                    ),
            );
            for index in 0..self.sidebar_projects {
                navigation = navigation.child(
                    SidebarItem::new(("sidebar.project", index), format!("Project {}", index + 1))
                        .icon(
                            lucide(LucideIcon::Folder)
                                .size(s * 4_f32)
                                .text_color(t.colors.sidebar_foreground),
                        )
                        .on_activate(cx.listener(move |this, _, _, cx| {
                            this.sidebar_note = format!("Project {} opened", index + 1);
                            cx.notify();
                        })),
                );
            }
        }
        let items = vec![
            MenuItem::new("view-profile", "View profile").on_click({
                let view = cx.entity().downgrade();
                move |_, _, cx| {
                    let _ = view.update(cx, |this, cx| {
                        this.sidebar_note = "Alex Morgan · alex@example.com".into();
                        cx.notify();
                    });
                }
            }),
            MenuItem::new("notifications", "Notifications").checked(true),
        ];
        let state = window
            .use_keyed_state("sidebar.account.state", cx, |_, cx| {
                cx.new(|cx| MenuState::new(items, cx))
            })
            .read(cx)
            .clone();
        let account = sidebar_dropdown(&state, "Account menu", collapsed, cx).trigger(
            div()
                .flex_row_in(t.direction)
                .w_full()
                .items_center()
                .gap(s * 2_f32)
                .child(
                    div()
                        .size(s * 8_f32)
                        .flex_shrink_0()
                        .rounded(t.radius.lg)
                        .bg(t.colors.sidebar_accent)
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(12.) * t.text_scale)
                        .line_height(px(16.) * t.text_scale)
                        .font_weight(FontWeight::MEDIUM)
                        .child("AM"),
                )
                .when(!collapsed, |el| {
                    el.child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .truncate()
                                    .font_weight(FontWeight::MEDIUM)
                                    .line_height(px(17.5) * t.text_scale)
                                    .child("Alex Morgan"),
                            )
                            .child(
                                div()
                                    .truncate()
                                    .text_size(px(12.) * t.text_scale)
                                    .line_height(px(16.) * t.text_scale)
                                    .child("alex@example.com"),
                            ),
                    )
                    .child(
                        lucide(LucideIcon::ChevronsUpDown)
                            .size(s * 4_f32)
                            .text_color(t.colors.sidebar_foreground),
                    )
                }),
        );
        navigation = navigation.footer(
            div()
                .flex()
                .flex_col()
                .gap(s * 2_f32)
                .when(mobile, |el| {
                    el.child(
                        Button::new("sidebar.close")
                            .variant(ButtonVariant::Ghost)
                            .label("Close navigation")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.sidebar_state.mobile_open = false;
                                cx.notify();
                            })),
                    )
                })
                .child(account),
        );
        let mut body = div().flex_1().min_h_0().min_w_0().p(s * 5_f32).flex().flex_col().gap(s * 4_f32)
            .child(div().text_size(px(20.) * t.text_scale).line_height(px(28.) * t.text_scale).font_weight(FontWeight::MEDIUM).child(title.to_owned()))
            .child(div().text_color(t.colors.muted_foreground).child(match example {
                "floating" => "Project navigation on the right, inside a floating surface.",
                "mobile" => "Open navigation to try the modal sheet. Escape and outside clicks close it.",
                "loading" => "Loading placeholders keep navigation stable until data arrives.",
                _ => "Your workspace at a glance. Switch teams, browse projects, or check your account.",
            }));
        body = body.child(div().flex().gap(s * 3_f32).children(
            [("Projects", "12"), ("Members", "8")].map(|(label, value)| {
                div()
                    .flex_1()
                    .min_w_0()
                    .p(s * 4_f32)
                    .border_1()
                    .border_color(t.colors.border)
                    .rounded(t.radius.lg)
                    .child(div().text_color(t.colors.muted_foreground).child(label))
                    .child(
                        div()
                            .text_size(px(24.) * t.text_scale)
                            .line_height(px(32.) * t.text_scale)
                            .child(value),
                    )
            }),
        ));
        if example == "loading" {
            body = body.child(
                Button::new("sidebar.load")
                    .variant(ButtonVariant::Outline)
                    .child(if loading {
                        "Load navigation"
                    } else {
                        "Show loading state"
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.sidebar_loaded = !this.sidebar_loaded;
                        cx.notify();
                    })),
            );
        }
        body = body.child(
            div()
                .text_size(px(12.) * t.text_scale)
                .line_height(px(16.) * t.text_scale)
                .text_color(t.colors.muted_foreground)
                .child(self.sidebar_note.clone()),
        );
        let content = div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .h(s * 12_f32)
                    .flex_shrink_0()
                    .px(s * 3_f32)
                    .border_b_1()
                    .border_color(t.colors.border)
                    .flex_row_in(t.direction)
                    .items_center()
                    .gap(s * 3_f32)
                    .when(mode != SidebarCollapsible::None, |el| {
                        el.child(sidebar_trigger("sidebar.toggle", toggle, cx))
                    })
                    .child(workspace),
            )
            .child(body);
        SidebarLayout::new("sidebar.layout", self.sidebar_state)
            .mobile(mobile)
            .sidebar(navigation)
            .content(content)
            .on_change(cx.listener(|this, state: &SidebarState, _, cx| {
                this.sidebar_state = *state;
                cx.notify();
            }))
            .variant(variant)
            .collapsible(mode)
            .rail(true)
            .side(if right {
                SidebarSide::Right
            } else {
                SidebarSide::Left
            })
    }

    fn sidebar_mail_preview(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let search_state = live_input(
            "mail.search",
            self.sidebar_search.clone(),
            "Search mail…",
            window,
            cx,
            |this, input, event, cx| {
                if matches!(event, InputEvent::Change) {
                    this.sidebar_search = input.read(cx).value().to_string();
                    cx.notify();
                }
            },
        );
        use gpui_kit::FontWeight;
        use gpuicn::sidebar::*;
        let t = UiTheme::read(cx).clone();
        let s = t.spacing.unit;
        let mobile = self.sidebar_mobile;
        let collapsed = self
            .sidebar_state
            .icon_collapsed(mobile, SidebarCollapsible::Icon);
        let folders = [
            ("Inbox", LucideIcon::Inbox),
            ("Drafts", LucideIcon::File),
            ("Sent", LucideIcon::Send),
            ("Archive", LucideIcon::Archive),
            ("Trash", LucideIcon::Trash),
        ];
        let folder = self.sidebar_selected.min(folders.len() - 1);
        let messages = [
            (
                "William Smith",
                "Meeting tomorrow",
                "Hi team, just a reminder about our meeting tomorrow at 10 AM. We’ll review the new design and agree on the next steps.",
                "9:34 AM",
            ),
            (
                "Alice Smith",
                "Re: Project update",
                "Thanks for the update. The progress looks great so far. I’ve added a few comments to the proposal.",
                "Yesterday",
            ),
            (
                "Bob Johnson",
                "Weekend plans",
                "Hey everyone! I’m thinking of organizing a team outing this weekend. Let me know if you can make it.",
                "2 days ago",
            ),
            (
                "Emily Davis",
                "Question about the budget",
                "I’ve reviewed the budget numbers you sent over. Can we find a time to go through the details?",
                "2 days ago",
            ),
            (
                "Michael Wilson",
                "An update from the team",
                "Please join us for our next team meeting. We’ll share what we’ve shipped and what comes next.",
                "1 week ago",
            ),
            (
                "Sarah Brown",
                "Feedback on the proposal",
                "I had a chance to review the proposal. I have a few thoughts and would love to discuss them.",
                "1 week ago",
            ),
        ];
        let selected = self.sidebar_message.min(messages.len() - 1);
        let (sender, subject, message, time) = messages[selected];
        let mut rail = Sidebar::new("mail.folders", "Mail folders")
            .header(
                div()
                    .size(s * 8_f32)
                    .rounded(t.radius.lg)
                    .bg(t.colors.sidebar_primary)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        lucide(LucideIcon::Command)
                            .size(s * 4_f32)
                            .text_color(t.colors.sidebar_primary_foreground),
                    ),
            )
            .footer(
                SidebarItem::new("mail.account", "Account")
                    .collapsed(true)
                    .icon(
                        div()
                            .size(s * 8_f32)
                            .rounded(t.radius.lg)
                            .bg(t.colors.sidebar_accent)
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(12.) * t.text_scale)
                            .line_height(px(16.) * t.text_scale)
                            .child("AM"),
                    )
                    .on_activate(cx.listener(|this, _, _, cx| {
                        this.sidebar_note = "Alex Morgan · alex@example.com".into();
                        cx.notify();
                    })),
            );
        for (index, (name, icon)) in folders.iter().enumerate() {
            rail = rail.child(
                SidebarItem::new(("mail.folder", index), *name)
                    .collapsed(true)
                    .selected(folder == index)
                    .icon(
                        lucide(*icon)
                            .size(s * 4_f32)
                            .text_color(t.colors.sidebar_foreground),
                    )
                    .on_activate(cx.listener(move |this, _, _, cx| {
                        this.sidebar_selected = index;
                        this.sidebar_message = 0;
                        cx.notify();
                    })),
            );
        }
        let search = Input::new(&search_state).aria_label("Search mail");
        let mut list = Sidebar::new("mail.list", "Messages")
            .content_padding(px(0.))
            .header(
                div()
                    .flex()
                    .flex_col()
                    .gap(s * 3_f32)
                    .p(s * 2_f32)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap(s * 2_f32)
                            .child(
                                div()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(folders[folder].0),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(s * 2_f32)
                                    .child("Unread")
                                    .child(
                                        Switch::new("mail.unread")
                                            .aria_label("Show unread messages only")
                                            .checked(self.sidebar_unread)
                                            .on_change({
                                                let view = cx.entity().downgrade();
                                                move |checked, _, _, cx| {
                                                    let _ = view.update(cx, |this, cx| {
                                                        this.sidebar_unread = checked;
                                                        cx.notify();
                                                    });
                                                }
                                            }),
                                    ),
                            ),
                    )
                    .child(search),
            );
        let query = self.sidebar_search.to_lowercase();
        let mut count = 0;
        for (index, (sender, subject, body, time)) in messages.iter().enumerate() {
            if self.sidebar_unread && index % 2 != 0 {
                continue;
            }
            if !format!("{sender} {subject}")
                .to_lowercase()
                .contains(&query)
            {
                continue;
            }
            count += 1;
            list = list.child(
                Button::new(("mail.message", index))
                    .variant(ButtonVariant::Ghost)
                    .w_full()
                    .h(s * 28_f32)
                    .rounded(px(0.))
                    .p(s * 4_f32)
                    .border_0()
                    .border_b_1()
                    .border_color(t.colors.sidebar_border)
                    .when(selected == index, |el| el.bg(t.colors.sidebar_accent))
                    .child(
                        div()
                            .w_full()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap(s)
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .gap(s * 2_f32)
                                    .child(
                                        div()
                                            .truncate()
                                            .font_weight(FontWeight::NORMAL)
                                            .line_height(px(17.5) * t.text_scale)
                                            .child(*sender),
                                    )
                                    .child(
                                        div()
                                            .flex_shrink_0()
                                            .text_size(px(12.) * t.text_scale)
                                            .line_height(px(16.) * t.text_scale)
                                            .text_color(t.colors.muted_foreground)
                                            .child(*time),
                                    ),
                            )
                            .child(
                                div()
                                    .truncate()
                                    .font_weight(FontWeight::MEDIUM)
                                    .line_height(px(17.5) * t.text_scale)
                                    .child(*subject),
                            )
                            .child(
                                div()
                                    .truncate()
                                    .text_size(px(12.) * t.text_scale)
                                    .line_height(px(16.) * t.text_scale)
                                    .text_color(t.colors.muted_foreground)
                                    .child(*body),
                            ),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.sidebar_message = index;
                        this.sidebar_state.mobile_open = false;
                        cx.notify();
                    })),
            );
        }
        if count == 0 {
            list = list.child(
                div()
                    .p(s * 4_f32)
                    .text_color(t.colors.muted_foreground)
                    .child("No matching messages"),
            );
        }
        let navigation = div()
            .size_full()
            .flex()
            .child(div().w(s * 12_f32).h_full().flex_shrink_0().child(rail))
            .when(!collapsed, |el| {
                el.child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .h_full()
                        .border_l_1()
                        .border_color(t.colors.sidebar_border)
                        .child(list),
                )
            });
        let content = div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .h(s * 14_f32)
                    .px(s * 3_f32)
                    .flex_row_in(t.direction)
                    .items_center()
                    .gap(s * 3_f32)
                    .border_b_1()
                    .border_color(t.colors.border)
                    .child(sidebar_trigger(
                        "mail.toggle",
                        cx.listener(move |this, _, _, cx| {
                            this.sidebar_state.toggle(mobile);
                            cx.notify();
                        }),
                        cx,
                    ))
                    .child(
                        div()
                            .text_color(t.colors.muted_foreground)
                            .child("All inboxes"),
                    )
                    .child(
                        lucide(LucideIcon::ChevronRight)
                            .size(s * 3_f32)
                            .text_color(t.colors.muted_foreground),
                    )
                    .child(folders[folder].0),
            )
            .child(
                div()
                    .id("mail.reader")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .p(s * 5_f32)
                    .child(
                        div()
                            .text_size(px(18.) * t.text_scale)
                            .line_height(px(24.) * t.text_scale)
                            .font_weight(FontWeight::MEDIUM)
                            .child(subject),
                    )
                    .child(
                        div()
                            .mt(s * 3_f32)
                            .text_size(px(12.) * t.text_scale)
                            .line_height(px(16.) * t.text_scale)
                            .text_color(t.colors.muted_foreground)
                            .child(format!("{sender} · {time}")),
                    )
                    .child(
                        div()
                            .mt(s * 6_f32)
                            .text_size(px(14.) * t.text_scale)
                            .child(message),
                    )
                    .child(
                        div()
                            .mt(s * 5_f32)
                            .text_color(t.colors.muted_foreground)
                            .child(self.sidebar_note.clone()),
                    ),
            );
        SidebarLayout::new("mail.layout", self.sidebar_state)
            .mobile(mobile)
            .sidebar(navigation)
            .content(content)
            .on_change(cx.listener(|this, state: &SidebarState, _, cx| {
                this.sidebar_state = *state;
                cx.notify();
            }))
            .width(s * 87_f32)
            .collapsible(SidebarCollapsible::Icon)
            .rail(true)
    }

    fn sidebar_docs_preview(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let search_state = live_input(
            "docs.search",
            self.sidebar_search.clone(),
            "Search documentation…",
            window,
            cx,
            |this, input, event, cx| {
                if matches!(event, InputEvent::Change) {
                    this.sidebar_search = input.read(cx).value().to_string();
                    cx.notify();
                }
            },
        );
        use gpui_kit::FontWeight;
        use gpuicn::sidebar::*;
        let t = UiTheme::read(cx).clone();
        let s = t.spacing.unit;
        let mobile = self.sidebar_mobile;
        let groups = [
            (
                "Getting started",
                ["Introduction", "Installation", "Project structure", "CLI"],
            ),
            (
                "Build your application",
                ["Components", "Theme tokens", "Typography", "Accessibility"],
            ),
            (
                "Resources",
                ["Registry", "Examples", "Changelog", "Contributing"],
            ),
        ];
        let selected = self.sidebar_selected.min(11);
        let title = groups[selected / 4].1[selected % 4];
        let header = div()
            .flex()
            .flex_col()
            .gap(s * 2_f32)
            .child(
                div()
                    .h(s * 10_f32)
                    .px(s * 2_f32)
                    .flex()
                    .items_center()
                    .gap(s * 2_f32)
                    .child(
                        lucide(LucideIcon::BookOpen)
                            .size(s * 5_f32)
                            .text_color(t.colors.sidebar_foreground),
                    )
                    .child(div().font_weight(FontWeight::MEDIUM).child("Documentation")),
            )
            .child(Input::new(&search_state).aria_label("Search documentation pages"));
        let query = self.sidebar_search.to_lowercase();
        let mut navigation = Sidebar::new("docs.navigation", "Documentation pages")
            .header(header)
            .footer(
                div()
                    .px(s * 2_f32)
                    .py(s * 2_f32)
                    .text_size(px(12.) * t.text_scale)
                    .line_height(px(16.) * t.text_scale)
                    .text_color(t.colors.muted_foreground)
                    .child("gpuicn · Developer documentation"),
            );
        let mut count = 0;
        for (section, (label, pages)) in groups.iter().enumerate() {
            let matching: Vec<_> = pages
                .iter()
                .enumerate()
                .filter(|(_, title)| title.to_lowercase().contains(&query))
                .collect();
            if matching.is_empty() {
                continue;
            }
            count += matching.len();
            let open = self.sidebar_sections[section] || !query.is_empty();
            let mut group = sidebar_group().mb(s * 3_f32).child(
                SidebarItem::new(("docs.section", section), *label)
                    .expanded(open)
                    .trailing(
                        lucide(if open {
                            LucideIcon::ChevronDown
                        } else {
                            LucideIcon::ChevronRight
                        })
                        .size(s * 4_f32)
                        .text_color(t.colors.sidebar_foreground),
                    )
                    .on_activate(cx.listener(move |this, _, _, cx| {
                        this.sidebar_sections[section] = !this.sidebar_sections[section];
                        cx.notify();
                    })),
            );
            if open {
                let mut submenu = sidebar_menu_sub(cx);
                for (index, title) in matching {
                    let page = section * 4 + index;
                    submenu = submenu.child(
                        SidebarItem::new(("docs.page", page), *title)
                            .h(s * 7_f32)
                            .selected(selected == page)
                            .on_activate(cx.listener(move |this, _, _, cx| {
                                this.sidebar_selected = page;
                                this.sidebar_state.mobile_open = false;
                                cx.notify();
                            })),
                    );
                }
                group = group.child(submenu);
            }
            navigation = navigation.child(group);
        }
        if count == 0 {
            navigation = navigation.child(
                div()
                    .p(s * 2_f32)
                    .text_color(t.colors.muted_foreground)
                    .child("No matching pages"),
            );
        }
        let description = match selected {
            1 => {
                "Install the components you need. Their Rust source stays in your app, ready to edit."
            }
            5 => {
                "One theme controls colors, spacing, type, corners, and motion across your application."
            }
            7 => {
                "Use named controls, visible focus, and the native keyboard behavior supplied by GPUI Kit."
            }
            _ => "Build native interfaces with components that share a consistent visual language.",
        };
        let content = div().size_full().flex().flex_col()
            .child(div().h(s * 14_f32).px(s * 3_f32).flex_shrink_0().flex_row_in(t.direction).items_center().gap(s * 3_f32)
                .border_b_1().border_color(t.colors.border)
                .child(sidebar_trigger("docs.toggle", cx.listener(move |this, _, _, cx| { this.sidebar_state.toggle(mobile); cx.notify(); }), cx))
                .child(div().text_size(px(12.) * t.text_scale).line_height(px(16.) * t.text_scale).text_color(t.colors.muted_foreground).child(groups[selected / 4].0)))
            .child(div().id("docs.article").flex_1().min_h_0().min_w_0().overflow_y_scroll().p(s * 6_f32)
                .child(div().text_size(px(20.) * t.text_scale).line_height(px(28.) * t.text_scale).font_weight(FontWeight::MEDIUM).child(title))
                .child(div().mt(s * 3_f32).text_size(px(14.) * t.text_scale).text_color(t.colors.muted_foreground).child(description))
                .child(div().mt(s * 6_f32).p(s * 4_f32).rounded(t.radius.lg).bg(t.colors.muted)
                    .font_family(t.fonts.mono).text_size(px(12.) * t.text_scale).line_height(px(16.) * t.text_scale).child("gpuicn add sidebar"))
                .child(div().mt(s * 6_f32).font_weight(FontWeight::MEDIUM).child("In this guide"))
                .child(div().mt(s * 3_f32).text_size(px(14.) * t.text_scale).text_color(t.colors.muted_foreground)
                    .child("Browse the sections on the left, search for a page, or hide navigation to give the article more room."))
                .child(div().mt(s * 6_f32).flex().justify_between().gap(s * 2_f32)
                    .child(Button::new("docs.previous").variant(ButtonVariant::Outline).disabled(selected == 0).label("Previous")
                        .on_click(cx.listener(|this, _, _, cx| { this.sidebar_selected = this.sidebar_selected.saturating_sub(1); cx.notify(); })))
                    .child(Button::new("docs.next").variant(ButtonVariant::Outline).disabled(selected == 11).label("Next")
                        .on_click(cx.listener(|this, _, _, cx| { this.sidebar_selected = (this.sidebar_selected + 1).min(11); cx.notify(); })))));
        SidebarLayout::new("docs.layout", self.sidebar_state)
            .mobile(mobile)
            .sidebar(navigation)
            .content(content)
            .on_change(cx.listener(|this, state: &SidebarState, _, cx| {
                this.sidebar_state = *state;
                cx.notify();
            }))
            .collapsible(SidebarCollapsible::Offcanvas)
            .rail(true)
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.sidebar_preview(window, cx).into_any_element()
}
