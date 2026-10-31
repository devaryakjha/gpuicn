//! Nova emoji picker: search, category tabs, a virtualized grid, recents and
//! custom emoji.
//!
//! The application owns the emoji set (names, glyphs, keywords and
//! categories), the recently used list and custom emoji images; gpuicn never
//! loads emoji data or fetches images itself.

use std::ops::Range;

use gpui_icons::{LucideIcon, lucide};
use gpui_kit::{
    AnyElement, App, AppContext as _, Context, ElementId, Entity, EventEmitter, FocusHandle,
    Focusable, FontWeight, ImageSource, InteractiveElement as _, IntoElement, Keystroke, ObjectFit,
    ParentElement as _, Render, RenderOnce, Role, ScrollStrategy, SharedString,
    StatefulInteractiveElement as _, StyleRefinement, Styled, StyledImage as _, Subscription,
    UniformListScrollHandle, Window, div, img, prelude::FluentBuilder as _, uniform_list,
};

use super::input::{Input, InputEvent, InputState};
use super::theme::{UiTheme, apply_style};

/// Emoji per grid row.
pub const COLUMNS: usize = 9;

/// One emoji: a Unicode glyph or a custom image, named by its shortcode.
#[derive(Clone)]
pub struct Emoji {
    /// The shortcode without colons, such as `"tada"`; reported when picked.
    pub name: SharedString,
    /// The Unicode text, for standard emoji.
    pub glyph: Option<SharedString>,
    /// The image, for custom emoji.
    pub image: Option<ImageSource>,
    /// Extra words search matches, such as aliases.
    pub keywords: Vec<SharedString>,
}

impl Emoji {
    /// A standard emoji drawn from its Unicode text.
    pub fn new(name: impl Into<SharedString>, glyph: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            glyph: Some(glyph.into()),
            image: None,
            keywords: Vec::new(),
        }
    }
    /// A custom emoji drawn from an image the application supplies.
    pub fn custom(name: impl Into<SharedString>, image: impl Into<ImageSource>) -> Self {
        Self {
            name: name.into(),
            glyph: None,
            image: Some(image.into()),
            keywords: Vec::new(),
        }
    }
    /// Adds words search should also match.
    pub fn keywords(mut self, keywords: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.keywords = keywords.into_iter().map(Into::into).collect();
        self
    }
}

/// A titled group of emoji with a tab icon.
#[derive(Clone)]
pub struct EmojiCategory {
    /// The heading, such as "Smileys & Emotion".
    pub name: SharedString,
    /// The tab icon.
    pub icon: LucideIcon,
    /// The emoji, in display order.
    pub emoji: Vec<Emoji>,
}

impl EmojiCategory {
    /// Creates a category.
    pub fn new(
        name: impl Into<SharedString>,
        icon: LucideIcon,
        emoji: impl IntoIterator<Item = Emoji>,
    ) -> Self {
        Self {
            name: name.into(),
            icon,
            emoji: emoji.into_iter().collect(),
        }
    }
}

/// Notifications from an emoji picker.
#[derive(Clone, Debug, PartialEq)]
pub enum EmojiPickerEvent {
    /// An emoji was chosen with Enter or a click: its shortcode without colons.
    /// The application records it as recently used and closes a hosting popover.
    Pick(SharedString),
    /// Escape was pressed with an empty query.
    Cancel,
}

/// A grid row: a section heading or up to `COLUMNS` emoji.
#[derive(Clone, Debug, PartialEq)]
enum Row {
    Heading(usize),
    Emoji { section: usize, cells: Range<usize> },
}

#[derive(Clone, Debug)]
struct Section {
    name: SharedString,
    icon: LucideIcon,
    /// Emoji as (category, index) into the categories, so recents and search
    /// results point at the same data.
    emoji: Vec<(usize, usize)>,
    tab: bool,
}

/// Retained emoji picker state; subscribe to `EmojiPickerEvent`.
pub struct EmojiPickerState {
    categories: Vec<EmojiCategory>,
    recents: Vec<SharedString>,
    input: Entity<InputState>,
    min_query: usize,
    sections: Vec<Section>,
    rows: Vec<Row>,
    /// (section, index in section)
    highlighted: Option<(usize, usize)>,
    scroll: UniformListScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<EmojiPickerEvent> for EmojiPickerState {}

impl Focusable for EmojiPickerState {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.input.read(cx).focus_handle(cx)
    }
}

