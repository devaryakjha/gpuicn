use std::borrow::Cow;

use gpui::{
    App, AppContext as _, Application, Bounds, Context, IntoElement, ParentElement as _, Render,
    Styled as _, Window, WindowBounds, WindowOptions, div, px, size,
};
use gpui_icons::LucideAssetSource;
use imajha_ui::{
    Button, ButtonSize, ButtonVariant, Checkbox, ThemeMode, UiTheme,
    dialog::{
        dialog_backdrop, dialog_close, dialog_description, dialog_popup, dialog_portal,
        dialog_root, dialog_title, dialog_trigger, dialog_viewport,
    },
};

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(
    inline_js = "export function previewReady(){window.parent.postMessage({imajhaUi:'preview-ready'},'*')}"
)]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = previewReady)]
    fn preview_ready();
}

fn main() {
    #[cfg(target_family = "wasm")]
    {
        gpui_platform::web_init();
        let handle = application().run_embedded(launch);
        std::mem::forget(handle);
    }

    #[cfg(not(target_family = "wasm"))]
    application().run(launch);
}

fn application() -> Application {
    #[cfg(target_family = "wasm")]
    let app =
        gpui_platform::application_with_web_backend(gpui_platform::WebBackendPreference::WebGpu);
    #[cfg(not(target_family = "wasm"))]
    let app = gpui_platform::application();

    app.with_assets(LucideAssetSource)
}

fn launch(cx: &mut App) {
    cx.text_system()
        .add_fonts(vec![
            Cow::Borrowed(include_bytes!("../assets/fonts/Geist-Regular.ttf")),
            Cow::Borrowed(include_bytes!("../assets/fonts/Geist-Medium.ttf")),
            Cow::Borrowed(include_bytes!("../assets/fonts/GeistMono-Regular.ttf")),
        ])
        .expect("failed to load pinned Geist fonts");
    imajha_ui::init(cx);

    let demo = requested_value("demo")
        .and_then(|value| Demo::parse(&value))
        .unwrap_or_default();
    let mode = match requested_value("theme").as_deref() {
        Some("dark") => ThemeMode::Dark,
        _ => ThemeMode::Light,
    };
    UiTheme::switch(cx, mode);

    let bounds = Bounds::centered(None, size(px(720.0), px(480.0)), cx);
    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            ..Default::default()
        },
        move |_window, cx| {
            #[cfg(target_family = "wasm")]
            _window.on_next_frame(|window, _| window.on_next_frame(|_, _| preview_ready()));
            cx.new(move |_| Showcase {
                demo,
                count: 0,
                checked: false,
            })
        },
    )
    .expect("failed to open showcase window");
    #[cfg(target_family = "wasm")]
    cx.activate(true);
    #[cfg(not(target_family = "wasm"))]
    if std::env::var_os("IMAJHA_SHOWCASE_BACKGROUND").is_none() {
        cx.activate(true);
    }
}

#[derive(Clone, Copy, Default)]
enum Demo {
    #[default]
    Button,
    Checkbox,
    Dialog,
}

impl Demo {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "button" => Some(Self::Button),
            "checkbox" => Some(Self::Checkbox),
            "dialog" => Some(Self::Dialog),
            _ => None,
        }
    }
}

struct Showcase {
    demo: Demo,
    count: usize,
    checked: bool,
}

impl Render for Showcase {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = UiTheme::read(cx).clone();
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(theme.colors.background)
            .text_color(theme.colors.foreground)
            .font_family(theme.fonts.body)
            .child(match self.demo {
                Demo::Button => self.button_preview(cx).into_any_element(),
                Demo::Checkbox => self.checkbox_preview(cx).into_any_element(),
                Demo::Dialog => self.dialog_preview(cx).into_any_element(),
            })
    }
}

impl Showcase {
    fn button_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let variants = [
            (ButtonVariant::Default, "Default"),
            (ButtonVariant::Secondary, "Secondary"),
            (ButtonVariant::Outline, "Outline"),
            (ButtonVariant::Ghost, "Ghost"),
            (ButtonVariant::Destructive, "Destructive"),
            (ButtonVariant::Link, "Link"),
        ];

        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(20.0))
            .child(
                Button::new("preview.button.interactive")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.count += 1;
                        cx.notify();
                    }))
                    .child(format!("Clicked {} times", self.count)),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .justify_center()
                    .gap(px(8.0))
                    .children(variants.into_iter().map(|(variant, label)| {
                        Button::new(format!("preview.button.{}", label.to_lowercase()))
                            .variant(variant)
                            .child(label)
                    })),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        Button::new("preview.button.small")
                            .size(ButtonSize::Xs)
                            .child("Extra small"),
                    )
                    .child(
                        Button::new("preview.button.large")
                            .size(ButtonSize::Lg)
                            .child("Large"),
                    )
                    .child(
                        Button::new("preview.button.disabled")
                            .disabled(true)
                            .child("Disabled"),
                    ),
            )
    }

    fn checkbox_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity().downgrade();
        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .child(checkbox_row(
                Checkbox::new("preview.checkbox.interactive")
                    .checked(self.checked)
                    .aria_label("Accept terms")
                    .on_checked_change(move |checked, _, _, cx| {
                        view.update(cx, |this, cx| {
                            this.checked = checked;
                            cx.notify();
                        })
                        .ok();
                    }),
                if self.checked {
                    "Accepted"
                } else {
                    "Accept terms"
                },
            ))
            .child(checkbox_row(
                Checkbox::new("preview.checkbox.indeterminate")
                    .indeterminate(true)
                    .aria_label("Partly selected"),
                "Partly selected",
            ))
            .child(checkbox_row(
                Checkbox::new("preview.checkbox.disabled")
                    .default_checked(true)
                    .disabled(true)
                    .aria_label("Disabled"),
                "Disabled",
            ))
            .child(checkbox_row(
                Checkbox::new("preview.checkbox.readonly")
                    .default_checked(true)
                    .read_only(true)
                    .aria_label("Read only"),
                "Read only",
            ))
    }

    fn dialog_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        dialog_root("preview.dialog")
            .child(dialog_trigger("preview.dialog.trigger", cx).child("Open dialog"))
            .child(
                dialog_portal().child(dialog_backdrop()).child(
                    dialog_viewport().child(
                        dialog_popup("preview.dialog.popup", "Edit profile", cx)
                            .child(dialog_title("preview.dialog.title", cx).child("Edit profile"))
                            .child(
                                dialog_description("preview.dialog.description", cx)
                                    .child("Make changes to your profile, then close this dialog."),
                            )
                            .child(dialog_close("preview.dialog.close", cx)),
                    ),
                ),
            )
    }
}

fn checkbox_row(checkbox: Checkbox, label: &'static str) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .child(checkbox)
        .child(label)
}

#[cfg(target_family = "wasm")]
fn requested_value(key: &str) -> Option<String> {
    let search = web_sys::window()?.location().search().ok()?;
    search
        .trim_start_matches('?')
        .split('&')
        .find_map(|pair| pair.split_once('=').filter(|(name, _)| *name == key))
        .map(|(_, value)| value.to_owned())
}

#[cfg(not(target_family = "wasm"))]
fn requested_value(key: &str) -> Option<String> {
    match key {
        "demo" => std::env::args().nth(1),
        "theme" => std::env::args().nth(2),
        _ => None,
    }
}
