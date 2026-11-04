import assert from "node:assert/strict"
import { highlight, type CodeLanguage } from "../src/lib/highlight"

// Build-time highlighting must escape its input and keep the text intact.
for (const [language, source] of [
  ["rust", 'fn main() { let text = "<script>alert(1)</script>"; }'],
  ["json", '{"name": "<img src=x onerror=alert(1)>"}'],
  ["toml", '[package]\nname = "example"'],
  ["bash", 'echo "hello"'],
] satisfies [CodeLanguage, string][]) {
  const html = await highlight(source, language)
  assert.match(html, /style="--shiki-light:/, `${language} must carry both palettes`)
  assert.match(html, /--shiki-dark:/)
  assert.doesNotMatch(html, /<(?:script|img)\b/)
  const text = html
    .replace(/<[^>]*>/g, "")
    .replace(
      /&(amp|lt|gt|quot|#x27|#39|#x3C|#x3E);/g,
      (_, entity: string) =>
        ({ amp: "&", lt: "<", gt: ">", quot: '"', "#x27": "'", "#39": "'", "#x3C": "<", "#x3E": ">" })[entity]!
    )
  assert.equal(text, source, `${language} must preserve the original code`)
}
console.log("Highlighting: every language escapes HTML and preserves its source.")
