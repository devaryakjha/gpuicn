//! Sortable: rows a person drags, or moves with the keyboard, to reorder them
//! and move them between groups, as in a chat sidebar's channel categories.
//!
//! shadcn/ui has no sortable list; this builds on GPUI's drag and drop. The
//! drag preview is a copy of the row at reduced opacity, a line in the
//! `ring` color marks where it will land, and a group that is empty or
//! collapsed fills with `accent` when the row would drop into it. Nothing
//! animates: rows move when the application re-renders in the new order, so
//! reduced motion needs no special case. The application owns the order;
//! every move arrives as one [`SortableMove`] to persist.

use std::{collections::HashMap, rc::Rc};

use gpui_kit::base::{ElementExt as _, TestSupportExt as _};
use gpui_kit::{
    AnyElement, App, AppContext as _, Context, ElementId, EntityId, FontWeight,
    InteractiveElement as _, IntoElement, KeyDownEvent, ParentElement as _, Pixels, Render,
    RenderOnce, Role, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};

use super::theme::{
    UiTheme, apply_style,
    primitives::{focus, text},
};

type RowRender = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;
type MoveHandler = Rc<dyn Fn(&SortableMove, &mut Window, &mut App)>;
type GroupMoveHandler = Rc<dyn Fn(usize, usize, &mut Window, &mut App)>;

/// A row's place: its group, then its index within the group.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct SortablePosition {
    /// The group's index in the list.
    pub group: usize,
    /// The row's index in its group.
    pub index: usize,
}

impl SortablePosition {
    /// The row at `index` in `group`.
    pub fn new(group: usize, index: usize) -> Self {
        Self { group, index }
    }
}

/// One row moved: where it was, and where it is once the move is applied.
///
/// `to.index` counts after the row has left `from`, so applying a move is a
/// remove followed by an insert, which is what [`SortableMove::apply`] does.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SortableMove {
    /// The row's place before the move.
    pub from: SortablePosition,
    /// The row's place after the move.
    pub to: SortablePosition,
}

impl SortableMove {
    /// Applies the move to the application's groups of rows.
    pub fn apply<T>(&self, groups: &mut [Vec<T>]) {
        let row = groups[self.from.group].remove(self.from.index);
        let to = &mut groups[self.to.group];
        let index = self.to.index.min(to.len());
        to.insert(index, row);
    }
}

/// Moves the group at `from` to `to`, counted after it has left `from`, as
/// [`Sortable::on_reorder_group`] reports it.
pub fn move_group<T>(groups: &mut Vec<T>, from: usize, to: usize) {
    let group = groups.remove(from);
    groups.insert(to.min(groups.len()), group);
}

/// A draggable row. Its content is built by a closure, because the drag
/// preview draws a second copy of it.
pub struct SortableRow {
    id: ElementId,
    label: SharedString,
    render: RowRender,
}

impl SortableRow {
    /// Creates a row. `id` must be unique in the whole list, not only in its
    /// group, so the row keeps its state (and focus) when it changes group.
    /// `label` names it in move announcements.
    pub fn new<E: IntoElement>(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        render: impl Fn(&mut Window, &mut App) -> E + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            render: Rc::new(move |window, cx| render(window, cx).into_any_element()),
        }
    }
}

/// A group of rows with a header, such as a sidebar category.
pub struct SortableGroup {
    id: ElementId,
    label: SharedString,
    header: Option<RowRender>,
    collapsed: bool,
    rows: Vec<SortableRow>,
}

impl SortableGroup {
    /// Creates a group named `label`. Without [`SortableGroup::header`], the
    /// label is the header.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            header: None,
            collapsed: false,
            rows: Vec::new(),
        }
    }
    /// Replaces the header, for example with a `Collapsible` trigger. Group
    /// drags start from it when the list reorders groups.
    pub fn header<E: IntoElement>(
        mut self,
        header: impl Fn(&mut Window, &mut App) -> E + 'static,
    ) -> Self {
        self.header = Some(Rc::new(move |window, cx| {
            header(window, cx).into_any_element()
        }));
        self
    }
    /// Hides the rows. A row dropped on a collapsed group goes to its end.
    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }
    /// Adds a row.
    pub fn row(mut self, row: SortableRow) -> Self {
        self.rows.push(row);
        self
    }
    /// Adds rows.
    pub fn rows(mut self, rows: impl IntoIterator<Item = SortableRow>) -> Self {
        self.rows.extend(rows);
        self
    }
}

