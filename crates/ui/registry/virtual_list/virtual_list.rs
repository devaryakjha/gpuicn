//! Virtualized native rows with stable identity, selection and keyboard navigation.
//!
//! GPUI owns measurement and scrolling; GPUI Kit owns the scrollbar. Keep this
//! state in your view. Replace metadata only when data changes, not on every render.
//! Domain data, loading, filtering and actions remain in the application.

use super::theme::UiTheme;
use gpui_kit::accesskit::Role;
use gpui_kit::{
    AnyElement, App, ElementId, FontWeight, InteractiveElement as _, IntoElement, ListOffset,
    Modifiers, MouseButton, ParentElement as _, Pixels, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, UniformListScrollHandle, Window, div, point,
    prelude::FluentBuilder as _, px, uniform_list,
};
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    fmt,
    rc::Rc,
    sync::Arc,
};

/// Metadata for one stable row. The renderer supplies its visual content.
#[derive(Clone, Debug)]
pub struct ListItem {
    /// Stable application identity, independent of row order.
    pub id: ElementId,
    /// Complete accessible name, even when visual text is truncated.
    pub label: SharedString,
    /// Whether selection and activation are unavailable.
    pub disabled: bool,
}
impl ListItem {
    /// IDs identify data, not its current position. Labels may repeat; IDs may not.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            disabled: false,
        }
    }
    /// Disabled rows remain visible but cannot be selected or activated.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// A rejected replacement leaves items, selection and scroll position unchanged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DuplicateListItemId(pub ElementId);
impl fmt::Display for DuplicateListItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "duplicate list item ID: {:?}", self.0)
    }
}
impl std::error::Error for DuplicateListItemId {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Whether gestures can select one row or several rows.
pub enum ListSelectionMode {
    #[default]
    /// Selection contains at most one enabled row.
    Single,
    /// Enable ranges, toggling and select-all.
    Multiple,
}

/// Selection operations apply to the current order and current anchor identity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListSelectionGesture {
    #[default]
    /// Replace selection and move the range anchor.
    Replace,
    /// Toggle this row and move the range anchor.
    Toggle,
    /// Replace selection with enabled rows between anchor and target.
    Range,
    /// Add the current anchor-to-target range to selection.
    AddRange,
}
impl ListSelectionGesture {
    fn from_modifiers(modifiers: Modifiers) -> Self {
        match (modifiers.shift, modifiers.platform || modifiers.control) {
            (false, false) => Self::Replace,
            (false, true) => Self::Toggle,
            (true, false) => Self::Range,
            (true, true) => Self::AddRange,
        }
    }
}

/// Events contain stable IDs. Read current selection from the view-owned state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VirtualListEvent {
    /// A pointer or keyboard selection gesture updated state.
    SelectionChanged,
    /// Keyboard navigation moved focus without changing selection.
    FocusChanged,
    /// Enter or double-click requested activation of a current enabled row.
    Activate(ElementId),
}

/// Facts about the row being rendered. Never use `index` as an action identity.
#[derive(Clone, Debug)]
pub struct ListRow {
    /// Current metadata, including stable identity and accessible name.
    pub item: ListItem,
    /// Position in the current ordering, for rendering only.
    pub index: usize,
    /// Whether this row belongs to selection.
    pub selected: bool,
    /// Whether this is the list's active row, independent of keyboard focus.
    pub focused: bool,
}

struct Data {
    items: Vec<ListItem>,
    positions: HashMap<ElementId, usize>,
    selected: HashSet<ElementId>,
    focused: Option<ElementId>,
    anchor: Option<ElementId>,
    mode: ListSelectionMode,
    height: Pixels,
}

