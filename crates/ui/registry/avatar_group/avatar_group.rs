//! Nova Avatar Group: overlapping avatars with a `+N` overflow chip.
//!
//! Visual reference: shadcn/ui's avatar stack (`-space-x-2` with a
//! background ring). Each face is a gpuicn `Avatar`; the group adds the
//! overlap, the ring in the surface color, the overflow chip, a hover
//! tooltip and one accessible name that lists the people.

use gpui_kit::{
    App, ElementId, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _,
    RenderOnce, Rgba, Role, SharedString, StatefulInteractiveElement as _, StyleRefinement,
    Styled, Window, div, prelude::FluentBuilder as _,
};

use gpui_kit::base::TestSupportExt as _;

use super::avatar::{Avatar, AvatarSize};
use super::theme::{UiTheme, apply_style};
use super::tooltip::text_tooltip;

/// A row of overlapping avatars, for "acknowledged by" or "seen by" stacks.
///
/// Add people with `person`; the first `max` show as faces and the rest
/// collapse into a `+N` chip. Hovering shows every name, and the group is
/// read as one image named with its label and the names.
#[derive(IntoElement)]
pub struct AvatarGroup {
    id: ElementId,
    people: Vec<(SharedString, Avatar)>,
    max: usize,
    size: AvatarSize,
    ring_color: Option<Rgba>,
    label: Option<SharedString>,
    style: StyleRefinement,
}

impl AvatarGroup {
    /// Creates an empty group with a stable caller-owned ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            people: Vec::new(),
            max: 4,
            size: AvatarSize::Sm,
            ring_color: None,
            label: None,
            style: StyleRefinement::default(),
        }
    }

    /// Adds a person: their name, read and shown in the tooltip, and their
    /// avatar. The group sets the avatar's size.
    pub fn person(mut self, name: impl Into<SharedString>, avatar: Avatar) -> Self {
        self.people.push((name.into(), avatar));
        self
    }

    /// Shows at most `max` faces (four by default); the rest become `+N`.
    pub fn max(mut self, max: usize) -> Self {
        self.max = max.max(1);
        self
    }

    /// Sets the avatar size, `Sm` by default.
    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = size;
        self
    }

    /// Sets the ring between faces. Match the surface under the group; the
    /// default is the `background` token.
    pub fn ring_color(mut self, color: impl Into<Rgba>) -> Self {
        self.ring_color = Some(color.into());
        self
    }

    /// Sets the words read before the names, such as "Acknowledged by".
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// "Ana", "Ana and Ben", "Ana, Ben and Cy".
pub fn list_names(names: &[SharedString]) -> String {
    match names {
        [] => String::new(),
        [only] => only.to_string(),
        [rest @ .., last] => format!(
            "{} and {last}",
            rest.iter()
                .map(SharedString::as_ref)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

impl Styled for AvatarGroup {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for AvatarGroup {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let colors = theme.colors;
        let ring = self.ring_color.unwrap_or(colors.background);
        let names: Vec<SharedString> = self.people.iter().map(|(name, _)| name.clone()).collect();
        let listed = list_names(&names);
        let label: SharedString = match &self.label {
            Some(label) if !listed.is_empty() => format!("{label} {listed}").into(),
            Some(label) => label.clone(),
            None => listed.clone().into(),
        };
        let total = self.people.len();
        // A lone overflow face is shown instead of a "+1" chip.
        let shown = if total > self.max { self.max - 1 } else { total }.max(1).min(total);
        let hidden = total - shown;
        let side = match self.size {
            AvatarSize::Sm => 24.,
            AvatarSize::Default => 32.,
            AvatarSize::Lg => 40.,
        };
        let face = theme.space(side / 4.);
        let ring_width = theme.space(0.5);
        let overlap = face / 3.;
        let framed = |index: usize, content: gpui_kit::AnyElement| {
            div()
                .flex_shrink_0()
                .rounded_full()
                .p(ring_width)
                .bg(ring)
                .when(index > 0, |frame| frame.ml(-overlap))
                .child(content)
        };
        let faces = self
            .people
            .into_iter()
            .take(shown)
            .enumerate()
            .map(|(index, (_, avatar))| framed(index, avatar.size(self.size).into_any_element()));
        let chip = (hidden > 0).then(|| {
            framed(
                shown,
                div()
                    .size(face)
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(colors.muted)
                    .text_color(colors.muted_foreground)
                    .text_size(theme.text(if self.size == AvatarSize::Sm { 11. } else { 12. }))
                    .font_weight(FontWeight::MEDIUM)
                    .debug_selector(|| "avatar-group-overflow".into())
                    .child(format!("+{hidden}"))
                    .into_any_element(),
            )
        });
        let tooltip = listed.clone();
        let root = div()
            .id(self.id)
            .test_support()
            .role(Role::Image)
            .aria_label(label)
            .flex()
            .items_center()
            .children(faces)
            .children(chip)
            .when(!tooltip.is_empty(), |root| {
                root.tooltip(move |_, cx| text_tooltip(tooltip.clone().into(), cx))
            });
        apply_style(root, &self.style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{Context, Render, TestAppContext, VisualTestContext, test::TestQueryExt as _};

    struct Stack {
        count: usize,
    }

    impl Render for Stack {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let names = ["Ana", "Ben", "Cy", "Dee", "Eve", "Fay"];
            names.into_iter().take(self.count).fold(
                AvatarGroup::new("acks").aria_label("Acknowledged by").max(4),
                |group, name| group.person(name, Avatar::new(name).fallback(&name[..1])),
            )
        }
    }

    fn draw(count: usize, cx: &mut TestAppContext) -> VisualTestContext {
        cx.update(super::super::theme::init);
        let window = cx.add_window(move |_, _| Stack { count });
        let mut cx = VisualTestContext::from_window(window.into(), cx);
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx
    }

    #[gpui_kit::test]
    fn extra_people_collapse_into_a_count_and_every_name_is_read(cx: &mut TestAppContext) {
        let mut cx = draw(6, cx);
        cx.update(|window, _| {
            window.find_by_label("Acknowledged by Ana, Ben, Cy, Dee, Eve and Fay");
        });
        assert!(cx.debug_bounds("avatar-group-overflow").is_some());
    }

    #[gpui_kit::test]
    fn a_group_within_the_limit_has_no_chip(cx: &mut TestAppContext) {
        let mut cx = draw(4, cx);
        assert!(cx.debug_bounds("avatar-group-overflow").is_none());
    }

    #[test]
    fn names_read_as_a_list() {
        let names: Vec<SharedString> = vec!["Ana".into(), "Ben".into()];
        assert_eq!(list_names(&names), "Ana and Ben");
        assert_eq!(list_names(&names[..1]), "Ana");
    }
}