/// A list of groups whose rows reorder by drag and drop and by keyboard.
///
/// Drag a row to move it within its group or into another one. With focus
/// inside a row, Alt+Shift+Up and Alt+Shift+Down move it one place, across
/// group boundaries at either end, and the new position is announced. With
/// [`Sortable::reorder_groups`], headers drag the same way and take the same
/// keys. The list reports moves and never reorders itself: apply them to the
/// application's data and render again.
#[derive(IntoElement)]
pub struct Sortable {
    id: ElementId,
    label: SharedString,
    groups: Vec<SortableGroup>,
    reorder_groups: bool,
    horizontal: bool,
    on_reorder: Option<MoveHandler>,
    on_reorder_group: Option<GroupMoveHandler>,
    style: gpui_kit::StyleRefinement,
}

impl Sortable {
    /// Creates an empty list named `label` for assistive technology.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            groups: Vec::new(),
            reorder_groups: false,
            horizontal: false,
            on_reorder: None,
            on_reorder_group: None,
            style: gpui_kit::StyleRefinement::default(),
        }
    }
    /// Adds a group.
    pub fn group(mut self, group: SortableGroup) -> Self {
        self.groups.push(group);
        self
    }
    /// Adds groups.
    pub fn groups(mut self, groups: impl IntoIterator<Item = SortableGroup>) -> Self {
        self.groups.extend(groups);
        self
    }
    /// Lets whole groups be dragged by their headers and moved with
    /// Alt+Shift+Up and Down. Off by default.
    pub fn reorder_groups(mut self, reorder: bool) -> Self {
        self.reorder_groups = reorder;
        self
    }
    /// Lays rows out in a row, such as a bar of bookmark chips, instead of a
    /// column. The drop line turns vertical, and Alt+Shift+Left and
    /// Alt+Shift+Right move the focused item, with the same announcements.
    pub fn horizontal(mut self) -> Self {
        self.horizontal = true;
        self
    }
    /// Reports a row move, by mouse or keyboard.
    pub fn on_reorder(
        mut self,
        handler: impl Fn(&SortableMove, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_reorder = Some(Rc::new(handler));
        self
    }
    /// Reports a group move as `(from, to)`, `to` counted after the group has
    /// left `from` (see [`move_group`]).
    pub fn on_reorder_group(
        mut self,
        handler: impl Fn(usize, usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_reorder_group = Some(Rc::new(handler));
        self
    }
}

impl Styled for Sortable {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}

/// Where a dragged row would land.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Target {
    /// Before the row at `slot`, or after the last row when `slot` is the
    /// group's length. Counted before the dragged row leaves.
    Slot { group: usize, slot: usize },
    /// Into an empty or collapsed group, at its end.
    Into { group: usize },
}

#[derive(Default)]
struct SortableState {
    row_target: Option<Target>,
    group_target: Option<usize>,
    dragging_row: Option<SortablePosition>,
    dragging_group: Option<usize>,
    widths: HashMap<SortablePosition, Pixels>,
    header_widths: HashMap<usize, Pixels>,
    announcement: SharedString,
}

/// The payload of a row drag. `list` keeps drops inside the list they came from.
#[derive(Clone)]
struct DraggedRow {
    list: EntityId,
    from: SortablePosition,
}

/// The payload of a group drag.
#[derive(Clone)]
struct DraggedGroup {
    list: EntityId,
    from: usize,
}

/// The floating copy of a dragged row or header.
struct DragPreview {
    render: RowRender,
    width: Pixels,
}

impl Render for DragPreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        div()
            .w(self.width)
            .opacity(0.7)
            .font_family(theme.fonts.body.clone())
            .child((self.render)(window, cx))
    }
}

/// Where a row lands when it moves one place up or down by keyboard, or
/// `None` at the very top or bottom of the list.
fn keyboard_target(
    from: SortablePosition,
    up: bool,
    lens: &[usize],
    collapsed: &[bool],
) -> Option<SortablePosition> {
    if up {
        if from.index > 0 {
            return Some(SortablePosition::new(from.group, from.index - 1));
        }
        // Into the end of the group above, collapsed or not.
        let group = from.group.checked_sub(1)?;
        Some(SortablePosition::new(group, lens[group]))
    } else {
        if from.index + 1 < lens[from.group] {
            return Some(SortablePosition::new(from.group, from.index + 1));
        }
        let group = from.group + 1;
        (group < lens.len()).then(|| {
            let index = if collapsed[group] { lens[group] } else { 0 };
            SortablePosition::new(group, index)
        })
    }
}