impl EmojiPickerState {
    /// Creates a picker over the application's emoji categories.
    pub fn new(
        categories: impl IntoIterator<Item = EmojiCategory>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let input = cx.new(|cx| InputState::new(window, cx).placeholder("Search emoji"));
        let window_handle = window.window_handle();
        let weak = cx.weak_entity();
        let subscriptions = vec![
            cx.subscribe(&input, |this, _, event, cx| {
                if let InputEvent::Change = event {
                    this.rebuild(cx);
                }
            }),
            // Arrow keys move through the grid while the search field has
            // focus, ahead of the field's own caret bindings.
            cx.intercept_keystrokes(move |event, window, cx| {
                if window.window_handle() != window_handle {
                    return;
                }
                let Some(state) = weak.upgrade() else {
                    return;
                };
                if !state.read(cx).focus_handle(cx).is_focused(window) {
                    return;
                }
                if state.update(cx, |state, cx| {
                    state.handle_key(&event.keystroke, window, cx)
                }) {
                    cx.stop_propagation();
                }
            }),
        ];
        let mut state = Self {
            categories: categories.into_iter().collect(),
            recents: Vec::new(),
            input,
            min_query: 2,
            sections: Vec::new(),
            rows: Vec::new(),
            highlighted: None,
            scroll: UniformListScrollHandle::new(),
            _subscriptions: subscriptions,
        };
        state.rebuild(cx);
        state
    }

    /// Replaces the categories, for example when custom emoji load.
    pub fn set_categories(
        &mut self,
        categories: impl IntoIterator<Item = EmojiCategory>,
        cx: &mut Context<Self>,
    ) {
        self.categories = categories.into_iter().collect();
        self.rebuild(cx);
    }

    /// Adds or replaces the "Custom" category of application-supplied images.
    pub fn set_custom(
        &mut self,
        custom: impl IntoIterator<Item = (SharedString, ImageSource)>,
        cx: &mut Context<Self>,
    ) {
        self.categories.retain(|category| category.name != "Custom");
        let emoji: Vec<_> = custom
            .into_iter()
            .map(|(name, image)| Emoji::custom(name, image))
            .collect();
        if !emoji.is_empty() {
            self.categories
                .push(EmojiCategory::new("Custom", LucideIcon::Sparkles, emoji));
        }
        self.rebuild(cx);
    }

    /// Shows these shortcodes first, as "Recently used". The application owns
    /// the list and updates it after each `EmojiPickerEvent::Pick`.
    pub fn set_recents(
        &mut self,
        recents: impl IntoIterator<Item = impl Into<SharedString>>,
        cx: &mut Context<Self>,
    ) {
        self.recents = recents.into_iter().map(Into::into).collect();
        self.rebuild(cx);
    }

    /// How many characters the query needs before it filters. Defaults to 2;
    /// shorter queries show every category.
    pub fn set_min_query(&mut self, min: usize, cx: &mut Context<Self>) {
        self.min_query = min.max(1);
        self.rebuild(cx);
    }

    /// The search text.
    pub fn query(&self, cx: &App) -> SharedString {
        self.input.read(cx).value()
    }

    /// Replaces the search text.
    pub fn set_query(
        &mut self,
        query: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let query = query.into();
        self.input
            .update(cx, |input, cx| input.set_value(query, window, cx));
        self.rebuild(cx);
    }

    /// The highlighted emoji, which Enter picks.
    pub fn highlighted(&self) -> Option<&Emoji> {
        let (section, index) = self.highlighted?;
        let (category, emoji) = *self.sections.get(section)?.emoji.get(index)?;
        Some(&self.categories[category].emoji[emoji])
    }

    /// How many emoji are listed, counting recents.
    pub fn visible_count(&self) -> usize {
        self.sections
            .iter()
            .map(|section| section.emoji.len())
            .sum()
    }

