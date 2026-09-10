//! Sidebar navigation composed from GPUI Kit buttons, Scroll Area and Tooltip.
//! Applications own navigation and collapsed state; this component owns no router.

use super::{scroll_area::*, theme::UiTheme, tooltip::text_tooltip};
use gpui_kit::base::Button as BaseButton;
use gpui_kit::{
    AnyElement, App, ClickEvent, Div, ElementId, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement, RenderOnce, Role, SharedString, StatefulInteractiveElement as _, Styled, Text,
    Window, div, prelude::FluentBuilder as _, px,
};
use std::{rc::Rc, sync::Arc};

type ActivateHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
fn part(id: &ElementId, name: &'static str) -> ElementId {
    ElementId::NamedChild(Arc::new(id.clone()), name.into())
}

/// A labelled sidebar with fixed header/footer and scrollable navigation content.
/// Width and collapse state belong to its parent layout.
#[derive(IntoElement)]
pub struct Sidebar {
    style: gpui_kit::StyleRefinement,
    id: ElementId,
    label: SharedString,
    header: Option<AnyElement>,
    content_padding: Option<gpui_kit::Pixels>,
    footer: Option<AnyElement>,
    children: Vec<AnyElement>,
}
impl Sidebar {
    /// Creates a navigation region with a caller-owned ID and accessible name.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            style: gpui_kit::StyleRefinement::default(),
            id: id.into(),
            label: label.into(),
            header: None,
            content_padding: None,
            footer: None,
            children: vec![],
        }
    }
    /// Override the content inset, for example zero for full-width mail rows.
    /// Header and footer retain their standard inset.
    pub fn content_padding(mut self, padding: gpui_kit::Pixels) -> Self {
        self.content_padding = Some(padding.max(px(0.)));
        self
    }
    /// Sets content that remains above the scrollable navigation.
    pub fn header(mut self, child: impl IntoElement) -> Self {
        self.header = Some(child.into_any_element());
        self
    }
    /// Sets content that remains below the scrollable navigation.
    pub fn footer(mut self, child: impl IntoElement) -> Self {
        self.footer = Some(child.into_any_element());
        self
    }
}
impl ParentElement for Sidebar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}
impl RenderOnce for Sidebar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let spacing = theme.spacing.unit;
        let text_scale = theme.text_scale;
        let color = theme.colors.sidebar;
        let padding = self.content_padding.unwrap_or(spacing * 2_f32);
        let content = ScrollArea::new(part(&self.id, "scroll"))
            .size_full()
            .aria_label(self.label.clone())
            .child(
                div()
                    .w_full()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .p(padding)
                    .children(self.children),
            );
        div()
            .id(self.id)
            .role(Role::Navigation)
            .aria_label(self.label)
            .size_full()
            .flex()
            .flex_col()
            .min_w_0()
            .min_h_0()
            .bg(color)
            .text_color(theme.colors.sidebar_foreground)
            .font_family(theme.fonts.body)
            .text_size(px(14.) * text_scale)
            .line_height(px(20.) * text_scale)
            .when_some(self.header, |el, header| {
                el.child(div().flex_shrink_0().p(spacing * 2_f32).child(header))
            })
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .overflow_hidden()
                    .child(content),
            )
            .when_some(self.footer, |el, footer| {
                el.child(div().flex_shrink_0().p(spacing * 2_f32).child(footer))
            })
            .map(|base| super::theme::apply_style(base, &self.style))
    }
}

/// A sidebar section label. Omit it in an icon rail; keep item accessible names.
pub fn sidebar_group_label(label: impl Into<SharedString>, cx: &App) -> Div {
    let theme = UiTheme::read(cx).clone();
    let spacing = theme.spacing.unit;
    let text_scale = theme.text_scale;
    div()
        .flex()
        .items_center()
        .h(spacing * 8_f32)
        .px(spacing * 2_f32)
        .text_size(px(12.) * text_scale)
        .line_height(px(16.) * text_scale)
        .font_weight(FontWeight::MEDIUM)
        .text_color(UiTheme::read(cx).colors.sidebar_foreground.opacity(0.7))
        .child(label.into())
}

