# App parity

Visual reference: none; shadcn/ui 4.19.0 has no equivalent because a web app's
start is `index.html`, the Tailwind entry and `globals.css`. Implementation:
`crates/ui/registry/app/app.rs`, using GPUI Kit 0.9.0.

`app::run(AppConfig, root)` is the native equivalent of that setup: it opens
the platform with the Lucide assets, loads the bundled Geist fonts, installs a
light and dark `UiTheme` pair, selects one by the configured `Appearance`
(Light, Dark or System), keeps it in step with the operating system through
`theme::follow_system`, and opens the first window titled after the app.
`AppConfig::install` does the same without opening a window for applications
that manage their own windows. `gpuicn init` generates `src/ui/app_theme.rs`
from the chosen base color so the starter `main.rs` is a window title, a theme
pair and a root view.

The Rust source and Usage example define the supported API.