    /// Moves keyboard focus to the search field.
    pub fn focus(&self, window: &mut Window, cx: &mut App) {
        self.input.read(cx).focus_handle(cx).focus(window, cx);
    }

    fn find(&self, name: &str) -> Option<(usize, usize)> {
        self.categories
            .iter()
            .enumerate()
            .find_map(|(category, group)| {
                group
                    .emoji
                    .iter()
                    .position(|emoji| emoji.name == name)
                    .map(|index| (category, index))
            })
    }

    fn rebuild(&mut self, cx: &mut Context<Self>) {
        let query = self.input.read(cx).value().trim().to_lowercase();
        let searching = query.chars().count() >= self.min_query;
        self.sections = if searching {
            let mut matches: Vec<(bool, (usize, usize))> = Vec::new();
            for (category, group) in self.categories.iter().enumerate() {
                for (index, emoji) in group.emoji.iter().enumerate() {
                    if let Some(prefix) = matches_query(emoji, &query) {
                        matches.push((prefix, (category, index)));
                    }
                }
            }
            // Names that start with the query come first; the sort is stable,
            // so the catalog order holds within each group.
            matches.sort_by_key(|(prefix, _)| !prefix);
            vec![Section {
                name: "Search results".into(),
                icon: LucideIcon::Search,
                emoji: matches.into_iter().map(|(_, at)| at).collect(),
                tab: false,
            }]
        } else {
            let recents: Vec<_> = self
                .recents
                .iter()
                .filter_map(|name| self.find(name))
                .collect();
            let recent = (!recents.is_empty()).then(|| Section {
                name: "Recently used".into(),
                icon: LucideIcon::Clock,
                emoji: recents,
                tab: true,
            });
            recent
                .into_iter()
                .chain(
                    self.categories
                        .iter()
                        .enumerate()
                        .filter(|(_, group)| !group.emoji.is_empty())
                        .map(|(category, group)| Section {
                            name: group.name.clone(),
                            icon: group.icon,
                            emoji: (0..group.emoji.len())
                                .map(|index| (category, index))
                                .collect(),
                            tab: true,
                        }),
                )
                .collect()
        };
        self.rows = grid_rows(
            self.sections.iter().map(|section| section.emoji.len()),
            !searching,
        );
        self.highlighted = self
            .sections
            .iter()
            .position(|section| !section.emoji.is_empty())
            .map(|section| (section, 0));
        self.scroll.scroll_to_item(0, ScrollStrategy::Top);
        cx.notify();
    }

    fn handle_key(
        &mut self,
        keystroke: &Keystroke,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let modifiers = keystroke.modifiers;
        if modifiers.control || modifiers.alt || modifiers.platform || modifiers.shift {
            return false;
        }
        match keystroke.key.as_str() {
            "left" => self.step(-1, cx),
            "right" => self.step(1, cx),
            "up" => self.step_row(-1, cx),
            "down" => self.step_row(1, cx),
            "enter" => {
                if let Some(emoji) = self.highlighted().map(|emoji| emoji.name.clone()) {
                    cx.emit(EmojiPickerEvent::Pick(emoji));
                }
            }
            "escape" => {
                if self.query(cx).is_empty() {
                    cx.emit(EmojiPickerEvent::Cancel);
                    // Let a hosting popover close on the same Escape.
                    return false;
                }
                self.set_query("", window, cx);
            }
            _ => return false,
        }
        true
    }

    /// The highlight as a flat position across sections, for Left and Right.
    fn flat(&self) -> Option<usize> {
        let (section, index) = self.highlighted?;
        Some(
            self.sections[..section]
                .iter()
                .map(|section| section.emoji.len())
                .sum::<usize>()
                + index,
        )
    }

    fn position_at(&self, mut flat: usize) -> Option<(usize, usize)> {
        for (section, group) in self.sections.iter().enumerate() {
            if flat < group.emoji.len() {
                return Some((section, flat));
            }
            flat -= group.emoji.len();
        }
        None
    }

    fn step(&mut self, delta: isize, cx: &mut Context<Self>) {
        let total = self.visible_count();
        if total == 0 {
            return;
        }
        let next = match self.flat() {
            None => 0,
            Some(flat) => (flat as isize + delta).clamp(0, total as isize - 1) as usize,
        };
        let target = self.position_at(next);
        self.highlight(target, cx);
    }

