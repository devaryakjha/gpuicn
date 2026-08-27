import { createFileRoute } from "@tanstack/react-router"

import { CodeBlock, DocsLayout, PageHeader } from "@/components/docs"

export const Route = createFileRoute("/theming")({ component: ThemingPage })

function ThemingPage() {
  const source = `use gpuicn::theme::{ThemeMode, UiTheme};\n\nUiTheme::switch(cx, ThemeMode::Dark);`

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
    </DocsLayout>
  )
}
