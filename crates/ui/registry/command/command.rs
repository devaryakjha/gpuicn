//! Nova command palette: a search field over grouped, filterable commands.
//!
//! Visual source: shadcn/ui 4.19.0 `command.tsx`, Nova style. The list is
//! virtualized, so a quick switcher can hold thousands of channels.

use std::rc::Rc;

use gpui_icons::{LucideIcon, lucide};
use gpui_kit::base::actions::{Cancel, Confirm, SelectDown, SelectUp};
use gpui_kit::{
    AnyElement, App, AppContext as _, Context, ElementId, Entity, EventEmitter, FocusHandle,
    Focusable, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _, Render,
    RenderOnce, Role, ScrollStrategy, SharedString, StatefulInteractiveElement as _,
    StyleRefinement, Styled, Subscription, UniformListScrollHandle, Window, div,
    prelude::FluentBuilder as _, px, uniform_list,
};

use super::input::{Input, InputEvent, InputState};
use super::kbd::format_shortcut;
use super::theme::{UiTheme, apply_style};

/// One command or destination in the palette.
#[derive(Clone, Debug)]
pub struct CommandItem {
    /// The stable value reported when the item is confirmed.
    pub value: SharedString,
    /// The visible label, which the query matches.
    pub label: SharedString,
    /// The heading this item is listed under.
    pub group: Option<SharedString>,
    /// An optional leading Lucide icon.
    pub icon: Option<LucideIcon>,
    /// The key binding shown at the trailing edge, such as `"secondary-k"`.
    pub shortcut: Option<SharedString>,
    /// Extra words the query matches, such as aliases.
    pub keywords: Vec<SharedString>,
    /// Whether the item is shown but cannot be chosen.
    pub disabled: bool,
}

impl CommandItem {
    /// Creates an enabled command from its stable value and visible label.
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            group: None,
            icon: None,
            shortcut: None,
            keywords: Vec::new(),
            disabled: false,
        }
    }
    /// Lists the item under a heading. Items with the same heading are shown together.
    pub fn group(mut self, group: impl Into<SharedString>) -> Self {
        self.group = Some(group.into());
        self
    }
    /// Adds a leading Lucide icon.
    pub fn icon(mut self, icon: LucideIcon) -> Self {
        self.icon = Some(icon);
        self
    }
    /// Shows a key binding such as `"secondary-shift-p"` with the platform's
    /// modifier names.
    pub fn shortcut(mut self, source: impl Into<SharedString>) -> Self {
        self.shortcut = Some(source.into());
        self
    }
    /// Adds words the query should also match.
    pub fn keywords(mut self, keywords: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.keywords = keywords.into_iter().map(Into::into).collect();
        self
    }
    /// Shows the item without letting it be chosen.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Notifications from a command palette.
#[derive(Clone, Debug, PartialEq)]
pub enum CommandEvent {
    /// The query text changed.
    Query(SharedString),
    /// An item was chosen with Enter or a click.
    Confirm(SharedString),
    /// Escape was pressed with an empty query. A hosting dialog closes itself.
    Cancel,
}

type ItemSlot = Rc<dyn Fn(&CommandItem, &mut Window, &mut App) -> Option<AnyElement> + 'static>;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Row {
    Heading(usize),
    Item(usize),
}

/// Own the palette's items, query and highlight; subscribe to `CommandEvent`.
pub struct CommandState {
    items: Vec<CommandItem>,
    input: Entity<InputState>,
    rows: Vec<Row>,
    highlighted: Option<usize>,
    filter: bool,
    scroll: UniformListScrollHandle,
    leading: Option<ItemSlot>,
    trailing: Option<ItemSlot>,
    _subscription: Subscription,
}

impl CommandState {
    /// Creates palette state and its search editor in the current window.
    pub fn new(
        items: impl IntoIterator<Item = CommandItem>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Type a command or search…"));
        let subscription = cx.subscribe(&input, |this, input, event, cx| {
            if let InputEvent::Change = event {
                this.refilter(cx);
                cx.emit(CommandEvent::Query(input.read(cx).value()));
            }
        });
        let mut state = Self {
            items: items.into_iter().collect(),
            input,
            rows: Vec::new(),
            highlighted: None,
            filter: true,
            scroll: UniformListScrollHandle::new(),
            leading: None,
            trailing: None,
            _subscription: subscription,
        };
        state.refilter(cx);
        state
    }

    /// Renders the leading element of each item, such as an avatar image or a
    /// channel glyph. Return `None` to fall back to the item's Lucide icon.
    /// Keep the element 16 to 20 px tall so rows stay one height.
    pub fn set_leading(
        &mut self,
        slot: impl Fn(&CommandItem, &mut Window, &mut App) -> Option<AnyElement> + 'static,
        cx: &mut Context<Self>,
    ) {
        self.leading = Some(Rc::new(slot));
        cx.notify();
    }

