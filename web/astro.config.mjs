// @ts-check
import { defineConfig } from "astro/config"

// A static site: every page is plain HTML with inlined CSS, and the only
// client scripts are the small ones under src/scripts (theme, previews,
// search, copy buttons). Cloudflare serves web/dist as static assets.
export default defineConfig({
  site: "https://ui.imajha.com",
  trailingSlash: "always",
  build: { inlineStylesheets: "always" },
  // Whitespace between Shiki's spans is code; keep it.
  compressHTML: false,
  devToolbar: { enabled: false },
})
