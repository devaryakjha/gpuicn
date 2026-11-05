//! The sortable catalog preview.

use crate::*;

/// The demo's channel categories and their channels, in the current order.
struct ChannelOrder {
    names: Vec<&'static str>,
    groups: Vec<Vec<&'static str>>,
}

impl Showcase {
    pub(crate) fn sortable_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let order = window.use_keyed_state("sortable.order", cx, |_, _| ChannelOrder {
            names: vec!["Favorites", "Channels", "Direct messages"],
            groups: vec![
                vec!["town-square", "design"],
                vec!["engineering", "releases", "random"],
                vec![],
            ],
        });
        let moved = order.clone();
        let regrouped = order.clone();
        let bookmarks = window.use_keyed_state("sortable.bookmarks", cx, |_, _| {
            vec![vec!["Roadmap", "Design specs", "On-call", "Runbook"]]
        });
        let rebooked = bookmarks.clone();
        let bar = Sortable::new("sortable.bookmarks", "Bookmarks")
            .horizontal()
            .group(
                SortableGroup::new("sortable.bookmarks.bar", "Bookmarks")
                    .header(|_, _| div())
                    .rows(bookmarks.read(cx)[0].iter().map(|&name| {
                        SortableRow::new(name, name, move |_, _| {
                            Button::new(gpui_kit::ElementId::from(format!("sortable.bookmark.{name}")))
                                .size(ButtonSize::Sm)
                                .variant(ButtonVariant::Secondary)
                                .child(lucide(LucideIcon::Bookmark).size(px(14.)))
                                .label(name)
                        })
                    })),
            )
            .on_reorder(move |movement, _, cx| {
                rebooked.update(cx, |items, cx| {
                    movement.apply(items);
                    cx.notify();
                })
            });
        let current = order.read(cx);
        let theme = UiTheme::read(cx);
        let sidebar = div()
            .w(px(260.))
            .p(px(8.))
            .rounded(theme.radius.lg)
            .border_1()
            .border_color(theme.colors.sidebar_border)
            .bg(theme.colors.sidebar)
            .child(
            Sortable::new("sortable.channels", "Channels")
                .reorder_groups(true)
                .groups(current.names.iter().zip(&current.groups).map(|(&name, rows)| {
                    SortableGroup::new(name, name).rows(rows.iter().map(|&channel| {
                        SortableRow::new(channel, channel, move |_, cx| {
                            let color = UiTheme::read(cx).colors.sidebar_foreground;
                            gpuicn::sidebar::SidebarItem::new(
                                gpui_kit::ElementId::from(format!("sortable.{channel}")),
                                channel,
                            )
                            .icon(lucide(LucideIcon::Hash).size(px(16.)).text_color(color))
                        })
                    }))
                }))
                .on_reorder(move |movement, _, cx| {
                    moved.update(cx, |order, cx| {
                        movement.apply(&mut order.groups);
                        cx.notify();
                    })
                })
                .on_reorder_group(move |from, to, _, cx| {
                    regrouped.update(cx, |order, cx| {
                        gpuicn::sortable::move_group(&mut order.groups, from, to);
                        gpuicn::sortable::move_group(&mut order.names, from, to);
                        cx.notify();
                    })
                }),
        );
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(16.))
            .child(div().max_w(px(460.)).child(bar))
            .child(sidebar)
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.sortable_preview(window, cx).into_any_element()
}