/// Caller-owned retained list state. Clones refer to the same state, like GPUI's
/// scroll handles. Use a separate state for each independently scrolling list.
#[derive(Clone)]
pub struct VirtualListState {
    data: Rc<RefCell<Data>>,
    scroll: UniformListScrollHandle,
    rows_rendered: Rc<Cell<usize>>,
}
impl VirtualListState {
    /// Build metadata and its ID index once. Only visible rows plus GPUI's
    /// measurement row are rendered, regardless of the total item count.
    pub fn new(items: Vec<ListItem>) -> Result<Self, DuplicateListItemId> {
        let state = Self {
            data: Rc::new(RefCell::new(Data {
                items: Vec::new(),
                positions: HashMap::new(),
                selected: HashSet::new(),
                focused: None,
                anchor: None,
                mode: ListSelectionMode::Single,
                height: px(32.),
            })),
            scroll: UniformListScrollHandle::new(),
            rows_rendered: Rc::new(Cell::new(0)),
        };
        state.replace_items(items)?;
        Ok(state)
    }

    /// Replace metadata atomically. Surviving IDs retain selection and focus.
    /// The first visible ID and its pixel offset remain anchored. If removed,
    /// prefer the next surviving old row, then the previous one, then the start.
    /// This is O(n) work on data changes; rendering does not rebuild the ID index.
    pub fn replace_items(&self, items: Vec<ListItem>) -> Result<(), DuplicateListItemId> {
        let mut positions = HashMap::with_capacity(items.len());
        for (index, item) in items.iter().enumerate() {
            if positions.insert(item.id.clone(), index).is_some() {
                return Err(DuplicateListItemId(item.id.clone()));
            }
        }
        let top = self.scroll_top();
        let mut data = self.data.borrow_mut();
        let surviving = |start: usize, enabled: bool| {
            data.items
                .iter()
                .skip(start)
                .chain(data.items.iter().take(start).rev())
                .filter_map(|item| positions.get(&item.id).copied())
                .find(|index| !enabled || !items[*index].disabled)
        };
        let exact_top = data
            .items
            .get(top.item_ix)
            .and_then(|item| positions.get(&item.id))
            .copied();
        let new_top = exact_top
            .or_else(|| surviving(top.item_ix, false))
            .unwrap_or(0);
        let new_focus = data
            .focused
            .as_ref()
            .and_then(|id| {
                positions
                    .get(id)
                    .copied()
                    .filter(|index| !items[*index].disabled)
                    .or_else(|| surviving(*data.positions.get(id).unwrap_or(&0), true))
            })
            .map(|index| items[index].id.clone());
        data.selected
            .retain(|id| positions.get(id).is_some_and(|i| !items[*i].disabled));
        data.focused = new_focus;
        if data
            .anchor
            .as_ref()
            .is_none_or(|id| positions.get(id).is_none_or(|i| items[*i].disabled))
        {
            data.anchor = data.focused.clone();
        }
        data.items = items;
        data.positions = positions;
        drop(data);
        self.scroll_to(ListOffset {
            item_ix: new_top,
            offset_in_item: if exact_top.is_some() {
                top.offset_in_item
            } else {
                px(0.)
            },
        });
        Ok(())
    }

    /// Change selection mode. Switching to Single keeps the focused selected row,
    /// or the first selected row in current order.
    pub fn set_selection_mode(&self, mode: ListSelectionMode) {
        let mut data = self.data.borrow_mut();
        data.mode = mode;
        if mode == ListSelectionMode::Single && data.selected.len() > 1 {
            let keep = data
                .focused
                .as_ref()
                .filter(|id| data.selected.contains(*id))
                .cloned()
                .or_else(|| {
                    data.items
                        .iter()
                        .find(|item| data.selected.contains(&item.id))
                        .map(|item| item.id.clone())
                });
            data.selected.clear();
            data.selected.extend(keep);
        }
    }

    /// Set selection from application state, ignoring missing/disabled IDs.
    /// In Single mode the first supplied valid ID wins. Does not scroll.
    pub fn set_selected(&self, ids: impl IntoIterator<Item = ElementId>) {
        let mut data = self.data.borrow_mut();
        data.selected.clear();
        for id in ids {
            if data
                .positions
                .get(&id)
                .is_some_and(|i| !data.items[*i].disabled)
            {
                data.selected.insert(id);
                if data.mode == ListSelectionMode::Single {
                    break;
                }
            }
        }
    }

