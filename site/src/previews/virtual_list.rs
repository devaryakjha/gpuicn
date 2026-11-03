//! The virtual list catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn virtual_list_preview(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        use gpuicn::virtual_list::{ListItem, VirtualList, VirtualListEvent};
        let t = UiTheme::read(cx).clone();
        let s = t.spacing.unit;
        let rows = self
            .virtual_rows
            .as_ref()
            .expect("list preview state")
            .clone();
        let details = self.virtual_details;
        let current = rows
            .focused()
            .and_then(|id| rows.index_of(&id))
            .and_then(|index| rows.item(index));
        let view = cx.entity().downgrade();
        let menu_rows = rows.clone();
        let menu_theme = t.clone();
        let list = VirtualList::new(
            "files.list",
            "Source files",
            rows.clone(),
            move |row, window, cx| {
                let state = menu_rows.clone();
                let view = view.clone();
                let target = row.item.id.clone();
                let content = div()
                    .w_full()
                    .min_w_0()
                    .flex()
                    .items_center()
                    .gap(s * 2_f32)
                    .when(details, |el| el.py(s))
                    .child(
                        lucide(LucideIcon::FileCode)
                            .size(s * 4_f32)
                            .text_color(menu_theme.colors.muted_foreground),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .child(div().truncate().child(row.item.label.clone()))
                            .when(details, |el| {
                                el.child(
                                    div()
                                        .text_size(px(12.) * menu_theme.text_scale)
                                        .line_height(px(16.) * menu_theme.text_scale)
                                        .text_color(menu_theme.colors.muted_foreground)
                                        .child("src/components · Updated a moment ago"),
                                )
                            }),
                    )
                    .child(
                        div()
                            .text_size(px(12.) * menu_theme.text_scale)
                            .text_color(menu_theme.colors.muted_foreground)
                            .child(if row.item.disabled { "Locked" } else { "M" }),
                    );
                let menu =
                    window
                        .use_keyed_state((row.item.id.clone(), "menu"), cx, |_, cx| {
                            cx.new(|cx| {
                                MenuState::new(
                                    [MenuItem::new("inspect-selection", "Inspect selection")
                                        .on_click(move |_, _, cx| {
                                            if let Some(item) = state
                                                .index_of(&target)
                                                .and_then(|index| state.item(index))
                                            {
                                                let count = state.selected_count();
                                                let _ = view.update(cx, |this, cx| {
                                                    this.virtual_note = format!(
                                                        "Inspect {} · {count} selected",
                                                        item.label
                                                    );
                                                    cx.notify();
                                                });
                                            }
                                        })],
                                    cx,
                                )
                            })
                        })
                        .read(cx)
                        .clone();
                context_menu(&menu, "File actions", content)
                    .w_full()
                    .disabled(row.item.disabled)
                    .into_any_element()
            },
        )
        .when(details, |list| list.row_height(s * 12_f32))
        .on_event(cx.listener(|this, event, _, cx| {
            if let VirtualListEvent::Activate(id) = event
                && let Some(state) = &this.virtual_rows
                && let Some(item) = state.index_of(id).and_then(|index| state.item(index))
            {
                this.virtual_note = format!("Opened {}", item.label);
            }
            cx.notify();
        }));
        div().size_full().min_w_0().min_h_0().flex().flex_col()
            .font_family(t.fonts.body).text_size(px(14.) * t.text_scale).line_height(px(20.) * t.text_scale)
            .child(div().px(s * 4_f32).py(s * 3_f32).flex().items_center().justify_between().gap(s * 2_f32)
                .border_b_1().border_color(t.colors.border)
                .child(div().flex().flex_col().child(div().font_weight(gpui_kit::FontWeight::MEDIUM).child("Source files"))
                    .child(div().text_size(px(12.) * t.text_scale).text_color(t.colors.muted_foreground)
                        .child(format!("{} rows · Multi-select", rows.len()))))
                .child(Button::new("files.details").aria_label("Toggle row details").variant(if details { ButtonVariant::Secondary } else { ButtonVariant::Outline })
                    .size(ButtonSize::Sm).label("Details")
                    .on_click(cx.listener(|this, _, _, cx| { this.virtual_details = !this.virtual_details; cx.notify(); }))))
            .child(div().px(s * 3_f32).py(s * 2_f32).flex().flex_wrap().gap(s * 2_f32)
                .border_b_1().border_color(t.colors.border)
                .child(Button::new("files.reveal").aria_label("Jump to row 50,000").variant(ButtonVariant::Outline).size(ButtonSize::Sm).label("Jump to 50,000")
                    .on_click(cx.listener(|this, _, _, cx| {
                        if let Some(rows) = &this.virtual_rows { rows.reveal(&("file", 49_999usize).into()); }
                        cx.notify();
                    })))
                .child(Button::new("files.reverse").aria_label("Reverse row order").variant(ButtonVariant::Outline).size(ButtonSize::Sm)
                    .child(if self.virtual_reversed { "Original order" } else { "Reverse order" })
                    .on_click(cx.listener(|this, _, _, cx| {
                        if let Some(rows) = &this.virtual_rows {
                            let items = (0..rows.len()).rev().filter_map(|index| rows.item(index)).collect();
                            rows.replace_items(items).expect("reorder retains unique IDs");
                            this.virtual_reversed = !this.virtual_reversed;
                        }
                        cx.notify();
                    })))
                .child(Button::new("files.refresh").aria_label("Refresh rows").variant(ButtonVariant::Outline).size(ButtonSize::Sm).label("Refresh rows")
                    .on_click(cx.listener(|this, _, _, cx| {
                        if let Some(rows) = &this.virtual_rows {
                            let added = gpui_kit::ElementId::from("new-file");
                            let existed = rows.index_of(&added).is_some();
                            let mut items: Vec<_> = (0..rows.len()).filter_map(|index| rows.item(index)).filter(|item| item.id != added).collect();
                            if !existed { items.insert(0, ListItem::new("new-file", "new_component.rs")); }
                            rows.replace_items(items).expect("refresh keeps unique IDs");
                            this.virtual_note = if existed { "Removed the added row; surviving selection and viewport retained." } else { "Added a row at the top; surviving selection and viewport retained." }.into();
                        }
                        cx.notify();
                    }))))
            .child(div().flex_1().min_h_0().min_w_0().px(s * 2_f32).py(s).child(list))
            .child(div().px(s * 4_f32).py(s * 3_f32).border_t_1().border_color(t.colors.border).flex().flex_col().gap(s)
                .child(div().flex().justify_between().gap(s * 2_f32)
                    .child(format!("{} selected", rows.selected_count()))
                    .child(div().min_w_0().truncate().text_color(t.colors.muted_foreground)
                        .child(current.map(|item| item.label).unwrap_or_else(|| "Use arrows, Shift and Cmd/Ctrl".into()))))
                .when(!self.virtual_note.is_empty(), |el| el.child(div().text_size(px(12.) * t.text_scale)
                    .line_height(px(16.) * t.text_scale).text_color(t.colors.muted_foreground).child(self.virtual_note.clone()))))
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.virtual_list_preview(window, cx).into_any_element()
}
