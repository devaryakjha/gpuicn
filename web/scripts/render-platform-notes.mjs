import { readFileSync, readdirSync, writeFileSync } from "node:fs";

export function renderPlatformNotes(markdown) {
  const html = Bun.markdown.html(markdown, { noHtmlBlocks: true, noHtmlSpans: true });
  return new HTMLRewriter()
    .on("h1", { element: (element) => element.remove() })
    .on("a", {
      element(element) {
        const url = new URL(element.getAttribute("href"), "https://github.com/devaryakjha/gpuicn/blob/main/docs/parity/");
        if (["https:", "http:", "mailto:"].includes(url.protocol)) {
          element.setAttribute("href", url.href);
        } else {
          element.removeAndKeepContent();
        }
      },
    })
    .on("table", {
      element(element) {
        element.before('<div class="overflow-x-auto">', { html: true });
        element.after("</div>", { html: true });
      },
    })
    .transform(html);
}

if (import.meta.main) {
  const directory = new URL("../src/generated/components/", import.meta.url);
  for (const name of readdirSync(directory)) {
    const file = new URL(name, directory);
    const docs = JSON.parse(readFileSync(file, "utf8"));
    docs.parityHtml = renderPlatformNotes(docs.parity);
    writeFileSync(file, JSON.stringify(docs, null, 2) + "\n");
  }
}
