import { createHighlighter, type HighlighterGeneric } from "shiki"

export type CodeLanguage = "rust" | "bash" | "toml" | "json"

let highlighter: Promise<HighlighterGeneric<any, any>> | undefined

function load() {
  highlighter ??= createHighlighter({
    themes: ["vitesse-light", "vitesse-dark"],
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
    themes: { light: "vitesse-light", dark: "vitesse-dark" },
    defaultColor: false,
  })
  // Keep only the highlighted lines; Code.astro supplies its own <pre><code>.
  // Lines stay separated by real newlines, so copying the text is exact.
  const inner = html.match(/<code>([\s\S]*)<\/code><\/pre>$/)?.[1]
  if (inner === undefined) throw new Error("Unexpected Shiki output")
  return inner
}
