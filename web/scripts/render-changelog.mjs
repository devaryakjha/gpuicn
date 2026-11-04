// Renders CHANGELOG.md into src/generated/changelog.json for the changelog page.
import { readFileSync, writeFileSync } from "node:fs";

const source = new URL("../../CHANGELOG.md", import.meta.url);
const base = "https://github.com/devaryakjha/gpuicn/blob/main/";

export function renderChangelog(markdown) {
  const html = Bun.markdown.html(markdown, { noHtmlBlocks: true, noHtmlSpans: true });
  const versions = [];
  let heading = null;
  const out = new HTMLRewriter()
    // The page supplies its own title.
    .on("h1", { element: (element) => element.remove() })
    .on("h2", {
      element(element) {
        heading = { text: "" };
        element.setAttribute("class", "release");
        element.onEndTag(() => {
          const version = heading.text.trim();
          versions.push(version);
          heading = null;
        });
      },
      text(chunk) {
        if (heading) heading.text += chunk.text;
      },
    })
    .on("h3", { element: (element) => element.setAttribute("class", "release__kind") })
    .on("a", {
      element(element) {
        const url = new URL(element.getAttribute("href"), base);
        if (["https:", "http:", "mailto:"].includes(url.protocol)) element.setAttribute("href", url.href);
        else element.removeAndKeepContent();
      },
    })
    .transform(html);
  // Give each release heading an anchor, in document order.
  let index = 0;
  const withIds = out.replace(/<h2 class="release">/g, () => `<h2 class="release" id="v${versions[index++]}">`);
  return { html: withIds, versions };
}

if (import.meta.main) {
  const { html, versions } = renderChangelog(readFileSync(source, "utf8"));
  if (versions.length === 0) throw new Error("CHANGELOG.md has no release headings");
  writeFileSync(new URL("../src/generated/changelog.json", import.meta.url), JSON.stringify({ html, versions }, null, 2) + "\n");
}
