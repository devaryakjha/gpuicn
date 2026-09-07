import hljs from "highlight.js/lib/core"
import bash from "highlight.js/lib/languages/bash"
import ini from "highlight.js/lib/languages/ini"
import json from "highlight.js/lib/languages/json"
import rust from "highlight.js/lib/languages/rust"

hljs.registerLanguage("rust", rust)
hljs.registerLanguage("bash", bash)
hljs.registerLanguage("toml", ini)
hljs.registerLanguage("json", json)

export type CodeLanguage = "rust" | "bash" | "toml" | "json"

export function highlight(value: string, language: CodeLanguage) {
  return hljs.highlight(value, { language }).value
}