    /// Selected stable IDs in current row order. No full row scan for small selections.
    pub fn selected(&self) -> Vec<ElementId> {
        let data = self.data.borrow();
        let mut ids: Vec<_> = data.selected.iter().cloned().collect();
        ids.sort_unstable_by_key(|id| data.positions[id]);
        ids
    }
    /// Current selection count without allocating a snapshot.
    pub fn selected_count(&self) -> usize {
        self.data.borrow().selected.len()
    }
    /// Current focused identity, independent of selection.
    pub fn focused(&self) -> Option<ElementId> {
        self.data.borrow().focused.clone()
    }
    /// Current row count.
    pub fn len(&self) -> usize {
        self.data.borrow().items.len()
    }
    /// Whether the current metadata is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Resolve an identity against the current data, for application intent handlers.
    pub fn index_of(&self, id: &ElementId) -> Option<usize> {
        self.data.borrow().positions.get(id).copied()
    }
    /// A metadata snapshot for one current row; this does not build a visual row.
    pub fn item(&self, index: usize) -> Option<ListItem> {
        self.data.borrow().items.get(index).cloned()
    }
    /// Number of row-renderer calls in the most recent layout, including GPUI's
    /// measurement row. Useful for native qualification; not a count of all retained IDs.
    pub fn rows_rendered(&self) -> usize {
        self.rows_rendered.get()
    }

    /// Reveal by current identity. Returns false for a missing row.
    pub fn reveal(&self, id: &ElementId) -> bool {
        let Some(index) = self.index_of(id) else {
            return false;
        };
        self.scroll_to(ListOffset {
            item_ix: index,
            offset_in_item: px(0.),
        });
        true
    }
    /// Apply a gesture to the current ID/order. Missing or disabled IDs are ignored,
    /// including callbacks captured before a data replacement.
    pub fn select(&self, id: &ElementId, gesture: ListSelectionGesture) -> bool {
        let mut data = self.data.borrow_mut();
        let Some(&index) = data.positions.get(id) else {
            return false;
        };
        if data.items[index].disabled {
            return false;
        }
        if data.mode == ListSelectionMode::Single || gesture == ListSelectionGesture::Replace {
            data.selected.clear();
            data.selected.insert(id.clone());
            data.anchor = Some(id.clone());
        } else if gesture == ListSelectionGesture::Toggle {
            if !data.selected.remove(id) {
                data.selected.insert(id.clone());
            }
            data.anchor = Some(id.clone());
        } else {
            let anchor = data
                .anchor
                .as_ref()
                .or(data.focused.as_ref())
                .and_then(|id| data.positions.get(id))
                .copied()
                .unwrap_or(index);
            let anchor_id = data.items[anchor].id.clone();
            if gesture == ListSelectionGesture::Range {
                data.selected.clear();
            }
            for index in anchor.min(index)..=anchor.max(index) {
                let item = &data.items[index];
                if !item.disabled {
                    let id = item.id.clone();
                    data.selected.insert(id);
                }
            }
            data.anchor = Some(anchor_id);
        }
        data.focused = Some(id.clone());
        true
    }

    fn configure(&self, height: Pixels) {
        let top = self.scroll_top();
        if self.data.borrow().height != height {
            self.data.borrow_mut().height = height;
            self.scroll_to(top);
        }
    }

    fn scroll_top(&self) -> ListOffset {
        let height = self.data.borrow().height;
        let offset = (-self.scroll.0.borrow().base_handle.offset().y).max(px(0.));
        let item_ix = (f32::from(offset.max(px(0.))) / f32::from(height)).floor() as usize;
        ListOffset {
            item_ix,
            offset_in_item: offset - height * item_ix,
        }
    }

    fn scroll_to(&self, top: ListOffset) {
        let height = self.data.borrow().height;
        self.scroll
            .0
            .borrow()
            .base_handle
            .set_offset(point(px(0.), -(height * top.item_ix + top.offset_in_item)));
    }