    /// Up and Down keep the column, moving across section boundaries.
    fn step_row(&mut self, delta: isize, cx: &mut Context<Self>) {
        let Some((section, index)) = self.highlighted else {
            self.step(0, cx);
            return;
        };
        let target = step_row(
            &self
                .sections
                .iter()
                .map(|section| section.emoji.len())
                .collect::<Vec<_>>(),
            (section, index),
            delta,
        );
        self.highlight(Some(target), cx);
    }

    fn highlight(&mut self, target: Option<(usize, usize)>, cx: &mut Context<Self>) {
        self.highlighted = target;
        if let Some((section, index)) = target
            && let Some(row) = self.row_of(section, index)
        {
            self.scroll.scroll_to_item(row, ScrollStrategy::Nearest);
        }
        cx.notify();
    }

    fn row_of(&self, section: usize, index: usize) -> Option<usize> {
        self.rows.iter().position(|row| {
            matches!(row, Row::Emoji { section: s, cells } if *s == section && cells.contains(&index))
        })
    }

    fn jump_to(&mut self, section: usize, cx: &mut Context<Self>) {
        let row = self
            .rows
            .iter()
            .position(|row| *row == Row::Heading(section))
            .unwrap_or(0);
        self.scroll.scroll_to_item(row, ScrollStrategy::Top);
        self.highlighted = Some((section, 0));
        cx.notify();
    }
}

/// Lays sections out as rows of up to `COLUMNS` cells, each after its heading.
fn grid_rows(sections: impl IntoIterator<Item = usize>, headings: bool) -> Vec<Row> {
    let mut rows = Vec::new();
    for (section, len) in sections.into_iter().enumerate() {
        if len == 0 {
            continue;
        }
        if headings {
            rows.push(Row::Heading(section));
        }
        let mut start = 0;
        while start < len {
            let end = (start + COLUMNS).min(len);
            rows.push(Row::Emoji {
                section,
                cells: start..end,
            });
            start = end;
        }
    }
    rows
}

/// Moves one grid row up or down from `(section, index)`, keeping the column
/// where the target row is long enough and stopping at the first and last rows.
fn step_row(sections: &[usize], (section, index): (usize, usize), delta: isize) -> (usize, usize) {
    let column = index % COLUMNS;
    let row = index / COLUMNS;
    let rows_in = |section: usize| sections[section].div_ceil(COLUMNS);
    let place = |section: usize, row: usize| {
        let len = sections[section];
        (section, (row * COLUMNS + column).min(len - 1))
    };
    if delta < 0 {
        if row > 0 {
            return place(section, row - 1);
        }
        match (0..section).rev().find(|s| sections[*s] > 0) {
            Some(previous) => place(previous, rows_in(previous) - 1),
            None => (section, index),
        }
    } else {
        if row + 1 < rows_in(section) {
            return place(section, row + 1);
        }
        match (section + 1..sections.len()).find(|s| sections[*s] > 0) {
            Some(next) => place(next, 0),
            None => (section, index),
        }
    }
}

/// Whether the emoji matches, and whether its name starts with the query.
fn matches_query(emoji: &Emoji, query: &str) -> Option<bool> {
    let name = emoji.name.to_lowercase();
    if name.starts_with(query) {
        return Some(true);
    }
    let found = name.contains(query)
        || emoji
            .keywords
            .iter()
            .any(|keyword| keyword.to_lowercase().contains(query));
    found.then_some(false)
}

