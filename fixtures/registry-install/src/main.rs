#![allow(dead_code)]

mod ui;

use std::borrow::Cow;

use gpui::{
    App, AppContext as _, Context, IntoElement, ParentElement as _, Render, Styled,
    Window, WindowOptions, div, px,
};
use gpui_icons::LucideAssetSource;
use ui::{button::Button, theme::UiTheme};

fn main() {
    gpui_kit::application()
        .with_assets(LucideAssetSource)
        .run(|cx: &mut App| {
            ui::theme::init(cx);
            cx.text_system()
                .add_fonts(vec![
                    Cow::Borrowed(include_bytes!("../assets/fonts/Geist-Regular.ttf")),
                    Cow::Borrowed(include_bytes!("../assets/fonts/Geist-Medium.ttf")),
                    Cow::Borrowed(include_bytes!("../assets/fonts/GeistMono-Regular.ttf")),
                ])
                .expect("load bundled Geist fonts");
            UiTheme::set(cx, UiTheme::neutral_light());
            cx.open_window(WindowOptions::default(), |_, cx| {
                cx.new(|_| Hello { count: 0 })
            })
            .expect("open application window");
            cx.activate(true);
        });
}

struct Hello {
    count: usize,
}

impl Render for Hello {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = UiTheme::read(cx);
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .p(px(24.))
            .bg(theme.colors.background)
            .text_color(theme.colors.foreground)
            .font_family(theme.fonts.body.clone())
            .child(
                Button::new("hello.increment")
                    .aria_label(format!("Clicked {} times", self.count))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.count += 1;
                        cx.notify();
                    }))
                    .label(format!("Clicked {} times", self.count)),
            )
    }
}
