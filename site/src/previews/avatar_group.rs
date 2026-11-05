//! The avatar group catalog preview.

use crate::*;
use gpuicn::avatar_group::AvatarGroup;

impl Showcase {
    pub(crate) fn avatar_group_preview(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let image = "https://raw.githubusercontent.com/devaryakjha/devaryakjha/6526e3d7415b2fb573ba3da4523b5c6948aa5d08/avatar.png";
        let theme = UiTheme::read(cx);
        let people = ["Ada Lovelace", "Grace Hopper", "Alan Turing", "Katherine Johnson", "Niklaus Wirth", "Margaret Hamilton"];
        let group = |id: &'static str, count: usize, size: AvatarSize| {
            people.iter().take(count).fold(
                AvatarGroup::new(id).aria_label("Acknowledged by").size(size),
                |group, name| {
                    let initial: String = name.chars().take(1).collect();
                    let avatar = Avatar::new(*name).fallback(initial);
                    let avatar = if name.starts_with("Ada") { avatar.image(image) } else { avatar };
                    group.person(*name, avatar)
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