/// A navigation action with selected/disabled styling and an optional icon/badge.
/// Collapsed items keep their accessible name and expose a tooltip. Keep item IDs
/// stable across rail/expanded layouts so focus survives the change.
#[derive(IntoElement)]
pub struct SidebarItem {
    style: gpui_kit::StyleRefinement,
    id: ElementId,
    label: SharedString,
    selected: bool,
    disabled: bool,
    collapsed: bool,
    icon: Option<AnyElement>,
    trailing: Option<AnyElement>,
    size: SidebarItemSize,
    outline: bool,
    expanded: Option<bool>,
    on_activate: Option<ActivateHandler>,
}
impl SidebarItem {
    /// Creates a navigation action. Its label is also used for accessibility and tooltips.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            style: gpui_kit::StyleRefinement::default(),
            id: id.into(),
            label: label.into(),
            selected: false,
            disabled: false,
            collapsed: false,
            icon: None,
            trailing: None,
            size: SidebarItemSize::Default,
            outline: false,
            expanded: None,
            on_activate: None,
        }
    }
    /// Marks the current destination with shadcn's active background and medium weight.
    pub fn selected(mut self, value: bool) -> Self {
        self.selected = value;
        self
    }
    /// Prevents activation using the shared GPUI Kit button guard.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    /// Shows only the icon (or first character if absent), retaining a full-label tooltip.
    pub fn collapsed(mut self, value: bool) -> Self {
        self.collapsed = value;
        self
    }
    /// Sets a caller-rendered leading icon.
    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }
    /// Adds noninteractive trailing content, such as a count. Separate actions belong beside the item.
    pub fn trailing(mut self, child: impl IntoElement) -> Self {
        self.trailing = Some(child.into_any_element());
        self
    }
    /// Set the menu row size.
    pub fn size(mut self, size: SidebarItemSize) -> Self {
        self.size = size;
        self
    }
    /// Use a background surface with a sidebar-colored outline.
    pub fn outline(mut self, outline: bool) -> Self {
        self.outline = outline;
        self
    }
    /// Announces disclosure state when this item controls a nested menu.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = Some(expanded);
        self
    }
    /// Emits activation without owning routing or selection state.
    pub fn on_activate(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_activate = Some(Rc::new(handler));
        self
    }
}
impl RenderOnce for SidebarItem {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let spacing = theme.spacing.unit;
        let selected = self.selected;
        let size = self.size;
        let outline = self.outline;
        let collapsed = self.collapsed;
        let mut accessible = self.label.to_string();
        if selected {
            accessible.push_str(", current destination");
        }
        if self.disabled {
            accessible.push_str(", unavailable");
        }
        if let Some(expanded) = self.expanded {
            accessible.push_str(if expanded {
                ", expanded"
            } else {
                ", collapsed"
            });
        }
        let icon = self.icon.or_else(|| {
            collapsed.then(|| {
                Text::new_inaccessible(self.label.chars().next().unwrap_or(' ').to_string().into())
                    .into_any_element()
            })
        });
        let tooltip_label = self.label.clone();
        BaseButton::new(part(&self.id, "button"))
            .accessibility_label(accessible)
            .disabled(self.disabled)
            .map(|base| {
                let base = {
                    let mut base = base;
                    let label = tooltip_label.clone();
                    if collapsed {
                        base.interactivity()
                            .tooltip(move |_, cx| text_tooltip(label.clone(), cx));
                    }
                    base.relative()
                        .flex()
                        .w_full()
                        .h(spacing
                            * if collapsed {
                                8_f32
                            } else {
                                match size {
                                    SidebarItemSize::Small => 7.,
                                    SidebarItemSize::Default => 8.,
                                    SidebarItemSize::Large => 12.,
                                }
                            })
                        .text_size(
                            px(if size == SidebarItemSize::Small {
                                12.
                            } else {
                                14.
                            }) * theme.text_scale,
                        )
                        .line_height(
                            px(if size == SidebarItemSize::Small {
                                16.
                            } else {
                                20.
                            }) * theme.text_scale,
                        )
                        .font_family(theme.fonts.body.clone())
                        .font_weight(FontWeight::NORMAL)
                        .when(outline, |el| {
                            el.border_1().border_color(theme.colors.sidebar_border)
                        })
                        .min_w_0()
                        .flex_shrink_0()
                        .items_center()
                        .gap(spacing * 2_f32)
                        .px(spacing * 2_f32)
                        .rounded(theme.radius.md)
                        .text_color(if selected {
                            theme.colors.sidebar_accent_foreground
                        } else {
                            theme.colors.sidebar_foreground
                        })
                        .bg(if selected {
                            theme.colors.sidebar_accent
                        } else if outline {
                            theme.colors.background
                        } else {
                            theme.colors.sidebar
                        })
                        .when(selected, |el| el.font_weight(FontWeight::MEDIUM))
                        .when(collapsed, |el| el.w(spacing * 8_f32).justify_center())
                        .when(self.disabled, |el| el.opacity(0.5).cursor_not_allowed())
                        .when(!self.disabled, |el| {
                            el.cursor_pointer().hover(|style| {
                                style
                                    .bg(theme.colors.sidebar_accent)
                                    .text_color(theme.colors.sidebar_accent_foreground)
                            })
                        })
                        .focus_visible(|style| {
                            style
                                .border_color(theme.colors.sidebar_ring)
                                .shadow(theme.focus_ring())
                        })
                };
                super::theme::apply_style(base, &self.style)
            })
            .on_click(move |event, window, cx| {
                if let Some(handler) = &self.on_activate {
                    handler(event, window, cx);
                }
            })
            .when_some(icon, |el, icon| el.child(div().flex_shrink_0().child(icon)))
            .when(!collapsed, |el| {
                el.child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .child(Text::new_inaccessible(self.label.clone())),
                )
            })
            .when(!collapsed, |el| {
                el.when_some(self.trailing, |el, content| {
                    el.child(div().flex_shrink_0().child(content))
                })
            })
    }
}