    fn reveal_nearest(&self, index: usize) {
        let height = self.data.borrow().height;
        let scroll = self.scroll.0.borrow();
        let viewport = scroll
            .last_item_size
            .map_or(px(0.), |size| size.item.height);
        let offset = -scroll.base_handle.offset().y;
        let top = height * index;
        let next = if top < offset || viewport <= px(0.) {
            top
        } else if top + height > offset + viewport {
            top + height - viewport
        } else {
            return;
        };
        scroll.base_handle.set_offset(point(px(0.), -next));
    }

    fn row(&self, index: usize) -> Option<ListRow> {
        let data = self.data.borrow();
        let item = data.items.get(index)?.clone();
        Some(ListRow {
            selected: data.selected.contains(&item.id),
            focused: data.focused.as_ref() == Some(&item.id),
            item,
            index,
        })
    }

    fn key(&self, key: &str, modifiers: Modifiers) -> Option<VirtualListEvent> {
        let data = self.data.borrow();
        let command = modifiers.platform || modifiers.control;
        if modifiers.alt {
            return None;
        }
        if key == "a" && command && data.mode == ListSelectionMode::Multiple {
            drop(data);
            let mut data = self.data.borrow_mut();
            data.selected = data
                .items
                .iter()
                .filter(|item| !item.disabled)
                .map(|item| item.id.clone())
                .collect();
            return Some(VirtualListEvent::SelectionChanged);
        }
        let current = data
            .focused
            .as_ref()
            .and_then(|id| data.positions.get(id))
            .copied();
        if key == "enter" {
            return current
                .filter(|i| !data.items[*i].disabled)
                .map(|i| VirtualListEvent::Activate(data.items[i].id.clone()));
        }
        if key == "space" {
            let id = current.map(|i| data.items[i].id.clone())?;
            drop(data);
            self.select(
                &id,
                if modifiers.shift {
                    ListSelectionGesture::Range
                } else {
                    ListSelectionGesture::Toggle
                },
            );
            return Some(VirtualListEvent::SelectionChanged);
        }
        let count = data.items.len();
        if count == 0 {
            return None;
        }
        let page = (f32::from(
            self.scroll
                .0
                .borrow()
                .last_item_size
                .map_or(px(0.), |size| size.item.height),
        ) / f32::from(data.height))
        .floor()
        .max(1.) as usize;
        let (start, reverse) = match key {
            "up" => (current.map_or(count - 1, |i| i.saturating_sub(1)), true),
            "down" => (current.map_or(0, |i| (i + 1).min(count - 1)), false),
            "home" => (0, false),
            "end" => (count - 1, true),
            "pageup" => (current.unwrap_or(0).saturating_sub(page), true),
            "pagedown" => (
                current.unwrap_or(0).saturating_add(page).min(count - 1),
                false,
            ),
            _ => return None,
        };
        let index = if reverse {
            (0..=start).rev().find(|i| !data.items[*i].disabled)
        } else {
            (start..count).find(|i| !data.items[*i].disabled)
        }
        .or(current)?;
        let id = data.items[index].id.clone();
        drop(data);
        if command && !modifiers.shift {
            self.data.borrow_mut().focused = Some(id);
        } else {
            self.select(&id, ListSelectionGesture::from_modifiers(modifiers));
        }
        self.reveal_nearest(index);
        Some(if command && !modifiers.shift {
            VirtualListEvent::FocusChanged
        } else {
            VirtualListEvent::SelectionChanged
        })
    }
}

type Renderer = Rc<dyn Fn(&ListRow, &mut Window, &mut App) -> AnyElement>;
type EventHandler = Rc<dyn Fn(&VirtualListEvent, &mut Window, &mut App)>;

