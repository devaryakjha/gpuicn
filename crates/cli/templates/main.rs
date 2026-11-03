#![allow(dead_code)]

mod ui;

use gpui_kit::{Context, IntoElement, ParentElement as _, Render, Styled, Window, div, px};
use ui::{
    app::{AppConfig, run},
    app_theme,
    button::Button,
    theme::UiTheme,
};

fn main() {
    // Opens the window with the Lucide icons, the Geist fonts and the theme pair
    // from assets/theme.css; `gpuicn theme set` changes the base color, radius
    // and appearance.
    run(
        AppConfig::new("My App")
            .themes(app_theme::light(), app_theme::dark())
            .appearance(app_theme::APPEARANCE),
        |_, _| Hello { count: 0 },
    );
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