impl Render for EmojiPickerState {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let id = ElementId::from(("emoji-picker", cx.entity_id()));
        let cell = theme.space(8.);
        let rows = self.rows.clone();
        let total = rows.len();
        let highlighted = self.highlighted;
        let row_theme = theme.clone();
        let list_id = id.clone();
        let grid = uniform_list(
            (id.clone(), "grid"),
            total,
            cx.processor(move |this, range: Range<usize>, _, cx| {
                range
                    .map(|row| match &rows[row] {
                        Row::Heading(section) => div()
                            .id((list_id.clone(), SharedString::from(format!("h{row}"))))
                            .role(Role::Heading)
                            .w_full()
                            .h(cell)
                            .px(row_theme.space(1.))
                            .pt(row_theme.space(3.))
                            .text_size(row_theme.text(12.))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(row_theme.colors.muted_foreground)
                            .child(this.sections[*section].name.clone())
                            .into_any_element(),
                        Row::Emoji { section, cells } => div()
                            .w_full()
                            .h(cell)
                            .flex()
                            .children(cells.clone().map(|index| {
                                let (category, at) = this.sections[*section].emoji[index];
                                let emoji = &this.categories[category].emoji[at];
                                let selected = highlighted == Some((*section, index));
                                let target = (*section, index);
                                let name = emoji.name.clone();
                                div()
                                    .id((
                                        list_id.clone(),
                                        SharedString::from(format!("{section}-{index}")),
                                    ))
                                    .role(Role::ListBoxOption)
                                    .aria_label(format!(":{}:", emoji.name))
                                    .aria_selected(selected)
                                    .when(selected, |cell| cell.aria_active_descendant())
                                    .size(cell)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded(row_theme.radius.md)
                                    .cursor_pointer()
                                    .when(selected, |cell| cell.bg(row_theme.colors.muted))
                                    .on_hover(cx.listener(move |this, hovered, _, cx| {
                                        if *hovered && this.highlighted != Some(target) {
                                            this.highlighted = Some(target);
                                            cx.notify();
                                        }
                                    }))
                                    .on_click(cx.listener(move |_, _, _, cx| {
                                        cx.emit(EmojiPickerEvent::Pick(name.clone()))
                                    }))
                                    .child(emoji_face(emoji, row_theme.space(5.5)))
                            }))
                            .into_any_element(),
                    })
                    .collect()
            }),
        )
        .track_scroll(&self.scroll)
        .w_full()
        .h(cell * 8.);

        let active_tab = highlighted.map(|(section, _)| section);
        let tabs = self
            .sections
            .iter()
            .enumerate()
            .filter(|(_, section)| section.tab)
            .map(|(index, section)| {
                let active = active_tab == Some(index);
                div()
                    .id((id.clone(), SharedString::from(format!("tab-{index}"))))
                    .role(Role::Tab)
                    .aria_label(section.name.clone())
                    .aria_selected(active)
                    .flex_1()
                    .h(theme.space(8.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .border_b_2()
                    .border_color(if active {
                        gpui_kit::Hsla::from(colors.foreground)
                    } else {
                        gpui_kit::transparent_black()
                    })
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| this.jump_to(index, cx)))
                    .child(
                        lucide(section.icon)
                            .size(theme.space(4.))
                            .text_color(if active {
                                colors.foreground
                            } else {
                                colors.muted_foreground
                            }),
                    )
            })
            .collect::<Vec<_>>();

        let preview = self.highlighted().cloned();
        div()
            .id(id.clone())
            .key_context("EmojiPicker")
            .w(cell * COLUMNS as f32 + theme.space(4.))
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(colors.popover)
            .text_color(colors.popover_foreground)
            .font_family(theme.fonts.body.clone())
            .text_size(theme.text(14.))
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
                            .aria_label("Search emoji")
                            .bordered(false)
                            .px_0(),
                    ),
            )
            .when(!tabs.is_empty(), |root| {
                root.child(
                    div()
                        .id((id.clone(), "tabs"))
                        .role(Role::TabList)
                        .aria_label("Emoji categories")
                        .flex()
                        .px(theme.space(2.))
                        .border_b_1()
                        .border_color(colors.border)
                        .children(tabs),
                )
            })
            .child(
                div()
                    .id((id.clone(), "list"))
                    .role(Role::ListBox)
                    .aria_label("Emoji")
                    .px(theme.space(2.))
                    .py(theme.space(1.))
                    .when(total == 0, |list| {
                        list.child(
                            div()
                                .h(cell * 8.)
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_color(colors.muted_foreground)
                                .child("No emoji found."),
                        )
                    })
                    .when(total > 0, |list| list.child(grid)),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(theme.space(2.))
                    .h(theme.space(11.))
                    .px(theme.space(3.))
                    .border_t_1()
                    .border_color(colors.border)
                    .when_some(preview, |footer, emoji| {
                        footer.child(emoji_face(&emoji, theme.space(7.))).child(
                            div()
                                .min_w_0()
                                .overflow_hidden()
                                .text_ellipsis()
                                .whitespace_nowrap()
                                .text_color(colors.muted_foreground)
                                .child(format!(":{}:", emoji.name)),
                        )
                    }),
            )
    }
}