impl gpui_kit::Styled for Sidebar {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}

impl gpui_kit::Styled for SidebarItem {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}

/// Which edge contains the navigation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SidebarSide {
    #[default]
    /// Navigation before the content.
    Left,
    /// Navigation after the content.
    Right,
}
/// The three shadcn application layouts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SidebarVariant {
    #[default]
    /// Full-height navigation with an edge border.
    Sidebar,
    /// A rounded, bordered navigation surface.
    Floating,
    /// Content inset into the sidebar-colored shell.
    Inset,
}
/// Desktop collapse behavior. Mobile uses a modal sheet except for `None`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SidebarCollapsible {
    #[default]
    /// Completely hide desktop navigation when closed.
    Offcanvas,
    /// Keep a compact icon rail when closed.
    Icon,
    /// Always display full navigation, including on narrow windows.
    None,
}
/// Caller-owned state. Desktop preference survives opening and closing mobile navigation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SidebarState {
    /// Desktop expanded preference.
    pub open: bool,
    /// Whether the mobile sheet is open.
    pub mobile_open: bool,
}
impl Default for SidebarState {
    fn default() -> Self {
        Self {
            open: true,
            mobile_open: false,
        }
    }
}
impl SidebarState {
    /// Toggle the state for the active viewport.
    pub fn toggle(&mut self, mobile: bool) {
        if mobile {
            self.mobile_open = !self.mobile_open;
        } else {
            self.open = !self.open;
        }
    }
    /// Whether navigation should render icons without labels.
    pub fn icon_collapsed(self, mobile: bool, mode: SidebarCollapsible) -> bool {
        !mobile && !self.open && mode == SidebarCollapsible::Icon
    }
}

