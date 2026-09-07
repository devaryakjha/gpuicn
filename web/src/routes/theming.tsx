import { createFileRoute } from "@tanstack/react-router"

import { CodeBlock, DocsLayout, PageHeader } from "@/components/docs"
import setup from "@/generated/setup.json"

export const Route = createFileRoute("/theming")({ component: ThemingPage })

function ThemingPage() {
  const source = `use crate::ui::theme::{ThemeMode, UiTheme};\n\n// Install the default dark palette.\nUiTheme::switch(cx, ThemeMode::Dark);\ncx.refresh_windows();`

  return (
    <DocsLayout>
      <PageHeader
        eyebrow="Sections"
        title="Theming"
        description="gpuicn uses editable semantic tokens for color, radius, and typography."
      />
      <CodeBlock className="mt-8" value={source} />
      <p className="mt-5 max-w-2xl leading-7 text-muted-foreground">
        Components consume the same semantic tokens. Own the theme in your app
        and switch it without changing component source.
      </p>
      <h2 className="mt-10 text-2xl font-semibold tracking-tight">
        Customize the tokens
      </h2>
      <p className="mt-3 leading-7 text-muted-foreground">
        Start with a palette, change the values you need, then install it.
        Repaint existing windows after changing the theme. Calling{" "}
        <code className="text-sm">UiTheme::switch</code> resets all tokens to
        the chosen default palette, including custom fonts and radii.
      </p>
      <CodeBlock
        className="mt-4"
        value={
          "use gpui::{px, rgb};\nuse crate::ui::theme::UiTheme;\n\nlet mut theme = UiTheme::neutral_light();\ntheme.colors.primary = rgb(0x2563eb);\ntheme.colors.primary_foreground = rgb(0xffffff);\ntheme.radius.lg = px(8.);\nUiTheme::set(cx, theme);\ncx.refresh_windows();"
        }
      />
      <h2 className="mt-10 text-2xl font-semibold tracking-tight">
        Shared design values
      </h2>
      <ul className="mt-4 list-disc space-y-2 pl-5 leading-7 text-muted-foreground">
        <li>
          Semantic colors for surfaces, content, controls, borders, and focus.
        </li>
        <li>Geist body and heading fonts, plus Geist Mono for code.</li>
        <li>Corner radii, spacing, shadows, and focus rings.</li>
      </ul>
      <details className="mt-6">
        <summary className="cursor-pointer text-sm font-medium">
          View all editable theme tokens
        </summary>
        <CodeBlock className="mt-4" value={setup.theme} />
      </details>
    </DocsLayout>
  )
}