/// Draws an emoji's glyph or image at `size`.
fn emoji_face(emoji: &Emoji, size: gpui_kit::Pixels) -> AnyElement {
    match (&emoji.image, &emoji.glyph) {
        (Some(image), _) => img(image.clone())
            .size(size)
            .object_fit(ObjectFit::Contain)
            .into_any_element(),
        (None, Some(glyph)) => div()
            .text_size(size * 0.82)
            .line_height(size)
            .child(glyph.clone())
            .into_any_element(),
        (None, None) => div().size(size).into_any_element(),
    }
}

/// An emoji picker bound to retained `EmojiPickerState`.
///
/// Put it in a popover anchored to a reaction or composer button, and close
/// the popover on `EmojiPickerEvent::Pick` or `Cancel`.
#[derive(IntoElement)]
pub struct EmojiPicker {
    state: Entity<EmojiPickerState>,
    bordered: bool,
    style: StyleRefinement,
}

impl EmojiPicker {
    /// Creates a picker for retained state.
    pub fn new(state: &Entity<EmojiPickerState>) -> Self {
        Self {
            state: state.clone(),
            bordered: true,
            style: StyleRefinement::default(),
        }
    }
    /// Draws the rounded border, on by default. Turn it off inside a popover surface.
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }
}

impl Styled for EmojiPicker {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for EmojiPicker {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx);
        let root = div()
            .flex_none()
            .overflow_hidden()
            .rounded(theme.radius.xl)
            .when(self.bordered, |root| {
                root.border_1()
                    .border_color(theme.colors.border)
                    .shadow(theme.shadows.md.clone())
            })
            .child(self.state);
        apply_style(root, &self.style)
    }
}

#[cfg(test)]
mod tests {
    use super::{COLUMNS, Emoji, Row, grid_rows, matches_query, step_row};

    #[test]
    fn sections_wrap_into_rows_after_their_headings() {
        let rows = grid_rows([COLUMNS + 2, 0, 1], true);
        assert_eq!(
            rows,
            vec![
                Row::Heading(0),
                Row::Emoji {
                    section: 0,
                    cells: 0..COLUMNS
                },
                Row::Emoji {
                    section: 0,
                    cells: COLUMNS..COLUMNS + 2
                },
                Row::Heading(2),
                Row::Emoji {
                    section: 2,
                    cells: 0..1
                },
            ]
        );
    }

    #[test]
    fn up_and_down_keep_the_column_across_sections() {
        let sections = [COLUMNS + 2, 0, COLUMNS];
        assert_eq!(step_row(&sections, (0, 1), 1), (0, COLUMNS + 1));
        // The short last row clamps to its final emoji.
        assert_eq!(step_row(&sections, (0, 5), 1), (0, COLUMNS + 1));
        // Down from the last row skips the empty section.
        assert_eq!(step_row(&sections, (0, COLUMNS), 1), (2, 0));
        assert_eq!(step_row(&sections, (2, 4), -1), (0, COLUMNS + 1));
        assert_eq!(step_row(&sections, (0, 3), -1), (0, 3));
        assert_eq!(step_row(&sections, (2, 4), 1), (2, 4));
    }

    #[test]
    fn search_matches_names_first_then_keywords() {
        let tada = Emoji::new("tada", "🎉").keywords(["party"]);
        assert_eq!(matches_query(&tada, "ta"), Some(true));
        assert_eq!(matches_query(&tada, "ad"), Some(false));
        assert_eq!(matches_query(&tada, "par"), Some(false));
        assert_eq!(matches_query(&tada, "zz"), None);
    }
}