/// Uses the available GPUI window width, including an embedded preview's viewport.
/// Pass the same result to the layout and menu composition. Applications with a
/// narrower nested pane can instead supply their own breakpoint decision.
pub fn sidebar_is_mobile(window: &Window) -> bool {
    window.viewport_size().width < px(768.)
}

struct SidebarFocus {
    mobile_open: bool,
    return_to: Option<gpui_kit::FocusHandle>,
    navigation: gpui_kit::FocusHandle,
    content: gpui_kit::FocusHandle,
    sheet: gpui_kit::FocusHandle,
}

type StateHandler = Rc<dyn Fn(&SidebarState, &mut Window, &mut App)>;

/// A controlled application shell. Supplies side, variant, animated desktop width,
/// keyboard toggle and a focus-trapped mobile sheet. Render menu labels using
/// `state.icon_collapsed(mobile, mode)`; the mobile sheet always gets full labels.
#[derive(IntoElement)]
pub struct SidebarLayout {
    id: ElementId,
    state: SidebarState,
    mobile: bool,
    side: SidebarSide,
    variant: SidebarVariant,
    collapsible: SidebarCollapsible,
    width: Option<gpui_kit::Pixels>,
    icon_width: Option<gpui_kit::Pixels>,
    mobile_width: Option<gpui_kit::Pixels>,
    rail: bool,
    sidebar: AnyElement,
    content: AnyElement,
    on_change: StateHandler,
    style: gpui_kit::StyleRefinement,
}
impl SidebarLayout {
    /// Compose navigation and application content with controlled state.
    pub fn new(
        id: impl Into<ElementId>,
        state: SidebarState,
        mobile: bool,
        sidebar: impl IntoElement,
        content: impl IntoElement,
        on_change: impl Fn(&SidebarState, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            state,
            mobile,
            side: SidebarSide::Left,
            variant: SidebarVariant::Sidebar,
            collapsible: SidebarCollapsible::Offcanvas,
            width: None,
            icon_width: None,
            mobile_width: None,
            rail: false,
            sidebar: sidebar.into_any_element(),
            content: content.into_any_element(),
            on_change: Rc::new(on_change),
            style: Default::default(),
        }
    }
    /// Add a pointer rail along the navigation edge. Keep a visible keyboard trigger.
    pub fn rail(mut self, rail: bool) -> Self {
        self.rail = rail;
        self
    }
    /// Set the navigation edge.
    pub fn side(mut self, side: SidebarSide) -> Self {
        self.side = side;
        self
    }
    /// Set the application layout.
    pub fn variant(mut self, variant: SidebarVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Set desktop collapse behavior.
    pub fn collapsible(mut self, mode: SidebarCollapsible) -> Self {
        self.collapsible = mode;
        self
    }
    /// Icon rail width before floating/inset gutters; defaults to a themed 48px.
    pub fn icon_width(mut self, width: gpui_kit::Pixels) -> Self {
        self.icon_width = Some(width);
        self
    }
    /// Mobile sheet width; defaults to a themed 288px and cannot exceed its viewport.
    pub fn mobile_width(mut self, width: gpui_kit::Pixels) -> Self {
        self.mobile_width = Some(width);
        self
    }
    /// Expanded width. Invalid values fall back to the themed 256px default.
    pub fn width(mut self, width: gpui_kit::Pixels) -> Self {
        self.width = Some(width);
        self
    }
}
impl Styled for SidebarLayout {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        &mut self.style
    }
}
impl RenderOnce for SidebarLayout {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        use super::dialog::*;
        use super::theme::transition_value;
        let theme = UiTheme::read(cx).clone();
        let s = theme.spacing.unit;
        let mobile = self.mobile && self.collapsible != SidebarCollapsible::None;
        let inset = self.variant == SidebarVariant::Inset && !mobile;
        let floating = self.variant == SidebarVariant::Floating && !mobile;
        let padded = inset || floating;
        let valid_width = |width: Option<gpui_kit::Pixels>, default| {
            width
                .filter(|w| f32::from(*w).is_finite() && *w > px(0.))
                .unwrap_or(default)
        };
        let expanded = valid_width(self.width, s * 64_f32);
        let icon_width = valid_width(self.icon_width, s * 12_f32).max(s * 12_f32);
        let mobile_width = valid_width(self.mobile_width, s * 72_f32);
        let collapsed = !self.state.open && self.collapsible != SidebarCollapsible::None;
        let hidden = mobile || (collapsed && self.collapsible == SidebarCollapsible::Offcanvas);
        let target = if hidden {
            px(0.)
        } else if collapsed {
            icon_width + if padded { s * 4_f32 } else { px(0.) }
        } else {
            expanded + if padded { s * 4_f32 } else { px(0.) }
        };
        let width = px(transition_value(
            part(&self.id, "width"),
            target.into(),
            theme.motion.normal,
            window,
            cx,
        ));
        let focus = window.use_keyed_state(part(&self.id, "focus"), cx, |_, cx| SidebarFocus {
            mobile_open: false,
            return_to: None,
            navigation: cx.focus_handle(),
            content: cx.focus_handle(),
            sheet: cx.focus_handle(),
        });
        let (navigation_focus, content_focus, sheet_focus) = focus.update(cx, |focus, cx| {
            let sheet_open = mobile && self.state.mobile_open;
            if sheet_open && !focus.mobile_open {
                focus.return_to = window.focused(cx);
                // Base's controlled open reconciliation does not request focus.
                // Focus a descendant that participates in the popup's key context.
                focus.sheet.focus(window, cx);
            } else if !sheet_open && focus.mobile_open {
                if let Some(handle) = focus.return_to.take() {
                    handle.focus(window, cx);
                }
            } else if hidden && focus.navigation.contains_focused(window, cx) {
                focus.content.focus(window, cx);
            }
            focus.mobile_open = sheet_open;
            (
                focus.navigation.clone(),
                focus.content.clone(),
                focus.sheet.clone(),
            )
        });
        let state = self.state;
        let change = self.on_change.clone();
        let main = div()
            .track_focus(&content_focus)
            .flex_1()
            .min_w_0()
            .min_h_0()
            .overflow_hidden()
            .bg(theme.colors.background)
            .when(inset, |el| {
                el.m(s * 2_f32)
                    .when(!collapsed && self.side == SidebarSide::Left, |el| {
                        el.ml(px(0.))
                    })
                    .when(!collapsed && self.side == SidebarSide::Right, |el| {
                        el.mr(px(0.))
                    })
                    .rounded(theme.radius.xl)
                    .shadow(theme.shadows.sm.clone())
            })
            .child(self.content);
        let mut shell = div()
            .id(self.id.clone())
            .size_full()
            .min_w_0()
            .min_h_0()
            .flex()
            .bg(theme.colors.sidebar)
            .font_family(theme.fonts.body.clone())
            .text_size(px(14.) * theme.text_scale)
            .line_height(px(20.) * theme.text_scale)
            .when(self.side == SidebarSide::Right, |el| el.flex_row_reverse())
            .on_key_down(move |event, window, cx| {
                if event.keystroke.key == "b"
                    && (event.keystroke.modifiers.platform || event.keystroke.modifiers.control)
                    && self.collapsible != SidebarCollapsible::None
                {
                    let mut next = state;
                    next.toggle(mobile);
                    change(&next, window, cx);
                    window.prevent_default();
                    cx.stop_propagation();
                }
            });
        if mobile {
            let id = part(&self.id, "sheet");
            let popup_focus = modal_focus::prepare(id.clone(), self.state.mobile_open, window, cx);
            let focus = modal_focus::ModalFocus::new(id.clone());
            let popup = focus
                .trap(div(), true)
                .id(id)
                .track_focus(&popup_focus)
                .aria_label("Navigation")
                .occlude()
                .h_full()
                .w(mobile_width)
                .max_w_full()
                .flex()
                .flex_col()
                .bg(theme.colors.sidebar)
                .text_color(theme.colors.sidebar_foreground)
                .child(focus.boundary(false))
                .child(focus.boundary(true))
                .child(
                    div()
                        .size_full()
                        .track_focus(&sheet_focus)
                        .child(self.sidebar),
                );
            let change = self.on_change;
            let root = Dialog::new(cx)
                .open(self.state.mobile_open)
                .on_ok(|_, _, _| false)
                .on_open_change(move |open, _, window, cx| {
                    change(
                        &SidebarState {
                            mobile_open: open,
                            ..state
                        },
                        window,
                        cx,
                    );
                })
                .backdrop(dialog_backdrop(cx))
                .popup(
                    div()
                        .absolute()
                        .inset_0()
                        .flex()
                        .when(self.side == SidebarSide::Right, |el| el.justify_end())
                        .child(popup),
                );
            shell = shell.child(main).child(root);
        } else {
            let navigation = div()
                .track_focus(&navigation_focus)
                .w(width)
                .h_full()
                .flex_shrink_0()
                .overflow_hidden()
                .when(padded, |el| el.p(s * 2_f32))
                .when(!padded, |el| {
                    el.border_color(theme.colors.sidebar_border)
                        .when(self.side == SidebarSide::Left, |el| el.border_r_1())
                        .when(self.side == SidebarSide::Right, |el| el.border_l_1())
                })
                // Fully hidden navigation is removed immediately from focus traversal.
                .when(!hidden, |el| {
                    el.child(
                        div()
                            .size_full()
                            .overflow_hidden()
                            .when(floating, |el| {
                                el.rounded(theme.radius.lg)
                                    .border_1()
                                    .border_color(theme.colors.sidebar_border)
                                    .shadow(theme.shadows.sm.clone())
                            })
                            .child(self.sidebar),
                    )
                });
            shell = shell.child(navigation).child(main);
            if self.rail && self.collapsible != SidebarCollapsible::None {
                let change = self.on_change;
                shell = shell.relative().child(
                    sidebar_rail(
                        part(&self.id, "rail"),
                        move |_, window, cx| {
                            let mut state = state;
                            state.toggle(false);
                            change(&state, window, cx);
                        },
                        cx,
                    )
                    .absolute()
                    .top_0()
                    .when(self.side == SidebarSide::Left, |el| {
                        el.left((width - s).max(px(0.)))
                    })
                    .when(self.side == SidebarSide::Right, |el| {
                        el.right((width - s).max(px(0.)))
                    }),
                );
            }
        }
        super::theme::apply_style(shell, &self.style)
    }
}