/// A single-tab-stop listbox with lazy, caller-rendered rows. Arrow/Home/End/Page
/// keys move selection; Shift extends ranges; Cmd/Ctrl toggles or selects all;
/// Enter and double-click emit activation. Place noninteractive row content or
/// existing ContextMenu parts inside rows, and keep other controls outside.
#[derive(IntoElement)]
pub struct VirtualList {
    id: ElementId,
    label: SharedString,
    state: VirtualListState,
    render_item: Renderer,
    on_event: Option<EventHandler>,
    height: Option<Pixels>,
    empty: Option<AnyElement>,
    style: gpui_kit::StyleRefinement,
}
impl VirtualList {
    /// Compose a list from retained state and a visible-row renderer.
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: VirtualListState,
        render_item: impl Fn(&ListRow, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            state,
            render_item: Rc::new(render_item),
            on_event: None,
            height: None,
            empty: None,
            style: Default::default(),
        }
    }
    /// Fixed row height; defaults to eight spacing units. Invalid heights fall
    /// back to the theme default. Keep text density usable when overriding it.
    pub fn row_height(mut self, height: Pixels) -> Self {
        self.height = Some(height);
        self
    }
    /// Observe live selection/focus or perform an application-owned activation.
    /// State is updated before this callback; it may replace selection or data.
    pub fn on_event(
        mut self,
        handler: impl Fn(&VirtualListEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_event = Some(Rc::new(handler));
        self
    }
    /// Compose loading/error/empty content from existing components. It appears
    /// only when the caller supplies no rows; this component does not load data.
    pub fn empty(mut self, content: impl IntoElement) -> Self {
        self.empty = Some(content.into_any_element());
        self
    }
}
impl Styled for VirtualList {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}
impl RenderOnce for VirtualList {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let default_height = theme.spacing.unit * 8_f32;
        let height = self
            .height
            .filter(|h| f32::from(*h).is_finite() && *h >= px(1.))
            .unwrap_or(default_height);
        self.state.configure(height);
        self.state.rows_rendered.set(0);
        let focus = window
            .use_keyed_state(
                ElementId::NamedChild(Arc::new(self.id.clone()), "focus".into()),
                cx,
                |_, cx| cx.focus_handle(),
            )
            .read(cx)
            .clone();
        let state = self.state.clone();
        let change = self.on_event.clone();
        let key_focus = focus.clone();
        let instructions = if self.state.data.borrow().mode == ListSelectionMode::Multiple {
            "Multi-select list. Use arrows to navigate, Shift for a range, Command or Control to toggle."
        } else {
            "Use arrow keys to navigate and Enter to activate."
        };
        let mut root = div()
            .id(self.id.clone())
            .role(Role::ListBox)
            .aria_label(self.label)
            .aria_description(instructions)
            .border_1()
            .border_color(theme.colors.background.opacity(0.))
            .when(self.state.focused().is_none(), |el| {
                el.focus_visible(move |el| el.border_color(theme.colors.ring))
            })
            .track_focus(&focus.clone().tab_stop(true))
            .relative()
            .size_full()
            .min_w_0()
            .min_h_0()
            .overflow_hidden()
            .font_family(theme.fonts.body.clone())
            .text_size(px(14.) * theme.text_scale)
            .line_height(px(20.) * theme.text_scale)
            .bg(theme.colors.background)
            .text_color(theme.colors.foreground)
            .on_key_down(move |event, window, cx| {
                if !key_focus.is_focused(window) {
                    return;
                }
                if let Some(event) = state.key(&event.keystroke.key, event.keystroke.modifiers) {
                    window.refresh();
                    if let Some(change) = &change {
                        change(&event, window, cx);
                    }
                    window.prevent_default();
                    cx.stop_propagation();
                }
            });
        if self.state.is_empty() {
            root = root.child(self.empty.unwrap_or_else(|| {
                div()
                    .p(theme.spacing.unit * 4_f32)
                    .text_color(theme.colors.muted_foreground)
                    .child("No items")
                    .into_any_element()
            }));
        } else {
            let state = self.state.clone();
            let total = state.len();
            let renderer = self.render_item;
            let change = self.on_event;
            let rows_theme = theme.clone();
            let rows = uniform_list("virtual-list.rows", total, move |range, window, cx| {
                range
                    .map(|index| {
                        let Some(row) = state.row(index) else {
                            return div().into_any_element();
                        };
                        state.rows_rendered.set(state.rows_rendered.get() + 1);
                        let id = row.item.id.clone();
                        let down_state = state.clone();
                        let down_id = id.clone();
                        let down_focus = focus.clone();
                        let down_change = change.clone();
                        let left_state = state.clone();
                        let left_id = id.clone();
                        let left_focus = focus.clone();
                        let left_change = change.clone();
                        let activate_state = state.clone();
                        let activate_change = change.clone();
                        let has_focus = focus.is_focused(window);
                        let content = renderer(&row, window, cx);
                        let tooltip_label = row.item.label.clone();
                        let mut element = div()
                            .id(id.clone())
                            .role(Role::ListBoxOption)
                            .aria_label(row.item.label.clone())
                            .tooltip(move |_, cx| {
                                super::tooltip::text_tooltip(tooltip_label.clone(), cx)
                            })
                            .aria_selected(row.selected)
                            .aria_position_in_set(index + 1)
                            .aria_size_of_set(total)
                            .when(row.focused, |el| el.aria_active_descendant())
                            .when(row.item.disabled, |el| {
                                el.aria_description("Unavailable").opacity(0.5)
                            })
                            .w_full()
                            .min_w_0()
                            .flex()
                            .items_center()
                            .gap(rows_theme.spacing.unit * 2_f32)
                            .px(rows_theme.spacing.unit * 2_f32)
                            .h(height)
                            .overflow_hidden()
                            .rounded(rows_theme.radius.md)
                            .border_1()
                            .border_color(rows_theme.colors.background.opacity(0.))
                            .when(row.selected, |el| {
                                el.bg(rows_theme.colors.accent)
                                    .text_color(rows_theme.colors.accent_foreground)
                            })
                            .when(!row.item.disabled, |el| {
                                el.hover(|el| el.bg(rows_theme.colors.accent))
                            })
                            .when(row.focused && has_focus, |el| {
                                el.border_color(rows_theme.colors.ring)
                            })
                            .font_weight(if row.selected {
                                FontWeight::MEDIUM
                            } else {
                                FontWeight::NORMAL
                            })
                            .capture_any_mouse_down(move |event, window, cx| {
                                if event.button != MouseButton::Right
                                    || (down_focus.contains_focused(window, cx)
                                        && !down_focus.is_focused(window))
                                {
                                    return;
                                }
                                let valid = down_state
                                    .index_of(&down_id)
                                    .and_then(|i| down_state.item(i))
                                    .is_some_and(|item| !item.disabled);
                                if !valid {
                                    return;
                                }
                                down_focus.focus(window, cx);
                                let keep_selection =
                                    down_state.data.borrow().selected.contains(&down_id);
                                if keep_selection {
                                    down_state.data.borrow_mut().focused = Some(down_id.clone());
                                } else {
                                    down_state.select(&down_id, ListSelectionGesture::Replace);
                                }
                                window.refresh();
                                if let Some(change) = &down_change {
                                    change(&VirtualListEvent::SelectionChanged, window, cx);
                                }
                            })
                            .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                                // A menu owns descendant focus while its portal is open.
                                if left_focus.contains_focused(window, cx)
                                    && !left_focus.is_focused(window)
                                {
                                    return;
                                }
                                if left_state.select(
                                    &left_id,
                                    ListSelectionGesture::from_modifiers(event.modifiers),
                                ) {
                                    left_focus.focus(window, cx);
                                    window.refresh();
                                    if let Some(change) = &left_change {
                                        change(&VirtualListEvent::SelectionChanged, window, cx);
                                    }
                                }
                            })
                            .on_click(move |event, window, cx| {
                                if event.click_count() == 2
                                    && activate_state
                                        .index_of(&id)
                                        .and_then(|i| activate_state.item(i))
                                        .is_some_and(|item| !item.disabled)
                                    && let Some(change) = &activate_change
                                {
                                    change(&VirtualListEvent::Activate(id.clone()), window, cx);
                                }
                            })
                            .child(content);
                        if !row.item.disabled {
                            for action in [
                                gpui_kit::accesskit::Action::Click,
                                gpui_kit::accesskit::Action::Focus,
                            ] {
                                let state = state.clone();
                                let id = row.item.id.clone();
                                let focus = focus.clone();
                                let change = change.clone();
                                element = element.on_a11y_action(action, move |_, window, cx| {
                                    let valid = state
                                        .index_of(&id)
                                        .and_then(|i| state.item(i))
                                        .is_some_and(|item| !item.disabled);
                                    if !valid {
                                        return;
                                    }
                                    let event = if action == gpui_kit::accesskit::Action::Focus {
                                        state.data.borrow_mut().focused = Some(id.clone());
                                        VirtualListEvent::FocusChanged
                                    } else {
                                        state.select(&id, ListSelectionGesture::Replace);
                                        VirtualListEvent::SelectionChanged
                                    };
                                    {
                                        focus.focus(window, cx);
                                        if let Some(index) = state.index_of(&id) {
                                            state.reveal_nearest(index);
                                        }
                                        window.refresh();
                                        if let Some(change) = &change {
                                            change(&event, window, cx);
                                        }
                                    }
                                });
                            }
                        }
                        element.into_any_element()
                    })
                    .collect::<Vec<_>>()
            })
            .track_scroll(&self.state.scroll)
            .size_full();
            root = root
                .child(rows)
                .child(super::scroll_area::scroll_area_scrollbar_for(
                    ElementId::NamedChild(Arc::new(self.id), "scrollbar".into()),
                    &self.state.scroll,
                    cx,
                ));
        }
        super::theme::apply_style(root, &self.style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{
        AppContext as _, Context, Render, TestAppContext, VisualTestContext, point, size,
    };

    fn items(ids: &[u64]) -> Vec<ListItem> {
        ids.iter()
            .map(|id| ListItem::new(("row", *id), format!("Row {id}")))
            .collect()
    }
    fn id(value: u64) -> ElementId {
        ("row", value).into()
    }

    #[test]
    fn selection_uses_current_identity_order_and_disabled_guards() {
        let state = VirtualListState::new(items(&[1, 2, 3, 4, 5])).unwrap();
        state.set_selection_mode(ListSelectionMode::Multiple);
        state.select(&id(2), ListSelectionGesture::Replace);
        state.select(&id(4), ListSelectionGesture::Range);
        assert_eq!(state.selected(), [id(2), id(3), id(4)]);
        state.select(&id(3), ListSelectionGesture::Toggle);
        assert_eq!(state.selected(), [id(2), id(4)]);
        let mut next = items(&[5, 3, 2, 4]);
        next[2].disabled = true;
        state.replace_items(next).unwrap();
        assert_eq!(state.selected(), [id(4)]);
        assert!(!state.select(&id(1), ListSelectionGesture::Replace));
        assert!(!state.select(&id(2), ListSelectionGesture::Replace));
        state.select(&id(5), ListSelectionGesture::Range);
        assert_eq!(state.selected(), [id(5), id(3)]);
        state.set_selection_mode(ListSelectionMode::Single);
        assert_eq!(state.selected(), [id(5)]);
        state.set_selected([id(2), id(4), id(3)]);
        assert_eq!(state.selected(), [id(4)]);
    }

    #[test]
    fn replacements_preserve_viewport_and_reject_duplicates_atomically() {
        let state = VirtualListState::new(items(&[1, 2, 3, 4, 5])).unwrap();
        state.select(&id(3), ListSelectionGesture::Replace);
        state.scroll_to(ListOffset {
            item_ix: 2,
            offset_in_item: px(9.),
        });
        state.replace_items(items(&[5, 3, 2, 1, 4])).unwrap();
        let top = state.scroll_top();
        assert_eq!((top.item_ix, top.offset_in_item), (1, px(9.)));
        assert_eq!(state.focused(), Some(id(3)));
        assert_eq!(
            state.replace_items(items(&[8, 8])),
            Err(DuplicateListItemId(id(8)))
        );
        assert_eq!(state.selected(), [id(3)]);
        assert_eq!(state.len(), 5);
        state.replace_items(items(&[5, 4, 2])).unwrap();
        assert_eq!(state.focused(), Some(id(2))); // Next surviving old identity.
        assert!(state.selected().is_empty());
        let top = state.scroll_top();
        assert_eq!((top.item_ix, top.offset_in_item), (2, px(0.)));
        state.replace_items(Vec::new()).unwrap();
        assert_eq!(state.focused(), None);
        assert!(!state.reveal(&id(3)));
    }

    struct View {
        state: VirtualListState,
        activated: Option<ElementId>,
    }
    impl Render for View {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            VirtualList::new("list", "Test rows", self.state.clone(), move |row, _, _| {
                div().child(row.item.label.clone()).into_any_element()
            })
            .on_event(cx.listener(|view, event, _, _| {
                if let VirtualListEvent::Activate(id) = event {
                    view.activated = Some(id.clone());
                }
            }))
        }
    }

    #[test]
    fn large_list_virtualizes_reveals_and_handles_real_keyboard_and_pointer_input() {
        {
            let mut cx = TestAppContext::single();
            cx.update(|cx| UiTheme::set(cx, UiTheme::neutral_light()));
            let state = VirtualListState::new(
                (0..100_000u64)
                    .map(|i| ListItem::new(("row", i), format!("Row {i}")).disabled(i == 2))
                    .collect(),
            )
            .unwrap();
            state.set_selection_mode(ListSelectionMode::Multiple);
            let window = cx.add_window({
                let state = state.clone();
                move |_, _| View {
                    state,
                    activated: None,
                }
            });
            let mut visual = VisualTestContext::from_window(window.into(), &cx);
            visual.simulate_resize(size(px(480.), px(320.)));
            let draw = |cx: &mut TestAppContext| {
                cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
                    .unwrap();
            };
            draw(&mut cx);
            assert!(
                state.rows_rendered() > 0 && state.rows_rendered() < 40,
                "{} rows built",
                state.rows_rendered()
            );
            visual.simulate_click(point(px(60.), px(15.)), Modifiers::default());
            visual.simulate_keystrokes("down shift-down");
            draw(&mut cx);
            assert_eq!(state.selected(), [id(1), id(3)]);
            assert_eq!(state.focused(), Some(id(3)));
            visual.simulate_keystrokes("enter");
            assert_eq!(
                cx.read_window(&window, |view, cx| view.read(cx).activated.clone())
                    .unwrap(),
                Some(id(3))
            );
            state.reveal(&id(99_999));
            draw(&mut cx);
            assert!(state.scroll_top().item_ix <= 99_999 && state.scroll_top().item_ix >= 99_989);
            assert!(state.rows_rendered() < 40);
            visual.simulate_keystrokes("end");
            draw(&mut cx);
            assert_eq!(state.focused(), Some(id(99_999)));
            assert!(state.scroll_top().item_ix <= 99_999 && state.scroll_top().item_ix >= 99_989);
            visual.simulate_keystrokes("home");
            draw(&mut cx);
            assert_eq!(state.focused(), Some(id(0)));
            // Selection callback from the previous frame must not target the new
            // row occupying its old index after replacement.
            state.replace_items(items(&[11, 12])).unwrap();
            assert!(!state.select(&id(0), ListSelectionGesture::Replace));
            assert!(!state.selected().contains(&id(11)));
            // Let GPUI Kit's scrollbar idle timer release its runtime handle.
            cx.run_until_parked();
            cx.update_window(window.into(), |_, window, _| window.remove_window())
                .unwrap();
            cx.executor()
                .advance_clock(std::time::Duration::from_secs(3));
            cx.run_until_parked();
        }
    }
}
