// Shoots every catalog preview in light and dark mode from the built WASM
// site, and writes a Markdown sheet that pull requests and release notes link.
//
//   bun scripts/screenshots.mjs --dist web/public/demo --out docs/screenshots/v0.9.0 [--only button,dialog] [--compare docs/screenshots/v0.8.0]
//
// Needs `bun add playwright` somewhere on NODE_PATH, the site built first (`trunk build --release --public-url /demo/ --dist
// ../web/public/demo` in `site/`), Chromium through Playwright, and no network.
// Headless Chromium has no WebGPU, so the page is loaded with `?backend=webgl`,
// which the showcase honors, and SwiftShader draws the frames. With --compare,
// every image is diffed against the same file in that folder and the sheet
// lists the ones that changed by more than --threshold percent of pixels
// (default 0.5); the exit code is 1 when any did, so CI can gate on it.

import { createServer } from "node:http";
import { readFileSync, existsSync, mkdirSync, writeFileSync, readdirSync, statSync } from "node:fs";
import { extname, join, resolve } from "node:path";
import { chromium } from "playwright";

const args = Object.fromEntries(
  process.argv.slice(2).map((arg, i, all) =>
    arg.startsWith("--") ? [arg.slice(2), all[i + 1]?.startsWith("--") || all[i + 1] === undefined ? "true" : all[i + 1]] : [],
  ).filter((pair) => pair.length),
);
const dist = resolve(args.dist ?? "web/public/demo");
const out = resolve(args.out ?? "docs/screenshots/current");
const compare = args.compare ? resolve(args.compare) : null;
const threshold = Number(args.threshold ?? "0.5");
const only = args.only ? new Set(args.only.split(",")) : null;
const wait = Number(args.wait ?? "1500");

if (!existsSync(join(dist, "index.html"))) {
  console.error(`No built site at ${dist}; run trunk build first.`);
  process.exit(2);
}
mkdirSync(out, { recursive: true });

// The preview list is the catalog's; keep it in step with site/src/gallery.rs.
const registry = JSON.parse(readFileSync(resolve("registry.json"), "utf8"));
const demos = registry.items.map((item) => item.name).filter((name) => name !== "theme" && (!only || only.has(name)));
// Wide previews, as in site/src/main.rs.
const wide = new Set(["sidebar", "virtual-list", "message-list"]);

const types = { ".html": "text/html", ".js": "text/javascript", ".wasm": "application/wasm", ".css": "text/css", ".ttf": "font/ttf", ".svg": "image/svg+xml", ".png": "image/png", ".json": "application/json" };
const server = createServer((request, response) => {
  const url = new URL(request.url, "http://localhost");
  let path = join(dist, url.pathname.replace(/^\/demo\//, "/"));
  if (!existsSync(path) || statSync(path).isDirectory()) path = join(dist, "index.html");
  response.writeHead(200, {
    "Content-Type": types[extname(path)] ?? "application/octet-stream",
    "Cross-Origin-Embedder-Policy": "require-corp",
    "Cross-Origin-Opener-Policy": "same-origin",
  });
  response.end(readFileSync(path));
});
await new Promise((ready) => server.listen(0, "127.0.0.1", ready));
const origin = `http://127.0.0.1:${server.address().port}`;

const browser = await chromium.launch({
  args: ["--use-angle=swiftshader", "--enable-unsafe-swiftshader", "--no-sandbox", "--disable-gpu-sandbox"],
});
const rows = [];
const changed = [];
try {
  for (const demo of demos) {
    const [width, height] = wide.has(demo) ? [960, 600] : [640, 288];
    for (const theme of ["light", "dark"]) {
      const page = await browser.newPage({ viewport: { width, height }, deviceScaleFactor: 2 });
      const errors = [];
      page.on("pageerror", (error) => errors.push(String(error)));
      await page.goto(`${origin}/demo/index.html?demo=${demo}&theme=${theme}&backend=webgl`);
      try {
        await page.waitForSelector("html[data-gpuicn-ready]", { timeout: 60_000 });
      } catch {
        errors.push("preview never reported ready");
      }
      await page.waitForTimeout(wait);
      const file = `${demo}-${theme}.png`;
      await page.screenshot({ path: join(out, file) });
      await page.close();
      let delta = null;
      if (compare && existsSync(join(compare, file))) {
        delta = await diffPercent(join(compare, file), join(out, file));
        if (delta > threshold) changed.push({ demo, theme, delta });
      }
      rows.push({ demo, theme, file, errors, delta });
      console.log(`${file}${delta === null ? "" : ` (${delta.toFixed(2)}% changed)`}${errors.length ? ` errors: ${errors.join("; ")}` : ""}`);
    }
  }
} finally {
  await browser.close();
  server.close();
}

// Decodes both images in the browser, so no PNG library is needed here.
async function diffPercent(a, b) {
  const page = await browser.newPage();
  try {
    const [da, db] = [a, b].map((path) => `data:image/png;base64,${readFileSync(path).toString("base64")}`);
    return await page.evaluate(async ([da, db]) => {
      const load = (src) => new Promise((ok, err) => { const img = new Image(); img.onload = () => ok(img); img.onerror = err; img.src = src; });
      const [ia, ib] = await Promise.all([load(da), load(db)]);
      if (ia.width !== ib.width || ia.height !== ib.height) return 100;
      const pixels = (img) => { const c = document.createElement("canvas"); c.width = img.width; c.height = img.height; const g = c.getContext("2d"); g.drawImage(img, 0, 0); return g.getImageData(0, 0, img.width, img.height).data; };
      const [pa, pb] = [pixels(ia), pixels(ib)];
      let differing = 0;
      for (let i = 0; i < pa.length; i += 4) {
        if (Math.abs(pa[i] - pb[i]) > 8 || Math.abs(pa[i + 1] - pb[i + 1]) > 8 || Math.abs(pa[i + 2] - pb[i + 2]) > 8) differing++;
      }
      return (100 * differing) / (ia.width * ia.height);
    }, [da, db]);
  } finally {
    await page.close();
  }
}

const sheet = [
  `# Catalog screenshots`,
  ``,
  `Generated by \`scripts/screenshots.mjs\` from the built site. Each row is one preview at 2x in light and dark mode.`,
  ``,
  `| Component | Light | Dark |`,
  `| --- | --- | --- |`,
  ...demos.map((demo) => `| ${demo} | ![${demo} light](${demo}-light.png) | ![${demo} dark](${demo}-dark.png) |`),
];
if (compare) {
  sheet.push(``, `## Changed against ${compare}`, ``);
  sheet.push(changed.length ? changed.map((c) => `- ${c.demo} (${c.theme}): ${c.delta.toFixed(2)}% of pixels`).join("\n") : "No preview changed by more than the threshold.");
}
const failed = rows.filter((row) => row.errors.length);
if (failed.length) {
  sheet.push(``, `## Previews that reported errors`, ``, ...failed.map((row) => `- ${row.file}: ${row.errors.join("; ")}`));
}
writeFileSync(join(out, "README.md"), sheet.join("\n") + "\n");
console.log(`Wrote ${rows.length} screenshots and ${join(out, "README.md")}`);
if (failed.length) {
  console.error(`${failed.length} preview(s) reported errors`);
  process.exit(1);
}
if (changed.length) {
  console.error(`${changed.length} preview(s) changed by more than ${threshold}%`);
  process.exit(1);
}
