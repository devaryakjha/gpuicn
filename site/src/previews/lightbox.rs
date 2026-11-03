//! The lightbox catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn lightbox_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        use gpui_kit::StyledImage as _;
        let images = lightbox_images();
        let handle = window
            .use_keyed_state("lightbox.handle", cx, |_, _| DialogHandle::new(false))
            .read(cx)
            .clone();
        let state = window.use_keyed_state("lightbox.state", cx, |_, _| {
            LightboxState::new(lightbox_images())
        });
        let theme = UiTheme::read(cx).clone();
        div()
            .flex()
            .gap(px(12.))
            .children(images.into_iter().enumerate().map(|(index, image)| {
                let open = handle.clone();
                let state = state.clone();
                div()
                    .id(gpui_kit::ElementId::Name(
                        format!("lightbox.thumb.{index}").into(),
                    ))
                    .role(gpui_kit::Role::Button)
                    .aria_label(format!("Open {}", image.name))
                    .w(px(144.))
                    .h(px(96.))
                    .overflow_hidden()
                    .rounded(theme.radius.lg)
                    .border_1()
                    .border_color(theme.colors.border)
                    .bg(theme.colors.muted)
                    .cursor_pointer()
                    .on_click(move |_, window, cx| {
                        state.update(cx, |state, cx| state.show(index, cx));
                        open.open(window, cx);
                    })
                    .child(
                        // GPUI clips to a rectangle, so the image takes the inner radius.
                        // Layout otherwise sizes an image by its own aspect ratio,
                        // so a tall image would overflow the frame's rounded clip.
                        gpui_kit::img(image.source)
                            .size_full()
                            .aspect_ratio(144. / 96.)
                            .rounded(theme.radius.lg - px(1.))
                            .object_fit(gpui_kit::ObjectFit::Cover),
                    )
            }))
            .child(lightbox(
                "lightbox.viewer",
                &handle,
                &state,
                [Button::new("lightbox.download")
                    .size(ButtonSize::Icon)
                    .variant(ButtonVariant::Ghost)
                    .aria_label("Download")
                    .child(
                        lucide(LucideIcon::Download)
                            .size(px(18.))
                            .text_color(gpui_kit::white()),
                    )
                    .into_any_element()],
                window,
                cx,
            ))
    }
}

fn lightbox_images() -> Vec<LightboxImage> {
    let repo = "https://raw.githubusercontent.com/devaryakjha/gpuicn/0f4ecbfad9ccd443af0925137c88d87996e999f7";
    vec![
        LightboxImage::new(format!("{repo}/docs/images/catalog-hero.png"), "catalog-hero.png")
            .caption("Arya in ~gpuicn, 10:42 AM"),
        LightboxImage::new(
            format!("{repo}/docs/images/catalog-components.png"),
            "catalog-components.png",
        )
        .caption("Arya in ~gpuicn, 10:43 AM"),
        LightboxImage::new(
            "https://raw.githubusercontent.com/devaryakjha/devaryakjha/6526e3d7415b2fb573ba3da4523b5c6948aa5d08/avatar.png",
            "avatar.png",
        )
        .caption("Sam in ~random, yesterday"),
    ]
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.lightbox_preview(window, cx).into_any_element()
}
