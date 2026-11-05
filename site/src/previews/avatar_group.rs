//! The avatar group catalog preview.

use crate::*;
use gpuicn::avatar_group::AvatarGroup;

impl Showcase {
    pub(crate) fn avatar_group_preview(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = UiTheme::read(cx);
        let people = ["Ada Lovelace", "Grace Hopper", "Alan Turing", "Katherine Johnson", "Niklaus Wirth", "Margaret Hamilton"];
        let group = |id: &'static str, count: usize, size: AvatarSize| {
            people.iter().take(count).fold(
                AvatarGroup::new(id).aria_label("Acknowledged by").size(size),
                |group, name| {
                    let initials: String = name.split(' ').filter_map(|part| part.chars().next()).collect();
                    group.person(*name, Avatar::new(*name).fallback(initials))
                },
            )
        };
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(theme.space(4.))
            .child(group("avatar-group.sm", 3, AvatarSize::Sm))
            .child(group("avatar-group.default", 6, AvatarSize::Default))
            .child(group("avatar-group.lg", 5, AvatarSize::Lg))
    }
}

pub(crate) fn preview(
    view: &Showcase,
    window: &mut Window,
    cx: &mut Context<Showcase>,
) -> gpui_kit::AnyElement {
    view.avatar_group_preview(window, cx).into_any_element()
}
