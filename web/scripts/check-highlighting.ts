import assert from "node:assert/strict"
import { highlight, type CodeLanguage } from "../src/lib/highlight"

for (const [language, source] of [
  ["rust", 'fn main() { let text = "<script>alert(1)</script>"; }'],
  ["json", '{"name": "<img src=x onerror=alert(1)>"}'],
  ["toml", '[package]\nname = "example"'],
  ["bash", 'echo "hello"'],
] satisfies [CodeLanguage, string][]) {
  const html = highlight(source, language)
  assert.match(html, /class="hljs-/)
  assert.doesNotMatch(html, /<(?:script|img)\b/)
  const text = html
    .replace(/<[^>]*>/g, "")
    .replace(
      /&(amp|lt|gt|quot|#x27);/g,
      (_, entity: string) =>
        ({ amp: "&", lt: "<", gt: ">", quot: '"', "#x27": "'" })[entity]!
    )
  assert.equal(text, source, `${language} must preserve the original code`)
}