    /// Renders an element at the trailing edge of each item, before its
    /// shortcut, such as an unread or mention count `Badge`.
    pub fn set_trailing(
        &mut self,
        slot: impl Fn(&CommandItem, &mut Window, &mut App) -> Option<AnyElement> + 'static,
        cx: &mut Context<Self>,
    ) {
        self.trailing = Some(Rc::new(slot));
        cx.notify();
    }

    /// Replaces the items, keeping the query. Use this to show results from an
    /// asynchronous search.
    pub fn set_items(
        &mut self,
        items: impl IntoIterator<Item = CommandItem>,
        cx: &mut Context<Self>,
    ) {
        self.items = items.into_iter().collect();
        self.refilter(cx);
    }

    /// Turns local filtering on or off. Turn it off when the application filters
    /// and ranks items itself in response to `CommandEvent::Query`.
    pub fn set_filter(&mut self, filter: bool, cx: &mut Context<Self>) {
        self.filter = filter;
        self.refilter(cx);
    }

    /// Sets the search field's placeholder.
    pub fn set_placeholder(
        &mut self,
        placeholder: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let placeholder = placeholder.into();
        self.input.update(cx, |input, cx| {
            input.set_placeholder(placeholder, window, cx)
        });
    }

    /// Returns the current query.
    pub fn query(&self, cx: &App) -> SharedString {
        self.input.read(cx).value()
    }

    /// Replaces the query and filters the items again.
    pub fn set_query(
        &mut self,
        query: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let query = query.into();
        self.input
            .update(cx, |input, cx| input.set_value(query, window, cx));
        self.refilter(cx);
    }

    /// Returns the highlighted item, which Enter confirms.
    pub fn highlighted(&self) -> Option<&CommandItem> {
        self.highlighted.map(|index| &self.items[index])
    }

    /// Returns how many items match the query.
    pub fn matched_count(&self) -> usize {
        self.rows
            .iter()
            .filter(|row| matches!(row, Row::Item(_)))
            .count()
    }

    /// Moves keyboard focus to the search field.
    pub fn focus(&self, window: &mut Window, cx: &mut App) {
        self.input.read(cx).focus_handle(cx).focus(window, cx);
    }

    fn refilter(&mut self, cx: &mut Context<Self>) {
        let query = self.input.read(cx).value().to_lowercase();
        let mut groups: Vec<(Option<SharedString>, Vec<usize>)> = Vec::new();
        for (index, item) in self.items.iter().enumerate() {
            if self.filter && !matches_query(item, &query) {
                continue;
            }
            match groups.iter_mut().find(|(group, _)| *group == item.group) {
                Some((_, members)) => members.push(index),
                None => groups.push((item.group.clone(), vec![index])),
            }
        }
        // Ungrouped items come first, as in cmdk.
        groups.sort_by_key(|(group, _)| group.is_some());
        self.rows = groups
            .into_iter()
            .flat_map(|(group, members)| {
                let heading = group.is_some().then(|| Row::Heading(members[0]));
                heading
                    .into_iter()
                    .chain(members.into_iter().map(Row::Item))
            })
            .collect();
        let first = self.enabled_items().next();
        self.highlighted = first;
        self.scroll.scroll_to_item(0, ScrollStrategy::Top);
        cx.notify();
    }

    fn enabled_items(&self) -> impl Iterator<Item = usize> + '_ {
        self.rows.iter().filter_map(|row| match row {
            Row::Item(index) if !self.items[*index].disabled => Some(*index),
            _ => None,
        })
    }

    fn navigate(&mut self, direction: isize, cx: &mut Context<Self>) {
        let enabled: Vec<_> = self.enabled_items().collect();
        if enabled.is_empty() {
            return;
        }
        let current = enabled
            .iter()
            .position(|index| Some(*index) == self.highlighted);
        let next = match (current, direction) {
            (None, _) => 0,
            (Some(current), direction) => {
                (current as isize + direction).rem_euclid(enabled.len() as isize) as usize
            }
        };
        self.highlight(enabled[next], cx);
    }

    fn highlight(&mut self, index: usize, cx: &mut Context<Self>) {
        self.highlighted = Some(index);
        if let Some(row) = self.rows.iter().position(|row| *row == Row::Item(index)) {
            // Keep the group heading in view above its first item.
            let row = if row > 0 && matches!(self.rows[row - 1], Row::Heading(_)) {
                row - 1
            } else {
                row
            };
            self.scroll.scroll_to_item(row, ScrollStrategy::Nearest);
        }
        cx.notify();
    }

    fn confirm(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.items[index].disabled {
            return;
        }
        self.highlighted = Some(index);
        cx.emit(CommandEvent::Confirm(self.items[index].value.clone()));
        cx.notify();
    }
}