/// The finished move for a drop on `target`, or `None` when nothing moves.
fn drop_move(from: SortablePosition, target: Target, lens: &[usize]) -> Option<SortableMove> {
    let to = match target {
        Target::Slot { group, slot } => {
            let index = if group == from.group && slot > from.index {
                slot - 1
            } else {
                slot
            };
            SortablePosition::new(group, index)
        }
        Target::Into { group } => {
            let len = lens[group];
            SortablePosition::new(group, if group == from.group { len - 1 } else { len })
        }
    };
    (to != from).then_some(SortableMove { from, to })
}

/// Whether a key press is the move-one-place shortcut, and whether it moves
/// toward the start: Up and Down, or Left and Right in a horizontal list.
fn move_key(event: &KeyDownEvent, horizontal: bool) -> Option<bool> {
    let modifiers = &event.keystroke.modifiers;
    if !(modifiers.alt && modifiers.shift) || modifiers.control || modifiers.platform {
        return None;
    }
    match (event.keystroke.key.as_str(), horizontal) {
        ("up", false) | ("left", true) => Some(true),
        ("down", false) | ("right", true) => Some(false),
        _ => None,
    }
}

/// The 2px line that shows where a drop lands: across a column, or down
/// beside an item in a horizontal list.
fn drop_line(before: bool, horizontal: bool, theme: &UiTheme) -> gpui_kit::Div {
    let line = div().absolute().rounded_full().bg(theme.colors.ring);
    // Half the gap between items, so the line sits between them.
    let offset = -(theme.space(1.) + px(1.));
    match (horizontal, before) {
        (false, true) => line.left_0().right_0().h(px(2.)).top(px(-1.)),
        (false, false) => line.left_0().right_0().h(px(2.)).bottom(px(-1.)),
        (true, true) => line.top_0().bottom_0().w(px(2.)).left(offset),
        (true, false) => line.top_0().bottom_0().w(px(2.)).right(offset),
    }
}

