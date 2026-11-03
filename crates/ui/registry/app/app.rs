//! One-call application start for gpuicn apps.
//!
//! `app::run` opens the platform with the Lucide icon assets, loads the
//! bundled Geist fonts, installs the light and dark themes, follows the
//! operating system's appearance and opens the first window, so a starter
//! `main.rs` is a window title, a theme pair and a root view.

use std::borrow::Cow;

use gpui_icons::LucideAssetSource;
use gpui_kit::{
    App, AppContext as _, Context, Render, SharedString, TitlebarOptions, Window, WindowOptions,
};

use super::theme::{Appearance, UiTheme, follow_system, init};

/// What `run` sets up before the first window opens.
pub struct AppConfig {
    title: SharedString,
    light: UiTheme,
    dark: UiTheme,
    appearance: Appearance,
    fonts: Vec<Cow<'static, [u8]>>,
    window: Option<WindowOptions>,
}

impl AppConfig {
    /// A window titled `title` with the Neutral themes, the system appearance
    /// and the bundled Geist fonts.
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            light: UiTheme::neutral_light(),
            dark: UiTheme::neutral_dark(),
            appearance: Appearance::System,
            fonts: bundled_fonts(),
            window: None,
        }
    }

    /// The light and dark themes the application switches between, for
    /// example `app_theme::light()` and `app_theme::dark()` from `gpuicn init`.
    pub fn themes(mut self, light: UiTheme, dark: UiTheme) -> Self {
        self.light = light;
        self.dark = dark;
        self
    }

    /// Which theme the application opens with; `System` follows the OS.
    pub fn appearance(mut self, appearance: Appearance) -> Self {
        self.appearance = appearance;
        self
    }

    /// Font files to load instead of the bundled Geist families. Set the
    /// theme's font names to match.
    pub fn fonts(mut self, fonts: impl IntoIterator<Item = Cow<'static, [u8]>>) -> Self {
        self.fonts = fonts.into_iter().collect();
        self
    }

    /// Options for the first window; the title bar text defaults to `title`.
    pub fn window_options(mut self, options: WindowOptions) -> Self {
        self.window = Some(options);
        self
    }

    /// Loads the fonts, initializes gpuicn and installs the theme pair without
    /// opening a window, for applications that open their own windows.
    pub fn install(self, cx: &mut App) -> WindowOptions {
        if !self.fonts.is_empty() {
            cx.text_system()
                .add_fonts(self.fonts)
                .expect("load the application's fonts");
        }
        init(cx);
        UiTheme::set_pair(cx, self.light, self.dark);
        UiTheme::set_appearance(cx, self.appearance);
        let mut options = self.window.unwrap_or_default();
        let titlebar = options
            .titlebar
            .get_or_insert_with(TitlebarOptions::default);
        if titlebar.title.is_none() {
            titlebar.title = Some(self.title);
        }
        options
    }
}

/// Starts the application and opens one window whose root view `root` builds.
///
/// Opens the platform with the Lucide assets, applies `config`, follows the
/// system appearance in the window and activates the application.
pub fn run<V: Render>(
    config: AppConfig,
    root: impl FnOnce(&mut Window, &mut Context<V>) -> V + 'static,
) {
    let launch = move |cx: &mut App| {
        let options = config.install(cx);
        cx.open_window(options, |window, cx| {
            follow_system(window, cx).detach();
            cx.new(|cx| root(window, cx))
        })
        .expect("open the application window");
        cx.activate(true);
    };
    let application = gpui_kit::platform::application().with_assets(LucideAssetSource);
    #[cfg(target_family = "wasm")]
    {
        gpui_kit::platform::web_init();
        std::mem::forget(application.run_embedded(launch));
    }
    #[cfg(not(target_family = "wasm"))]
    application.run(launch);
}

/// The Geist families the default theme names, from the app's `assets/fonts`.
pub fn bundled_fonts() -> Vec<Cow<'static, [u8]>> {
    vec![
        Cow::Borrowed(include_bytes!("../../assets/fonts/Geist-Regular.ttf")),
        Cow::Borrowed(include_bytes!("../../assets/fonts/Geist-Medium.ttf")),
        Cow::Borrowed(include_bytes!("../../assets/fonts/Geist-SemiBold.ttf")),
        Cow::Borrowed(include_bytes!("../../assets/fonts/GeistMono-Regular.ttf")),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::TestAppContext;

    #[gpui_kit::test]
    fn install_sets_the_pair_and_titles_the_window(cx: &mut TestAppContext) {
        let mut dark = UiTheme::neutral_dark();
        dark.text_scale = 1.25;
        let config = AppConfig::new("Matty")
            .themes(UiTheme::neutral_light(), dark)
            .appearance(Appearance::Dark)
            .fonts([]);
        let options = cx.update(|cx| config.install(cx));
        assert_eq!(options.titlebar.unwrap().title.as_deref(), Some("Matty"));
        cx.update(|cx| {
            assert_eq!(UiTheme::read(cx).mode, super::super::theme::ThemeMode::Dark);
            assert_eq!(UiTheme::read(cx).text_scale, 1.25);
            UiTheme::set_appearance(cx, Appearance::Light);
            assert_eq!(
                UiTheme::read(cx).mode,
                super::super::theme::ThemeMode::Light
            );
            assert_eq!(UiTheme::read(cx).text_scale, 1.0);
        });
    }
}
