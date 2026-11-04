// Trunk minifies the preview's HTML but not the JavaScript wasm-bindgen emits.
// Minify those modules in place and rewrite their integrity hashes in
// index.html. Does nothing when the preview bundle has not been built.
import { createHash } from "node:crypto"
import { existsSync, readFileSync, writeFileSync } from "node:fs"
import { join } from "node:path"

const demo = join(import.meta.dir, "../public/demo")
const index = join(demo, "index.html")
if (!existsSync(index)) process.exit(0)

const transpiler = new Bun.Transpiler({
  loader: "js",
  minifyWhitespace: true,
  minifySyntax: true,
  // wasm-bindgen looks exports up by name.
  minifyIdentifiers: false,
})

let html = readFileSync(index, "utf8")
let saved = 0
for (const match of html.matchAll(/<link crossorigin href=\/demo\/(\S+?\.js) integrity=(sha384-\S+?) rel=modulepreload>/g)) {
  const [, path, integrity] = match
  const file = join(demo, path)
  const source = readFileSync(file, "utf8")
  const output = transpiler.transformSync(source)
  saved += source.length - output.length
  writeFileSync(file, output)
  const hash = "sha384-" + createHash("sha384").update(output).digest("base64")
  html = html.replace(integrity, hash)
}
writeFileSync(index, html)
console.log(`Minified the preview JavaScript, ${saved} bytes smaller.`)
