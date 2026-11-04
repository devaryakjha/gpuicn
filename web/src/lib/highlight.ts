import { createHighlighter, type HighlighterGeneric } from "shiki"

export type CodeLanguage = "rust" | "bash" | "toml" | "json"

let highlighter: Promise<HighlighterGeneric<any, any>> | undefined

function load() {
  highlighter ??= createHighlighter({
    themes: ["github-light-default", "github-dark-default"],
    langs: ["rust", "bash", "toml", "json"],
  })
  return highlighter
}

/**
 * Highlights at build time. The output carries both palettes as CSS
 * variables (`--shiki-light`, `--shiki-dark`), so switching the site theme
 * needs no JavaScript and no second render.
 */
export async function highlight(value: string, language: CodeLanguage) {
  const shiki = await load()
  const html = shiki.codeToHtml(value, {
    lang: language,
    // GitHub's default themes keep every token above 4.5:1 on the code
    // background, except the light comment grey, which is darkened here.
    themes: { light: "github-light-default", dark: "github-dark-default" },
    colorReplacements: { "github-light-default": { "#6e7781": "#5f6874" } },
    defaultColor: false,
  })
  // Keep only the highlighted lines; Code.astro supplies its own <pre><code>.
  // Lines stay separated by real newlines, so copying the text is exact.
  const inner = html.match(/<code>([\s\S]*)<\/code><\/pre>$/)?.[1]
  if (inner === undefined) throw new Error("Unexpected Shiki output")
  return inner
}