impl RenderOnce for Sortable {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| SortableState::default());
        let list = state.entity_id();
        if !cx.has_active_drag() {
            state.update(cx, |state, _| {
                state.row_target = None;
                state.group_target = None;
                state.dragging_row = None;
                state.dragging_group = None;
            });
        }
        let (row_target, group_target, dragging_row, dragging_group, announcement) = {
            let state = state.read(cx);
            (
                state.row_target,
                state.group_target,
                state.dragging_row,
                state.dragging_group,
                state.announcement.clone(),
            )
        };
        let lens: Rc<Vec<usize>> = Rc::new(self.groups.iter().map(|g| g.rows.len()).collect());
        let collapsed: Rc<Vec<bool>> = Rc::new(self.groups.iter().map(|g| g.collapsed).collect());
        let group_labels: Rc<Vec<SharedString>> =
            Rc::new(self.groups.iter().map(|g| g.label.clone()).collect());
        let group_count = self.groups.len();
        let on_reorder = self.on_reorder.clone();
        let on_reorder_group = self.on_reorder_group.clone();
        let reorder_groups = self.reorder_groups && on_reorder_group.is_some();
        let horizontal = self.horizontal;
        let (previous_key, next_key) = if horizontal {
            ("Left", "Right")
        } else {
            ("Up", "Down")
        };

        let mut root = div()
            .id(self.id.clone())
            .role(Role::List)
            .aria_label(self.label.clone())
            .aria_description(format!(
                "Press Alt+Shift+{previous_key} or Alt+Shift+{next_key} to move the focused row."
            ))
            .relative()
            .flex()
            .when(horizontal, |root| root.flex_row().flex_wrap().items_center())
            .when(!horizontal, |root| root.flex_col())
            .gap(theme.space(2.))
            .child(
                div()
                    .id((self.id.clone(), "status"))
                    .test_support()
                    .role(Role::Status)
                    .aria_label(announcement)
                    .absolute()
                    .size_0(),
            );

        // A row dropped anywhere on the list lands on the last target shown.
        {
            let state = state.clone();
            let on_reorder = on_reorder.clone();
            let lens = lens.clone();
            root = root.on_drop(move |drag: &DraggedRow, window, cx| {
                if drag.list != list {
                    return;
                }
                let target = state.update(cx, |state, _| state.row_target.take());
                if let (Some(target), Some(handler)) = (target, &on_reorder)
                    && let Some(movement) = drop_move(drag.from, target, &lens)
                {
                    handler(&movement, window, cx);
                }
            });
        }
        {
            let state = state.clone();
            let on_reorder_group = on_reorder_group.clone();
            root = root.on_drop(move |drag: &DraggedGroup, window, cx| {
                if drag.list != list {
                    return;
                }
                let target = state.update(cx, |state, _| state.group_target.take());
                if let (Some(slot), Some(handler)) = (target, &on_reorder_group) {
                    let to = if slot > drag.from { slot - 1 } else { slot };
                    if to != drag.from {
                        handler(drag.from, to, window, cx);
                    }
                }
            });
        }

        for (group_index, group) in self.groups.into_iter().enumerate() {
            let empty = group.rows.is_empty();
            let drop_into = group.collapsed || empty;
            let highlighted = row_target == Some(Target::Into { group: group_index });
            let header_render = group.header.clone().unwrap_or_else(|| {
                let label = group.label.clone();
                Rc::new(move |_: &mut Window, cx: &mut App| {
                    let theme = UiTheme::read(cx);
                    text::xs(div(), theme)
                        .px(theme.space(2.))
                        .h(theme.space(8.))
                        .flex()
                        .items_center()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.colors.muted_foreground)
                        .child(label.clone())
                        .into_any_element()
                })
            });

            // The header: a drop target for rows, and the handle for group drags.
            let header_name = group.label.clone();
            let mut header = div()
                .id((group.id.clone(), "header"))
                .debug_selector(move || format!("header-{header_name}"))
                .when(group.header.is_none(), |header| {
                    header.role(Role::Heading).aria_label(group.label.clone())
                })
                .relative()
                .child(header_render(window, cx));
            {
                let state = state.clone();
                header = header.on_drag_move(
                    move |event: &gpui_kit::DragMoveEvent<DraggedRow>, window, cx| {
                        if !event.bounds.contains(&event.event.position)
                            || event.drag(cx).list != list
                        {
                            return;
                        }
                        let target = if drop_into {
                            Target::Into { group: group_index }
                        } else {
                            Target::Slot {
                                group: group_index,
                                slot: 0,
                            }
                        };
                        if state.read(cx).row_target != Some(target) {
                            state.update(cx, |state, _| state.row_target = Some(target));
                            window.refresh();
                        }
                    },
                );
            }
            if reorder_groups {
                // The label header takes focus itself, so the keyboard can
                // move the group; a custom header brings its own control.
                if group.header.is_none() {
                    let focus = window
                        .use_keyed_state((group.id.clone(), "header-focus"), cx, |_, cx| {
                            cx.focus_handle().tab_stop(true)
                        })
                        .read(cx)
                        .clone();
                    let ring = theme.clone();
                    header = header
                        .track_focus(&focus)
                        .role(Role::Button)
                        .aria_label(format!(
                            "{}, move with Alt+Shift+{previous_key} or {next_key}",
                            group.label
                        ))
                        .rounded(theme.radius.md)
                        .focus_visible(move |style| focus::ring(style, &ring));
                }
                let measured = state.clone();
                let started = state.clone();
                let render = header_render.clone();
                header = header
                    .cursor_grab()
                    .on_prepaint(move |bounds, _, cx| {
                        measured.update(cx, |state, _| {
                            state.header_widths.insert(group_index, bounds.size.width);
                        })
                    })
                    .on_drag(
                        DraggedGroup {
                            list,
                            from: group_index,
                        },
                        move |drag, _, _, cx| {
                            let width = started.update(cx, |state, _| {
                                state.dragging_group = Some(drag.from);
                                state.group_target = None;
                                state
                                    .header_widths
                                    .get(&drag.from)
                                    .copied()
                                    .unwrap_or(px(240.))
                            });
                            let render = render.clone();
                            cx.new(|_| DragPreview { render, width })
                        },
                    );
                let state = state.clone();
                let on_reorder_group = on_reorder_group.clone();
                let labels = group_labels.clone();
                header = header.on_key_down(move |event, window, cx| {
                    let Some(up) = move_key(event, horizontal) else {
                        return;
                    };
                    cx.stop_propagation();
                    let to = if up {
                        group_index.checked_sub(1)
                    } else {
                        (group_index + 1 < group_count).then_some(group_index + 1)
                    };
                    if let (Some(to), Some(handler)) = (to, &on_reorder_group) {
                        state.update(cx, |state, _| {
                            state.announcement = format!(
                                "{} moved to position {} of {}",
                                labels[group_index],
                                to + 1,
                                group_count
                            )
                            .into();
                        });
                        handler(group_index, to, window, cx);
                    }
                });
            }

            let mut rows = div()
                .flex()
                .when(horizontal, |rows| {
                    rows.flex_row()
                        .flex_wrap()
                        .items_center()
                        .gap(theme.space(2.))
                })
                .when(!horizontal, |rows| rows.flex_col());
            if !group.collapsed {
                let row_count = group.rows.len();
                for (index, row) in group.rows.into_iter().enumerate() {
                    let position = SortablePosition::new(group_index, index);
                    let line_before = row_target
                        == Some(Target::Slot {
                            group: group_index,
                            slot: index,
                        });
                    let line_after = index + 1 == row_count
                        && row_target
                            == Some(Target::Slot {
                                group: group_index,
                                slot: row_count,
                            });
                    let measured = state.clone();
                    let started = state.clone();
                    let moved = state.clone();
                    let keyed = state.clone();
                    let render = row.render.clone();
                    let on_reorder = on_reorder.clone();
                    let lens_for_keys = lens.clone();
                    let collapsed_for_keys = collapsed.clone();
                    let labels = group_labels.clone();
                    let label = row.label.clone();
                    let element = div()
                        .id(row.id.clone())
                        .relative()
                        .role(Role::ListItem)
                        .when(dragging_row == Some(position), |row| row.opacity(0.4))
                        .on_prepaint(move |bounds, _, cx| {
                            measured.update(cx, |state, _| {
                                state.widths.insert(position, bounds.size.width);
                            })
                        })
                        .on_drag(
                            DraggedRow {
                                list,
                                from: position,
                            },
                            move |drag, _, _, cx| {
                                let width = started.update(cx, |state, _| {
                                    state.dragging_row = Some(drag.from);
                                    state.row_target = None;
                                    state.widths.get(&drag.from).copied().unwrap_or(px(240.))
                                });
                                let render = render.clone();
                                cx.new(|_| DragPreview { render, width })
                            },
                        )
                        .on_drag_move(
                            move |event: &gpui_kit::DragMoveEvent<DraggedRow>, window, cx| {
                                if !event.bounds.contains(&event.event.position)
                                    || event.drag(cx).list != list
                                {
                                    return;
                                }
                                let after = if horizontal {
                                    event.event.position.x > event.bounds.center().x
                                } else {
                                    event.event.position.y > event.bounds.center().y
                                };
                                let target = Target::Slot {
                                    group: group_index,
                                    slot: index + usize::from(after),
                                };
                                if moved.read(cx).row_target != Some(target) {
                                    moved.update(cx, |state, _| state.row_target = Some(target));
                                    window.refresh();
                                }
                            },
                        )
                        .on_key_down(move |event, window, cx| {
                            let Some(up) = move_key(event, horizontal) else {
                                return;
                            };
                            cx.stop_propagation();
                            let Some(to) =
                                keyboard_target(position, up, &lens_for_keys, &collapsed_for_keys)
                            else {
                                return;
                            };
                            let movement = SortableMove { from: position, to };
                            let len = if to.group == position.group {
                                lens_for_keys[to.group]
                            } else {
                                lens_for_keys[to.group] + 1
                            };
                            keyed.update(cx, |state, _| {
                                state.announcement = format!(
                                    "{label} moved to position {} of {len} in {}",
                                    to.index + 1,
                                    labels[to.group]
                                )
                                .into();
                            });
                            if let Some(handler) = &on_reorder {
                                handler(&movement, window, cx);
                            }
                        })
                        .child((row.render)(window, cx))
                        .when(line_before, |row| {
                            row.child(drop_line(true, horizontal, &theme))
                        })
                        .when(line_after, |row| {
                            row.child(drop_line(false, horizontal, &theme))
                        });
                    rows = rows.child(element);
                }
            }

            // The group: a drop target for group drags, highlighted when a row
            // would drop into it.
            // No ID: rows sit directly under the list in GPUI's element
            // paths, so a row keeps its state and focus in any group.
            let mut container = div()
                .relative()
                .flex()
                .when(horizontal, |container| container.flex_row().items_center())
                .when(!horizontal, |container| container.flex_col())
                .rounded(theme.radius.md)
                .when(highlighted, |container| container.bg(theme.colors.accent))
                .when(dragging_group == Some(group_index), |container| {
                    container.opacity(0.4)
                })
                .child(header)
                .child(rows);
            if reorder_groups {
                let state = state.clone();
                container = container.on_drag_move(
                    move |event: &gpui_kit::DragMoveEvent<DraggedGroup>, window, cx| {
                        if !event.bounds.contains(&event.event.position)
                            || event.drag(cx).list != list
                        {
                            return;
                        }
                        let after = if horizontal {
                            event.event.position.x > event.bounds.center().x
                        } else {
                            event.event.position.y > event.bounds.center().y
                        };
                        let slot = group_index + usize::from(after);
                        if state.read(cx).group_target != Some(slot) {
                            state.update(cx, |state, _| state.group_target = Some(slot));
                            window.refresh();
                        }
                    },
                );
                if group_target == Some(group_index) {
                    container = container.child(drop_line(true, horizontal, &theme));
                }
                if group_index + 1 == group_count && group_target == Some(group_count) {
                    container = container.child(drop_line(false, horizontal, &theme));
                }
            }
            root = root.child(container);
        }
        apply_style(root, &self.style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{
        Context, Entity, Modifiers, Render, TestAppContext, VisualTestContext, point,
        test::TestQueryExt as _,
    };

    #[test]
    fn moves_apply_as_a_remove_then_an_insert() {
        let mut groups = vec![vec!["a", "b", "c"], vec!["d"]];
        SortableMove {
            from: SortablePosition::new(0, 0),
            to: SortablePosition::new(0, 2),
        }
        .apply(&mut groups);
        assert_eq!(groups, [vec!["b", "c", "a"], vec!["d"]]);
        SortableMove {
            from: SortablePosition::new(0, 1),
            to: SortablePosition::new(1, 1),
        }
        .apply(&mut groups);
        assert_eq!(groups, [vec!["b", "a"], vec!["d", "c"]]);
        let mut order = vec!["x", "y", "z"];
        move_group(&mut order, 2, 0);
        assert_eq!(order, ["z", "x", "y"]);
    }

    #[test]
    fn a_drop_slot_counts_before_the_row_leaves() {
        let lens = [3, 0];
        let from = SortablePosition::new(0, 0);
        // Below the second row, so it ends second once it has left the top.
        assert_eq!(
            drop_move(from, Target::Slot { group: 0, slot: 2 }, &lens).map(|m| m.to),
            Some(SortablePosition::new(0, 1))
        );
        // Just above or below itself: no move.
        assert_eq!(
            drop_move(from, Target::Slot { group: 0, slot: 0 }, &lens),
            None
        );
        assert_eq!(
            drop_move(from, Target::Slot { group: 0, slot: 1 }, &lens),
            None
        );
        assert_eq!(
            drop_move(from, Target::Into { group: 1 }, &lens).map(|m| m.to),
            Some(SortablePosition::new(1, 0))
        );
    }

    #[test]
    fn the_keyboard_crosses_into_the_next_group() {
        let lens = [2, 3];
        let collapsed = [false, false];
        let at = |g, i| SortablePosition::new(g, i);
        assert_eq!(
            keyboard_target(at(0, 1), false, &lens, &collapsed),
            Some(at(1, 0))
        );
        assert_eq!(
            keyboard_target(at(1, 0), true, &lens, &collapsed),
            Some(at(0, 2))
        );
        assert_eq!(keyboard_target(at(0, 0), true, &lens, &collapsed), None);
        assert_eq!(keyboard_target(at(1, 2), false, &lens, &collapsed), None);
        assert_eq!(
            keyboard_target(at(1, 1), true, &lens, &collapsed),
            Some(at(1, 0))
        );
        // A collapsed group below takes the row at its end.
        assert_eq!(
            keyboard_target(at(0, 1), false, &lens, &[false, true]),
            Some(at(1, 3))
        );
    }

    struct Channels {
        groups: Vec<Vec<&'static str>>,
        names: Vec<&'static str>,
    }

    impl Render for Channels {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let view = cx.entity();
            let group_view = cx.entity();
            div().w(px(240.)).child(
                Sortable::new("channels", "Channels")
                    .reorder_groups(true)
                    .groups(self.groups.iter().enumerate().map(|(g, rows)| {
                        SortableGroup::new(self.names[g], self.names[g]).rows(rows.iter().map(
                            |&name| {
                                SortableRow::new(name, name, move |_, _| {
                                    div()
                                        .h(px(32.))
                                        .debug_selector(move || format!("row-{name}"))
                                        // A control that keys its focus by
                                        // element path, as most rows hold.
                                        .child(
                                            super::super::button::Button::new(
                                                gpui_kit::ElementId::from(format!("{name}-button")),
                                            )
                                            .label(name),
                                        )
                                })
                            },
                        ))
                    }))
                    .on_reorder(move |movement, _, cx| {
                        view.update(cx, |this, cx| {
                            movement.apply(&mut this.groups);
                            cx.notify();
                        })
                    })
                    .on_reorder_group(move |from, to, _, cx| {
                        group_view.update(cx, |this, cx| {
                            move_group(&mut this.groups, from, to);
                            move_group(&mut this.names, from, to);
                            cx.notify();
                        })
                    }),
            )
        }
    }

    fn channels(cx: &mut TestAppContext) -> (Entity<Channels>, &mut VisualTestContext) {
        cx.update(super::super::theme::init);
        let (view, cx) = cx.add_window_view(|_, _| Channels {
            groups: vec![vec!["general", "random"], vec!["design"], vec![]],
            names: vec!["Favorites", "Channels", "Archive"],
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        (view, cx)
    }

    #[gpui_kit::test]
    fn alt_shift_arrows_move_the_focused_row_and_announce_it(cx: &mut TestAppContext) {
        let (view, cx) = channels(cx);
        // Tab past the Favorites header to the general row's button.
        cx.update(|window, cx| {
            window.activate_window();
            window.focus_next(cx);
            window.focus_next(cx);
            window.draw(cx).clear(cx);
            assert_eq!(window.find_by_label("general").focused(), Some(true));
        });
        cx.simulate_keystrokes("alt-shift-down");
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.update(|_, cx| {
            assert_eq!(
                view.read(cx).groups,
                [vec!["random", "general"], vec!["design"], vec![]]
            );
        });
        cx.simulate_keystrokes("alt-shift-down");
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.update(|window, cx| {
            let this = view.read(cx);
            assert_eq!(
                this.groups,
                [vec!["random"], vec!["general", "design"], vec![]]
            );
            // The row kept its focus across the group change.
            assert_eq!(window.find_by_label("general").focused(), Some(true));
            let status = window.find_by_label("general moved to position 1 of 2 in Channels");
            assert_eq!(status.role(), Some(Role::Status));
        });
        cx.simulate_keystrokes("alt-shift-up alt-shift-up");
        cx.update(|_, cx| {
            assert_eq!(
                view.read(cx).groups,
                [vec!["general", "random"], vec!["design"], vec![]]
            );
        });
    }

    #[gpui_kit::test]
    fn a_row_dropped_on_an_empty_group_goes_into_it(cx: &mut TestAppContext) {
        let (view, cx) = channels(cx);
        let random = cx.debug_bounds("row-random").expect("random is drawn");
        let archive = cx
            .debug_bounds("header-Archive")
            .expect("the empty group is drawn");
        drag(cx, random.center(), archive.center());
        cx.update(|_, cx| {
            assert_eq!(
                view.read(cx).groups,
                [vec!["general"], vec!["design"], vec!["random"]]
            );
        });
    }

    #[gpui_kit::test]
    fn groups_reorder_by_header_drag_and_by_keyboard(cx: &mut TestAppContext) {
        let (view, cx) = channels(cx);
        let favorites = cx.debug_bounds("header-Favorites").expect("drawn");
        let archive = cx.debug_bounds("header-Archive").expect("drawn");
        drag(
            cx,
            favorites.center(),
            point(archive.center().x, archive.bottom() - px(2.)),
        );
        cx.update(|_, cx| {
            assert_eq!(view.read(cx).names, ["Channels", "Archive", "Favorites"]);
            assert_eq!(
                view.read(cx).groups,
                [vec!["design"], vec![], vec!["general", "random"]]
            );
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        // Tab to the first header, Channels, and move it down one place.
        cx.update(|window, cx| {
            window.activate_window();
            window.blur(cx);
            window.focus_next(cx);
        });
        cx.simulate_keystrokes("alt-shift-down");
        cx.update(|_, cx| {
            assert_eq!(view.read(cx).names, ["Archive", "Channels", "Favorites"]);
        });
    }

    struct Bookmarks {
        items: Vec<Vec<&'static str>>,
    }

    impl Render for Bookmarks {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let view = cx.entity();
            div().w(px(480.)).child(
                Sortable::new("bookmarks", "Bookmarks")
                    .horizontal()
                    .group(
                        SortableGroup::new("bar", "Bookmarks")
                            .header(|_, _| div())
                            .rows(self.items[0].iter().map(|&name| {
                                SortableRow::new(name, name, move |_, _| {
                                    div()
                                        .w(px(80.))
                                        .h(px(28.))
                                        .debug_selector(move || format!("chip-{name}"))
                                        .child(
                                            super::super::button::Button::new(
                                                gpui_kit::ElementId::from(format!("{name}-chip")),
                                            )
                                            .label(name),
                                        )
                                })
                            })),
                    )
                    .on_reorder(move |movement, _, cx| {
                        view.update(cx, |this, cx| {
                            movement.apply(&mut this.items);
                            cx.notify();
                        })
                    }),
            )
        }
    }

    #[gpui_kit::test]
    fn a_horizontal_list_moves_by_left_and_right_and_by_drag(cx: &mut TestAppContext) {
        cx.update(super::super::theme::init);
        let (view, cx) = cx.add_window_view(|_, _| Bookmarks {
            items: vec![vec!["Docs", "Board", "Wiki"]],
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let docs = cx.debug_bounds("chip-Docs").expect("drawn");
        let board = cx.debug_bounds("chip-Board").expect("drawn");
        assert_eq!(docs.top(), board.top(), "chips share a row");
        assert!(board.left() > docs.right());
        cx.update(|window, cx| {
            window.activate_window();
            window.focus_next(cx);
            window.draw(cx).clear(cx);
            assert_eq!(window.find_by_label("Docs").focused(), Some(true));
        });
        cx.simulate_keystrokes("alt-shift-down");
        cx.update(|_, cx| assert_eq!(view.read(cx).items[0], ["Docs", "Board", "Wiki"]));
        cx.simulate_keystrokes("alt-shift-right");
        cx.update(|window, cx| {
            assert_eq!(view.read(cx).items[0], ["Board", "Docs", "Wiki"]);
            window.draw(cx).clear(cx);
            window.find_by_label("Docs moved to position 2 of 3 in Bookmarks");
        });
        let board = cx.debug_bounds("chip-Board").expect("drawn");
        let wiki = cx.debug_bounds("chip-Wiki").expect("drawn");
        drag(cx, board.center(), point(wiki.right() - px(4.), wiki.center().y));
        cx.update(|_, cx| assert_eq!(view.read(cx).items[0], ["Docs", "Wiki", "Board"]));
    }

    fn drag(
        cx: &mut VisualTestContext,
        from: gpui_kit::Point<Pixels>,
        to: gpui_kit::Point<Pixels>,
    ) {
        let left = gpui_kit::MouseButton::Left;
        cx.simulate_mouse_down(from, left, Modifiers::none());
        cx.simulate_mouse_move(from + point(px(0.), px(10.)), left, Modifiers::none());
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.simulate_mouse_move(to, left, Modifiers::none());
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.simulate_mouse_move(to, left, Modifiers::none());
        cx.simulate_mouse_up(to, left, Modifiers::none());
        cx.update(|window, cx| window.draw(cx).clear(cx));
    }

    #[gpui_kit::test]
    fn dragging_a_row_moves_it_within_and_across_groups(cx: &mut TestAppContext) {
        let (view, cx) = channels(cx);
        let general = cx.debug_bounds("row-general").expect("general is drawn");
        let design = cx.debug_bounds("row-design").expect("design is drawn");
        // Drag general below design, the lower half of the design row.
        cx.simulate_mouse_down(
            general.center(),
            gpui_kit::MouseButton::Left,
            Modifiers::none(),
        );
        cx.simulate_mouse_move(
            general.center() + point(px(0.), px(10.)),
            gpui_kit::MouseButton::Left,
            Modifiers::none(),
        );
        let below_design = point(design.center().x, design.bottom() - px(4.));
        cx.simulate_mouse_move(below_design, gpui_kit::MouseButton::Left, Modifiers::none());
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.simulate_mouse_move(below_design, gpui_kit::MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_up(below_design, gpui_kit::MouseButton::Left, Modifiers::none());
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.update(|_, cx| {
            assert_eq!(
                view.read(cx).groups,
                [vec!["random"], vec!["design", "general"], vec![]]
            );
        });
    }
}
