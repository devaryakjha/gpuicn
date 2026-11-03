//! The kbd catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn kbd_preview(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        let row = |label: &'static str, keys: gpui_kit::Div| {
            div()
                .w_full()
                .flex()
                .items_center()
                .justify_between()
                .gap(px(16.))
                .child(
                    div()
                        .text_size(px(14.))
                        .text_color(theme.colors.muted_foreground)
                        .child(label),
                )
                .child(keys)
        };
        div()
            .w_full()
            .max_w(px(320.0))
            .flex()
            .flex_col()
            .gap(px(12.0))
            .child(row(
                "Quick switcher",
                kbd_group([Kbd::shortcut("secondary-k")], cx),
            ))
            .child(row(
                "Command palette",
                kbd_group([Kbd::shortcut("secondary-shift-p")], cx),
            ))
            .child(row(
                "Jump to unread",
                kbd_group([Kbd::new("Alt"), Kbd::new("Shift"), Kbd::new("↑")], cx),
            ))
            .child(row("Close", kbd_group([Kbd::new("Esc")], cx)))
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.kbd_preview(window, cx).into_any_element()
}
