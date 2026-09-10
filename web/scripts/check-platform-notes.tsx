import assert from "node:assert/strict"
import { readFileSync, readdirSync } from "node:fs"
import { renderToStaticMarkup } from "react-dom/server"
import { PlatformNotes } from "../src/components/docs"
import { renderPlatformNotes } from "./render-platform-notes.mjs"

const directory = new URL("../src/generated/components/", import.meta.url)
const files = readdirSync(directory)
for (const name of files) {
  const docs = JSON.parse(readFileSync(new URL(name, directory), "utf8"))
  const html = renderToStaticMarkup(<PlatformNotes content={docs.parityHtml} />)
  assert.match(html, /<p>/, name)
  assert.doesNotMatch(html, /<h1>|\[migration qualification\]\(/, name)
  for (const [, code] of docs.parity.matchAll(/`([^`\n]+)`/g)) {
    assert.ok(html.includes(renderToStaticMarkup(<code>{code}</code>)), `${name}: ${code}`)
  }
  if (docs.parity.includes("[migration qualification]")) {
    assert.match(html, /<a href="https:\/\/github\.com\/devaryakjha\/gpuicn\/blob\/main\/docs\/validation\/gpui-kit-migration\.md">migration qualification<\/a>/, name)
  }
  if (name === "sidebar.json") {
    assert.match(html, /<table>[\s\S]*<th>Capability<\/th>[\s\S]*<td>Shared open state<\/td>/)
  }
}

const fixture = renderPlatformNotes("# Title\n\n## Details\n\n**Strong** and *emphasis*.\n\n- First\n- Second\n\n```rust\nlet value = 1;\n```\n\n<script>alert(1)</script>\n\n[Unsafe](javascript:alert(1))")
assert.match(fixture, /<h2>Details<\/h2>/)
assert.match(fixture, /<strong>Strong<\/strong> and <em>emphasis<\/em>/)
assert.match(fixture, /<ul>[\s\S]*<li>First<\/li>[\s\S]*<li>Second<\/li>/)
assert.match(fixture, /<pre><code class="language-rust">let value = 1;/)
assert.doesNotMatch(fixture, /<h1>|<script>|href="javascript:/)
console.log(`Platform notes: all ${files.length} components render code, links and tables; Markdown and HTML checks pass.`)