/// Case-insensitive match against the label or keywords: a substring, or the
/// query's characters in order, so `"gnrl"` finds "general".
fn matches_query(item: &CommandItem, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let subsequence = |text: &str| {
        let mut text = text.chars();
        query.chars().all(|wanted| text.any(|c| c == wanted))
    };
    std::iter::once(&item.label)
        .chain(&item.keywords)
        .map(|text| text.to_lowercase())
        .any(|text| text.contains(query) || subsequence(&text))
}

impl EventEmitter<CommandEvent> for CommandState {}

impl Focusable for CommandState {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.input.read(cx).focus_handle(cx)
    }
}

/// Row height shared by headings and items, so the list can be virtualized.
const ROW: f32 = 8.;

impl Render for CommandState {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let id = ElementId::from(("command", cx.entity_id()));
        let os = cx.platform_info().os;
        let rows = self.rows.clone();
        let total = rows.len();
        let highlighted = self.highlighted;
        let row_theme = theme.clone();
        let list_id = id.clone();
        let leading_slot = self.leading.clone();
        let trailing_slot = self.trailing.clone();
        let rows_list = uniform_list(
            (id.clone(), "rows"),
            total,
            cx.processor(move |this, range: std::ops::Range<usize>, window, cx| {
                range
                    .map(|row| match rows[row] {
                        Row::Heading(index) => div()
                            .id((list_id.clone(), SharedString::from(row.to_string())))
                            .role(Role::Heading)
                            .w_full()
                            .h(row_theme.space(ROW))
                            .px(row_theme.space(2.))
                            .pt(row_theme.space(2.5))
                            .text_size(row_theme.text(12.))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(row_theme.colors.muted_foreground)
                            .child(this.items[index].group.clone().unwrap_or_default())
                            .into_any_element(),
                        Row::Item(index) => {
                            let item = &this.items[index];
                            let selected = highlighted == Some(index);
                            let leading = leading_slot
                                .as_ref()
                                .and_then(|slot| slot(item, window, cx));
                            let trailing = trailing_slot
                                .as_ref()
                                .and_then(|slot| slot(item, window, cx));
                            div()
                                .id((list_id.clone(), SharedString::from(row.to_string())))
                                .role(Role::ListBoxOption)
                                .aria_label(item.label.clone())
                                .aria_selected(selected)
                                .when(selected, |row| row.aria_active_descendant())
                                .when(item.disabled, |row| {
                                    row.aria_description("Unavailable")
                                        .opacity(0.5)
                                        .cursor_not_allowed()
                                })
                                .w_full()
                                .h(row_theme.space(ROW))
                                .flex()
                                .items_center()
                                .gap(row_theme.space(2.))
                                .px(row_theme.space(2.))
                                .rounded(row_theme.radius.md)
                                .text_size(row_theme.text(14.))
                                .whitespace_nowrap()
                                .overflow_hidden()
                                .when(selected, |row| {
                                    row.bg(row_theme.colors.muted)
                                        .text_color(row_theme.colors.foreground)
                                })
                                .when(!item.disabled, |row| {
                                    row.cursor_pointer()
                                        .on_hover(cx.listener(move |this, hovered, _, cx| {
                                            if *hovered && this.highlighted != Some(index) {
                                                this.highlighted = Some(index);
                                                cx.notify();
                                            }
                                        }))
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.confirm(index, cx)
                                        }))
                                })
                                .map(|row| match (leading, item.icon) {
                                    (Some(leading), _) => row.child(
                                        div().flex_shrink_0().flex().items_center().child(leading),
                                    ),
                                    (None, Some(icon)) => row.child(
                                        lucide(icon)
                                            .size(row_theme.space(4.))
                                            .flex_shrink_0()
                                            .text_color(row_theme.colors.muted_foreground),
                                    ),
                                    (None, None) => row,
                                })
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .overflow_hidden()
                                        .text_ellipsis()
                                        .child(item.label.clone()),
                                )
                                .when_some(trailing, |row, trailing| {
                                    row.child(
                                        div().flex_shrink_0().flex().items_center().child(trailing),
                                    )
                                })
                                .when_some(item.shortcut.clone(), |row, shortcut| {
                                    row.child(
                                        div()
                                            .flex_shrink_0()
                                            .text_size(row_theme.text(12.))
                                            .text_color(row_theme.colors.muted_foreground)
                                            .child(format_shortcut(&shortcut, os)),
                                    )
                                })
                                .into_any_element()
                        }
                    })
                    .collect()
            }),
        )
        .track_scroll(&self.scroll)
        .w_full()
        .h(theme.space(ROW) * total.min(9) as f32);

        div()
            .id(id.clone())
            .key_context("Combobox")
            .w_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(colors.popover)
            .text_color(colors.popover_foreground)
            .font_family(theme.fonts.body.clone())
            .text_size(theme.text(14.))
            .capture_action(cx.listener(|this, _: &SelectUp, _, cx| {
                this.navigate(-1, cx);
                cx.stop_propagation();
            }))
            .capture_action(cx.listener(|this, _: &SelectDown, _, cx| {
                this.navigate(1, cx);
                cx.stop_propagation();
            }))
            .on_action(cx.listener(|this, _: &Confirm, _, cx| {
                if let Some(index) = this.highlighted {
                    this.confirm(index, cx);
                }
                cx.stop_propagation();
            }))
            .on_action(cx.listener(|this, _: &Cancel, window, cx| {
                if this.query(cx).is_empty() {
                    cx.emit(CommandEvent::Cancel);
                    // A hosting dialog closes on the same Escape.
                    cx.propagate();
                } else {
                    this.set_query("", window, cx);
                    cx.stop_propagation();
                }
            }))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(theme.space(2.))
                    .h(theme.space(9.))
                    .px(theme.space(3.))
                    .border_b_1()
                    .border_color(colors.border)
                    .child(
                        lucide(LucideIcon::Search)
                            .size(theme.space(4.))
                            .flex_shrink_0()
                            .text_color(colors.muted_foreground),
                    )
                    .child(
                        Input::new(&self.input)
                            .aria_label("Search commands")
                            .bordered(false)
                            .px_0(),
                    ),
            )
            .child(
                div()
                    .id((id, "list"))
                    .role(Role::ListBox)
                    .aria_label("Commands")
                    .p(theme.space(1.))
                    .when(total == 0, |list| {
                        list.child(
                            div()
                                .py(theme.space(6.))
                                .text_center()
                                .text_size(theme.text(14.))
                                .text_color(colors.muted_foreground)
                                .child("No results found."),
                        )
                    })
                    .when(total > 0, |list| list.child(rows_list)),
            )
    }
}

