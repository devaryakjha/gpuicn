import assert from "node:assert/strict";
import { copyFileSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";
import { dirname } from "node:path";

const root = fileURLToPath(new URL("../", import.meta.url));
const read = (path) => readFileSync(`${root}${path}`, "utf8");
const showcase = read("site/src/main.rs");
const usageExamples = read("crates/ui/examples/usage.rs");
const registry = JSON.parse(read("registry.json"));
const destination = `${root}web/src/generated/components`;
mkdirSync(destination, { recursive: true });
writeFileSync(`${root}web/src/generated/setup.json`, JSON.stringify({
  dependencies: read("fixtures/registry-install/Cargo.toml").split("[dependencies]\n")[1].split("\n[workspace]")[0].trim(),
  config: read("fixtures/registry-install/components.json").trim(),
  aliases: read("fixtures/registry-install/tsconfig.json").trim(),
  starter: read("fixtures/registry-install/src/main.rs").replace("#![allow(dead_code)]\n\n", "").trim(),
  theme: read("registry/theme/theme.rs"),
}, null, 2));

// Read rustfmt's method boundaries, not Rust expressions. Cargo checks these
// same methods; the assertions below fail if their layout or coverage changes.
for (const item of registry.items.filter((item) => item.name !== "theme")) {
  const module = item.name.replaceAll("-", "_");
  const source = read(`registry/${module}/${module}.rs`);
  const preview = showcase.match(new RegExp(`^    fn ${module}_preview\\([\\s\\S]*?^    \\}`, "m"))?.[0];
  assert(preview, `Missing compiled preview for ${item.name}`);
  const usage = usageExamples.match(new RegExp(`^mod ${module} \\{\\n([\\s\\S]*?)^\\}`, "m"))?.[1].replace(/^    /gm, "").trim();
  assert(usage?.includes("fn example("), `Missing compile-checked usage for ${item.name}`);

  const api = [...source.matchAll(/(?:^\s*\/\/\/[^\n]*\n)*(?:^\s*#\[[^\n]*\]\n)*^\s*pub (?:const )?fn [\s\S]*?\{/gm)]
    .map(([entry]) => {
      const description = [...entry.matchAll(/\/\/\/ ?([^\n]*)/g)].map(([, line]) => line).join(" ");
      const signature = entry.slice(entry.indexOf("pub "), -1).trim();
      return { signature, description };
    });
  assert(api.length > 0, `Missing public API for ${item.name}`);
  assert.equal(api.length, [...source.matchAll(/\bpub (?:const )?fn /g)].length, `Incomplete API for ${item.name}`);

  const parityName = readdirSync(`${root}docs/parity`).find((name) => name.replaceAll("_", "-") === `${item.name}.md`);
  assert(parityName, `Missing platform notes for ${item.name}`);
  const parity = read(`docs/parity/${parityName}`);
  const sidebarExample = (name) => {
    const method = showcase.match(new RegExp(`^    fn sidebar_${name}_preview\\([\\s\\S]*?^    \\}`, "m"))?.[0];
    assert(method, `Missing Sidebar ${name} example`);
    return method.replace(/^    /gm, "");
  };
  const examples = item.name === "sidebar" ? Object.fromEntries(
    ["workspace", "docs", "mail", "floating", "mobile", "loading"].map((name) =>
      [name, sidebarExample(name === "docs" || name === "mail" ? name : "application")])
  ) : undefined;

  writeFileSync(`${destination}/${item.name}.json`, `${JSON.stringify({
    preview: preview.replace(/^    /gm, ""),
    examples,
    usage,
    usageCall: usage.includes("fn example(cx:") ? "example(cx)" : "example()",
    source,
    api,
    parity,
    modules: item.files.filter((file) => file.path.endsWith(".rs") && !file.path.endsWith("/modal_focus.rs")).map((file) => file.path.split("/").at(-1).replace(".rs", "")),
  }, null, 2)}\n`);
}

mkdirSync(`${root}web/public/examples`, { recursive: true });
copyFileSync(`${root}site/src/main.rs`, `${root}web/public/examples/showcase.rs`);
copyFileSync(`${root}fixtures/registry-install/src/main.rs`, `${root}web/public/examples/hello.rs`);
mkdirSync(`${root}web/public/fonts`, { recursive: true });
mkdirSync(`${root}fixtures/registry-install/assets/fonts`, { recursive: true });
for (const font of readdirSync(`${root}site/assets/fonts`)) {
  copyFileSync(`${root}site/assets/fonts/${font}`, `${root}web/public/fonts/${font}`);
  copyFileSync(`${root}site/assets/fonts/${font}`, `${root}fixtures/registry-install/assets/fonts/${font}`);
}
copyFileSync(`${root}LICENSES/Geist-OFL-1.1`, `${root}web/public/fonts/OFL.txt`);
const metadata = JSON.parse(execFileSync("cargo", ["metadata", "--locked", "--format-version", "1"], { cwd: root, maxBuffer: 8 * 1024 * 1024 }));
const iconRoot = dirname(metadata.packages.find((item) => item.name === "gpui-icons").manifest_path);
const iconManifest = JSON.parse(readFileSync(`${iconRoot}/RELEASE-MANIFEST.json`, "utf8"));
assert.equal(iconManifest.icons.length, 1776, "The catalog requires the complete pinned gpui-icons library");
mkdirSync(`${root}web/public/icons/lucide`, { recursive: true });
for (const icon of iconManifest.icons) {
  copyFileSync(`${iconRoot}/assets/lucide/${icon.canonical_name}.svg`, `${root}web/public/${icon.asset_path}`);
}
copyFileSync(`${iconRoot}/LICENSE`, `${root}web/public/icons/LICENSE`);
writeFileSync(`${root}web/src/generated/icons.json`, JSON.stringify(iconManifest.icons.map((icon) => ({
  name: icon.canonical_name,
  rust: icon.rust_name,
  tags: icon.tags,
  categories: icon.categories,
  aliases: icon.aliases.map((alias) => typeof alias === "string" ? alias : alias.name),
}))));
console.log(`Synced ${registry.items.length - 1} compiled previews, APIs, and starter assets.`);