/// A keyboard-accessible toggle. Share the same callback with the layout and rail.
pub fn sidebar_trigger(
    id: impl Into<ElementId>,
    on_toggle: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    cx: &App,
) -> super::button::Button {
    super::button::Button::new(id)
        .variant(super::button::ButtonVariant::Ghost)
        .size(super::button::ButtonSize::IconSm)
        .aria_label("Toggle sidebar")
        .child(
            gpui_icons::lucide(gpui_icons::LucideIcon::PanelLeft)
                .size(UiTheme::read(cx).spacing.unit * 4_f32)
                .text_color(UiTheme::read(cx).colors.sidebar_foreground),
        )
        .on_click(on_toggle)
}

/// Group container. Header, menu and group action can be composed freely.
pub fn sidebar_group() -> Div {
    div().relative().w_full().min_w_0().flex().flex_col()
}
/// A vertical menu with Nova's contiguous rows.
pub fn sidebar_menu() -> Div {
    div().w_full().min_w_0().flex().flex_col()
}
/// Nested navigation. Omit it when its parent is closed or the sidebar is an icon rail.
pub fn sidebar_menu_sub(cx: &App) -> Div {
    let t = UiTheme::read(cx);
    let s = t.spacing.unit;
    div()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(s)
        .mx(s * 3.5_f32)
        .px(s * 2.5_f32)
        .py(s * 0.5_f32)
        .border_l_1()
        .border_color(t.colors.sidebar_border)
}
/// Noninteractive count that inherits sidebar colors.
pub fn sidebar_badge(label: impl Into<SharedString>, cx: &App) -> Div {
    let t = UiTheme::read(cx);
    let s = t.spacing.unit;
    div()
        .h(s * 5_f32)
        .min_w(s * 5_f32)
        .px(s)
        .flex()
        .items_center()
        .justify_center()
        .rounded(t.radius.md)
        .text_size(px(12.) * t.text_scale)
        .line_height(px(16.) * t.text_scale)
        .font_weight(FontWeight::MEDIUM)
        .child(label.into())
}
/// Separator between navigation groups.
pub fn sidebar_separator(cx: &App) -> Div {
    let t = UiTheme::read(cx);
    div()
        .h(px(1.))
        .my(t.spacing.unit * 2_f32)
        .flex_shrink_0()
        .bg(t.colors.sidebar_border)
}
/// Sidebar-sized text field; input editing and accessibility remain in the existing Input.
pub fn sidebar_input(
    state: &gpui_kit::Entity<super::input::InputState>,
    cx: &App,
) -> super::input::Input {
    let t = UiTheme::read(cx);
    super::input::Input::new(state)
        .h(t.spacing.unit * 8_f32)
        .w_full()
        .bg(t.colors.background)
}
/// A stable loading placeholder, with optional icon. No random widths or idle animation.
pub fn sidebar_menu_skeleton(show_icon: bool, cx: &App) -> Div {
    let t = UiTheme::read(cx);
    let s = t.spacing.unit;
    div()
        .h(s * 8_f32)
        .px(s * 2_f32)
        .flex()
        .items_center()
        .gap(s * 2_f32)
        .when(show_icon, |el| {
            el.child(
                div()
                    .size(s * 4_f32)
                    .rounded(t.radius.md)
                    .bg(t.colors.muted),
            )
        })
        .child(
            div()
                .h(s * 4_f32)
                .w(s * 28_f32)
                .rounded(t.radius.md)
                .bg(t.colors.muted),
        )
}