/// A command palette bound to retained `CommandState`.
///
/// Use it inline, or inside a dialog for a Cmd/Ctrl+K quick switcher.
#[derive(IntoElement)]
pub struct Command {
    state: Entity<CommandState>,
    bordered: bool,
    style: StyleRefinement,
}

impl Command {
    /// Creates a palette for retained palette state.
    pub fn new(state: &Entity<CommandState>) -> Self {
        Self {
            state: state.clone(),
            bordered: true,
            style: StyleRefinement::default(),
        }
    }
    /// Draws the rounded border, on by default. Turn it off inside a dialog surface.
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }
}

impl Styled for Command {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Command {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx);
        let root = div()
            .w_full()
            .overflow_hidden()
            .rounded(theme.radius.xl)
            .when(self.bordered, |root| {
                root.border_1().border_color(theme.colors.border)
            })
            .child(self.state);
        apply_style(root, &self.style)
    }
}

/// The popup surface for a command dialog: pass it to `dialog` with the handle
/// that opens the palette. Close the dialog on `CommandEvent::Confirm`.
pub fn command_dialog_popup(
    id: impl Into<ElementId>,
    state: &Entity<CommandState>,
    cx: &App,
) -> gpui_kit::Stateful<gpui_kit::Div> {
    let theme = UiTheme::read(cx);
    div()
        .id(id)
        .aria_label("Command palette")
        .occlude()
        .w_full()
        .min_w(px(0.))
        .max_w(theme.space(120.))
        .overflow_hidden()
        .rounded(theme.radius.xl)
        .border_1()
        .border_color(theme.colors.foreground.opacity(0.1))
        .bg(theme.colors.popover)
        .shadow(theme.shadows.lg.clone())
        .child(Command::new(state).bordered(false))
}

#[cfg(test)]
mod tests {
    use super::{CommandItem, matches_query};

    #[test]
    fn query_matches_substrings_subsequences_and_keywords() {
        let item = CommandItem::new("general", "General").keywords(["town-square"]);
        assert!(matches_query(&item, ""));
        assert!(matches_query(&item, "ner"));
        assert!(matches_query(&item, "gnrl"));
        assert!(matches_query(&item, "square"));
        assert!(!matches_query(&item, "random"));
    }
}
