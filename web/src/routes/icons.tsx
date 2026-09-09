import * as React from "react"
import { createFileRoute } from "@tanstack/react-router"

import {
  CodeBlock,
  DocsLayout,
  GpuPreview,
  PageHeader,
} from "@/components/docs"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import icons from "@/generated/icons.json"

const pageSize = 72

export const Route = createFileRoute("/icons")({
  validateSearch: (search: Record<string, unknown>): { icon?: string } => ({
    icon:
      typeof search.icon === "string" && /^[a-z0-9-]{1,80}$/.test(search.icon)
        ? search.icon
        : undefined,
  }),
  component: IconsPage,
})

function IconsPage() {
  const { icon: requestedName = "house" } = Route.useSearch()
  const navigate = Route.useNavigate()
  const detailsRef = React.useRef<HTMLElement>(null)
  const [query, setQuery] = React.useState("")
  const [page, setPage] = React.useState(0)
  const selected =
    icons.find((icon) => icon.name === requestedName) ??
    icons.find((icon) => icon.name === "house")!
  const selectedName = selected.name
  const terms = query.toLowerCase().trim().split(/\s+/).filter(Boolean)
  const results = icons.filter((icon) => {
    const text = [icon.name, ...icon.tags, ...icon.categories, ...icon.aliases]
      .join(" ")
      .toLowerCase()
    return terms.every((term) => text.includes(term))
  })
  const pageCount = Math.max(1, Math.ceil(results.length / pageSize))
  const visible = results.slice(page * pageSize, (page + 1) * pageSize)

  return (
    <DocsLayout>
      <PageHeader
        eyebrow="gpui-icons"
        title="Icons"
        description="The complete Lucide set for GPUI. Original SVGs, consistent strokes, and editable application code."
      />
      <p className="mt-4 text-sm leading-6 text-muted-foreground">
        {icons.length.toLocaleString()} icons · Lucide 1.43.0 · Native and WASM
        ·{" "}
        <a
          className="underline underline-offset-4"
          href="https://github.com/devaryakjha/gpui-icons"
        >
          Standalone Rust library
        </a>
      </p>

      <div className="mt-8 grid items-start gap-8 lg:grid-cols-[minmax(0,1fr)_18rem]">
        <section className="min-w-0" aria-label="Icon catalog">
          <label
            htmlFor="icon-search"
            className="mb-2 block text-sm font-medium"
          >
            Find an icon
          </label>
          <Input
            id="icon-search"
            value={query}
            placeholder="Search names, tags, or aliases…"
            onChange={(event) => {
              setQuery(event.target.value)
              setPage(0)
            }}
          />
          <p role="status" className="mt-3 text-sm text-muted-foreground">
            {results.length.toLocaleString()} icons
            {query ? ` matching “${query}”` : " available"}
          </p>

          <div className="mt-4 grid grid-cols-3 gap-2 sm:grid-cols-4 xl:grid-cols-5">
            {visible.map((icon) => (
              <button
                key={icon.name}
                type="button"
                aria-pressed={icon.name === selectedName}
                onClick={() =>
                  void navigate({
                    search: { icon: icon.name },
                    replace: true,
                    resetScroll: false,
                  }).then(() => {
                    if (window.matchMedia("(max-width: 1023px)").matches) {
                      detailsRef.current?.scrollIntoView({ block: "start" })
                      detailsRef.current?.focus({ preventScroll: true })
                    }
                  })
                }
                className="flex min-h-24 min-w-0 flex-col items-center justify-center gap-3 rounded-lg border px-2 py-3 hover:bg-muted focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring aria-pressed:border-foreground aria-pressed:bg-muted"
              >
                <img
                  src={`/icons/lucide/${icon.name}.svg`}
                  width={24}
                  height={24}
                  alt=""
                  loading="lazy"
                  className="dark:invert"
                />
                <span
                  className="w-full truncate text-center text-[11px]"
                  title={icon.name}
                >
                  {icon.name}
                </span>
              </button>
            ))}
          </div>
          {results.length === 0 ? (
            <div className="py-16 text-center">
              <p className="text-sm text-muted-foreground">
                No icons match this search.
              </p>
              <Button
                variant="outline"
                className="mt-4"
                onClick={() => {
                  setQuery("")
                  setPage(0)
                }}
              >
                Clear search
              </Button>
            </div>
          ) : (
            <nav
              className="mt-5 flex items-center justify-between gap-3"
              aria-label="Icon pages"
            >
              <Button
                variant="outline"
                size="sm"
                disabled={page === 0}
                onClick={() => setPage(page - 1)}
              >
                Previous
              </Button>
              <p className="text-xs text-muted-foreground">
                Page {page + 1} of {pageCount}
              </p>
              <Button
                variant="outline"
                size="sm"
                disabled={page + 1 >= pageCount}
                onClick={() => setPage(page + 1)}
              >
                Next
              </Button>
            </nav>
          )}
        </section>

        <aside
          ref={detailsRef}
          tabIndex={-1}
          aria-label="Selected icon"
          className="min-w-0 scroll-mt-20 lg:sticky lg:top-20 lg:col-start-2 lg:row-start-1"
        >
          <a
            href="#icon-search"
            className="mb-4 inline-block text-sm underline underline-offset-4 lg:hidden"
          >
            Back to icons
          </a>
          <h2 className="font-medium">{selected.name}</h2>
          <div className="mt-3 overflow-hidden rounded-xl border">
            <GpuPreview
              component={{
                slug: "icons",
                name: selected.name,
                description: "",
              }}
              icon={selected.name}
            />
          </div>
          <p className="mt-3 text-xs leading-5 text-muted-foreground">
            Rendered by GPUI at 16, 24, 32, and 48 pixels.
          </p>
          <CodeBlock
            className="mt-4"
            value={`use gpui::{Styled, px, rgb};\nuse gpui_icons::{lucide, LucideIcon};\n\nlucide(${selected.rust})\n    .size(px(24.))\n    .text_color(rgb(0x171717))`}
          />
          <p className="mt-4 text-sm leading-6 text-muted-foreground">
            Register <code className="text-xs">LucideAssetSource</code> on your
            application. Set the icon color explicitly with your theme’s
            foreground color. Give icon-only controls a clear accessible label.
          </p>
          <a
            className="mt-3 inline-block text-sm underline underline-offset-4"
            href={`/icons/lucide/${selected.name}.svg`}
            download
          >
            Download original SVG
          </a>
          <p className="mt-5 text-xs leading-5 text-muted-foreground">
            Original Lucide assets under{" "}
            <a href="/icons/LICENSE" className="underline underline-offset-4">
              ISC and Feather MIT terms
            </a>
            . gpui-icons is an independent port.
          </p>
        </aside>
      </div>
    </DocsLayout>
  )
}