/// Nova menu row sizes. Collapsed rows always use the icon size.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SidebarItemSize {
    /// 28px high, 12px text at the default theme scale.
    Small,
    /// 32px high, 14px text.
    #[default]
    Default,
    /// 48px high, suitable for workspace and account controls.
    Large,
}
/// A named secondary action. Place beside the menu button, never inside it.
pub fn sidebar_menu_action(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    cx: &App,
) -> super::button::Button {
    let t = UiTheme::read(cx);
    super::button::Button::new(id)
        .aria_label(label)
        .variant(super::button::ButtonVariant::Ghost)
        .size(super::button::ButtonSize::IconXs)
        .text_color(t.colors.sidebar_foreground)
}
/// A narrow pointer target along the navigation edge. Keep a visible trigger for keyboard users.
pub fn sidebar_rail(
    id: impl Into<ElementId>,
    on_toggle: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    cx: &App,
) -> gpui_kit::Stateful<Div> {
    let border = UiTheme::read(cx).colors.sidebar_border;
    div()
        .id(id)
        .w(px(8.))
        .h_full()
        .flex_shrink_0()
        .cursor_col_resize()
        .hover(move |el| el.bg(border))
        .on_click(on_toggle)
}

/// A sidebar menu with caller-owned state and a readable trigger label.
pub fn sidebar_dropdown(
    state: &gpui_kit::Entity<super::menu::MenuState>,
    label: impl Into<SharedString>,
    cx: &App,
) -> super::menu::Menu {
    super::menu::Menu::new(state, label)
        .w_full()
        .text_color(UiTheme::read(cx).colors.sidebar_foreground)
}

#[cfg(test)]
mod sidebar_tests {
    use super::*;
    #[test]
    fn mobile_and_desktop_state_are_independent() {
        let mut state = SidebarState::default();
        state.toggle(false);
        assert!(state.icon_collapsed(false, SidebarCollapsible::Icon));
        state.toggle(true);
        assert!(!state.open && state.mobile_open);
        assert!(!state.icon_collapsed(true, SidebarCollapsible::Icon));
        assert!(!state.icon_collapsed(false, SidebarCollapsible::None));
        state.toggle(true);
        assert!(!state.open && !state.mobile_open);
    }
    #[test]
    fn controlled_sheet_takes_focus_wraps_tabs_and_restores_the_opener() {
        use gpui_kit::{
            AppContext as _, Context, FocusHandle, Render, TestAppContext, VisualTestContext,
        };
        struct View {
            state: SidebarState,
            opener: FocusHandle,
            first: FocusHandle,
            last: FocusHandle,
        }
        impl Render for View {
            fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                let navigation = Sidebar::new("test.nav", "Navigation")
                    .child(
                        div()
                            .id("test.first")
                            .track_focus(&self.first.clone().tab_stop(true))
                            .child("First"),
                    )
                    .child(
                        div()
                            .id("test.last")
                            .track_focus(&self.last.clone().tab_stop(true))
                            .child("Last"),
                    );
                let main = div()
                    .id("test.opener")
                    .track_focus(&self.opener.clone().tab_stop(true))
                    .child("Open");
                SidebarLayout::new(
                    "test.layout",
                    self.state,
                    true,
                    navigation,
                    main,
                    cx.listener(|this, state: &SidebarState, _, cx| {
                        this.state = *state;
                        cx.notify();
                    }),
                )
            }
        }
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            super::super::theme::init(cx);
            UiTheme::set(cx, UiTheme::neutral_light());
        });
        let window = cx.add_window(|_, cx| View {
            state: SidebarState::default(),
            opener: cx.focus_handle(),
            first: cx.focus_handle(),
            last: cx.focus_handle(),
        });
        let mut visual = VisualTestContext::from_window(window.into(), &cx);
        cx.update_window(window.into(), |view, window, cx| {
            let view = view.downcast::<View>().unwrap();
            window.draw(cx).clear(cx);
            view.update(cx, |view, cx| {
                view.opener.focus(window, cx);
                view.state.mobile_open = true;
                cx.notify();
            });
            window.draw(cx).clear(cx);
        })
        .unwrap();
        for (key, first) in [
            ("tab", true),
            ("tab", false),
            ("tab", true),
            ("shift-tab", false),
        ] {
            visual.simulate_keystrokes(key);
            cx.update_window(window.into(), |view, window, cx| {
                window.draw(cx).clear(cx);
                let view = view.downcast::<View>().unwrap();
                let view = view.read(cx);
                let expected = if first { &view.first } else { &view.last };
                assert!(
                    expected.is_focused(window),
                    "{key}: expected {expected:?}, got {:?}",
                    window.focused(cx)
                );
            })
            .unwrap();
        }
        visual.simulate_keystrokes("escape");
        cx.update_window(window.into(), |view, window, cx| {
            window.draw(cx).clear(cx);
            let view = view.downcast::<View>().unwrap();
            let view = view.read(cx);
            assert!(!view.state.mobile_open);
            assert!(view.state.open);
            assert!(view.opener.is_focused(window));
        })
        .unwrap();
    }
}
