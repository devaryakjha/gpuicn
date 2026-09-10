import * as React from "react"
import { Link } from "@tanstack/react-router"
import {
  CheckIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
  ClipboardIcon,
} from "lucide-react"

import { Button, buttonVariants } from "@/components/ui/button"
import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import type { CatalogComponent } from "@/lib/catalog"
import { components, previewHeight, rustModule } from "@/lib/catalog"
import type { ComponentDocs } from "@/lib/component-docs"
import { cn } from "@/lib/utils"
import { highlight, type CodeLanguage } from "@/lib/highlight"


export function BetaNotice() {
  return (
    <aside
      id="beta"
      aria-label="Beta status"
      className="mt-6 scroll-mt-20 rounded-lg border border-dashed px-4 py-3 text-sm leading-6 text-muted-foreground"
    >
      <strong className="font-medium text-foreground">In beta.</strong> APIs and
      styling may change as the library develops. Pin dependency revisions and
      review updates before replacing your installed source.{" "}
      <a
        href="https://github.com/devaryakjha/gpuicn/releases"
        className="underline underline-offset-4"
      >
        Follow releases
      </a>
      .
    </aside>
  )
}

export function PageHeader({
  eyebrow,
  title,
  description,
}: {
  eyebrow: string
  title: string
  description: string
}) {
  React.useEffect(() => {
    document.title = `${title} - gpuicn`
  }, [title])

  return (
    <header className="flex flex-col gap-2">
      <p className="text-xs font-medium tracking-widest text-muted-foreground uppercase">
        {eyebrow}
      </p>
      <h1 className="font-heading text-3xl font-semibold tracking-tight sm:text-4xl">
        {title}
      </h1>
      <p className="max-w-2xl text-base leading-7 text-muted-foreground">
        {description}
      </p>
    </header>
  )
}

export function DocsLayout({
  children,
  toc,
}: {
  children: React.ReactNode
  toc?: React.ReactNode
}) {
  return (
    <div
      className={cn(
        "mx-auto grid w-full max-w-6xl grid-cols-1",
        toc && "xl:grid-cols-[minmax(0,1fr)_13rem]"
      )}
    >
      <article className="min-w-0 px-5 py-10 sm:px-8 lg:px-12 lg:py-14">
        {children}
      </article>
      {toc ? (
        <aside className="sticky top-20 hidden h-fit px-6 py-14 text-sm xl:block">
          {toc}
        </aside>
      ) : null}
    </div>
  )
}

export function TableOfContents() {
  return (
    <nav
      className="flex flex-col gap-2 text-muted-foreground"
      aria-label="On this page"
    >
      <p className="font-medium text-foreground">On This Page</p>
      <a className="hover:text-foreground" href="#installation">
        Installation
      </a>
      <a className="hover:text-foreground" href="#usage">
        Usage
      </a>
      <a className="hover:text-foreground" href="#api-reference">
        API Reference
      </a>
      <a className="hover:text-foreground" href="#platform-notes">
        Platform notes
      </a>
    </nav>
  )
}

export function ComponentGrid() {
  return (
    <section
      id="components"
      className="grid scroll-mt-20 grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3"
    >
      {components.map((component) => (
        <Link
          key={component.slug}
          to="/components/$slug"
          params={{ slug: component.slug }}
          className="group rounded-xl focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring"
        >
          <Card
            className="h-full transition-colors group-hover:bg-muted/50 group-focus-visible:ring-ring"
            size="sm"
          >
            <CardHeader>
              <CardTitle>{component.name}</CardTitle>
              <CardDescription>{component.description}</CardDescription>
            </CardHeader>
          </Card>
        </Link>
      ))}
    </section>
  )
}

export function ComponentPage({
  component,
  docs,
}: {
  component: CatalogComponent
  docs: ComponentDocs
}) {
  const index = components.findIndex((item) => item.slug === component.slug)
  const previous = components[index - 1]
  const next = components[index + 1]
  const source = docs.preview

  return (
    <DocsLayout toc={<TableOfContents />}>
      <div className="flex items-start gap-4">
        <PageHeader
          eyebrow="Components"
          title={component.name}
          description={component.description}
        />
        <nav
          className="ml-auto hidden shrink-0 gap-1 sm:flex"
          aria-label="Component pages"
        >
          {previous ? (
            <Link
              to="/components/$slug"
              params={{ slug: previous.slug }}
              aria-label={`Previous: ${previous.name}`}
              className={buttonVariants({ variant: "outline", size: "icon" })}
            >
              <ChevronLeftIcon />
            </Link>
          ) : null}
          {next ? (
            <Link
              to="/components/$slug"
              params={{ slug: next.slug }}
              aria-label={`Next: ${next.name}`}
              className={buttonVariants({ variant: "outline", size: "icon" })}
            >
              <ChevronRightIcon />
            </Link>
          ) : null}
        </nav>
      </div>

      <ComponentExample component={component} source={source} examples={docs.examples} />
      <InstallationTabs slug={component.slug} modules={docs.modules} />

      <section id="usage" className="scroll-mt-20 pt-10">
        <h2 className="font-heading text-2xl font-semibold tracking-tight">
          Usage
        </h2>
        <p className="mt-3 leading-7 text-muted-foreground">
          Copy this into your view module and return{" "}
          <code>{docs.usageCall}</code> from its <code>render</code> method.
          Complete the{" "}
          <Link to="/installation" className="underline underline-offset-4">
            app setup
          </Link>{" "}
          once, and keep IDs unique among siblings.
        </p>
        <CodeBlock className="mt-4" value={docs.usage} />
      </section>

      <section id="api-reference" className="scroll-mt-20 pt-10">
        <h2 className="font-heading text-2xl font-semibold tracking-tight">
          API Reference
        </h2>
        <p className="mt-3 leading-7 text-muted-foreground">
          Public constructors and methods from the source you install. Functions
          that return GPUI Kit parts also support those parts’ builder methods.
        </p>
        <div className="mt-4 divide-y rounded-xl border px-4">
          {docs.api.map((entry, index) => (
            <div key={index} className="py-4">
              <pre className="overflow-x-auto font-mono text-xs leading-6">
                <code>{entry.signature}</code>
              </pre>
              {entry.description ? (
                <p className="mt-2 text-sm leading-6 text-muted-foreground">
                  {entry.description}
                </p>
              ) : null}
            </div>
          ))}
        </div>
        <details className="mt-5">
          <summary className="cursor-pointer text-sm font-medium">
            View editable Rust source
          </summary>
          <CodeBlock className="mt-3" value={docs.source} />
        </details>
      </section>

      <section id="platform-notes" className="scroll-mt-20 pt-10">
        <h2 className="font-heading text-2xl font-semibold tracking-tight">
          Platform notes
        </h2>
        <p className="mt-3 leading-7 text-muted-foreground">
          GPUI handles rendering and input. These notes record the current
          platform differences, including accessibility and interaction limits.
        </p>
        <PlatformNotes content={docs.parityHtml} />
      </section>
    </DocsLayout>
  )
}

export function PlatformNotes({ content }: { content: string }) {
  return (
    <div
      className="platform-notes mt-4 space-y-3 text-sm leading-6 text-muted-foreground"
      dangerouslySetInnerHTML={{ __html: content }}
    />
  )
}

function ComponentExample({
  component,
  source,
  examples,
}: {
  component: CatalogComponent
  source: string
  examples?: Record<string, string>
}) {
  const [sidebarExample, setSidebarExample] = React.useState("workspace")
  const sidebarExamples = [
    ["workspace", "Workspace", "Inset layout with a team switcher, nested projects, a toggle rail, and an account menu."],
    ["docs", "Documentation", "Searchable documentation tree with independent sections and off-canvas navigation."],
    ["mail", "Mail", "A folder rail beside a searchable message list. Filter unread mail and open a message."],
    ["floating", "Floating right", "A rounded sidebar on the right, with an icon-only collapsed state."],
    ["mobile", "Mobile sheet", "Modal navigation with its own open state, focus containment, and Escape dismissal."],
    ["loading", "Loading", "Fixed navigation with stable placeholders. Load the menu to replace them with working controls."],
  ]
  return (
    <Tabs defaultValue="preview" className="relative mt-8">
      <TabsList variant="line">
        <TabsTrigger value="preview">Preview</TabsTrigger>
        <TabsTrigger value="code">Demo code</TabsTrigger>
      </TabsList>
      <TabsContent
        value="preview"
        keepMounted
        render={(props) => <div {...props} hidden={false} />}
        className="mt-2 overflow-hidden rounded-xl border border-[color-mix(in_oklab,var(--foreground)_10%,var(--background))] bg-background data-[hidden]:invisible data-[hidden]:absolute data-[hidden]:w-full"
      >
        {component.slug === "sidebar" ? (
          <div className="border-b p-3">
            <div className="flex flex-wrap gap-1" role="group" aria-label="Sidebar examples">
              {sidebarExamples.map(([value, label]) => (
                <Button key={value} size="sm" variant={sidebarExample === value ? "secondary" : "ghost"}
                  aria-pressed={sidebarExample === value} onClick={() => setSidebarExample(value)}>{label}</Button>
              ))}
            </div>
            <p className="mt-2 text-sm text-muted-foreground">{sidebarExamples.find(([value]) => value === sidebarExample)?.[2]}</p>
          </div>
        ) : null}
        <GpuPreview key={component.slug === "sidebar" ? sidebarExample : component.slug}
          component={component} example={component.slug === "sidebar" ? sidebarExample : undefined} />
      </TabsContent>
      <TabsContent value="code" className="mt-2">
        <p className="mb-3 text-sm leading-6 text-muted-foreground">
          Full preview, including its layout and demo state. For a small
          example, see{" "}
          <a href="#usage" className="underline underline-offset-4">
            Usage
          </a>{" "}
          below.{" "}
          <a
            href="/examples/showcase.rs"
            download
            className="underline underline-offset-4"
          >
            Download the complete showcase
          </a>{" "}
          for its imports and helpers.
        </p>
        <CodeBlock value={examples?.[sidebarExample] ?? source} />
      </TabsContent>
    </Tabs>
  )
}

export function GpuPreview({
  component,
  icon,
  example,
}: {
  component: CatalogComponent
  icon?: string
  example?: string
}) {
  const hostRef = React.useRef<HTMLDivElement>(null)
  const [width, setWidth] = React.useState<number | null>(null)
  const [theme, setTheme] = React.useState<"light" | "dark">(() =>
    document.documentElement.classList.contains("dark") ? "dark" : "light"
  )
  const height = previewHeight(component.slug)

  React.useLayoutEffect(() => {
    const host = hostRef.current
    if (host)
      setWidth(Math.max(1, Math.round(host.getBoundingClientRect().width)))
  }, [])

  React.useEffect(() => {
    const observer = new MutationObserver(() => {
      setTheme(
        document.documentElement.classList.contains("dark") ? "dark" : "light"
      )
    })
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["class"],
    })
    return () => observer.disconnect()
  }, [])

  const query = new URLSearchParams({
    demo: component.slug,
    theme,
    width: String(width),
    height: String(height),
  })
  if (icon) query.set("icon", icon)
  if (example) query.set("example", example)
  const src = `/demo/index.html?${query}`

  return (
    <div ref={hostRef} className="relative w-full" style={{ height }}>
      {width !== null ? (
        <PreviewFrame
          key={component.slug}
          src={src}
          name={component.name}
          theme={theme}
          icon={icon}
        />
      ) : null}
    </div>
  )
}

function PreviewFrame({
  src,
  name,
  theme,
  icon,
}: {
  src: string
  name: string
  theme: "light" | "dark"
  icon?: string
}) {
  const frameRef = React.useRef<HTMLIFrameElement>(null)
  const [initialSrc] = React.useState(src)
  const [attempt, setAttempt] = React.useState(0)
  const [status, setStatus] = React.useState<
    "loading" | "ready" | "failed" | "unsupported"
  >(() => ("gpu" in navigator ? "loading" : "unsupported"))

  React.useEffect(() => {
    if (!("gpu" in navigator)) return
    const timeout = window.setTimeout(() => setStatus("failed"), 45000)
    const onMessage = (event: MessageEvent) => {
      if (
        event.origin !== window.location.origin ||
        event.source !== frameRef.current?.contentWindow
      )
        return
      if (
        event.data?.gpuicn === "preview-ready" ||
        event.data?.gpuicn === "preview-error"
      ) {
        window.clearTimeout(timeout)
        setStatus(event.data.gpuicn === "preview-ready" ? "ready" : "failed")
      }
    }
    window.addEventListener("message", onMessage)
    return () => {
      window.clearTimeout(timeout)
      window.removeEventListener("message", onMessage)
    }
  }, [attempt])

  React.useEffect(() => {
    if (status === "ready") {
      frameRef.current?.contentWindow?.postMessage(
        { gpuicn: "preview-update", theme, icon: icon ?? "" },
        window.location.origin
      )
    }
  }, [status, theme, icon])

  return (
    <>
      {status !== "unsupported" ? (
        <iframe
          key={attempt}
          ref={frameRef}
          src={initialSrc}
          title={`Interactive ${name} GPUI preview`}
          onError={() => setStatus("failed")}
          className={cn(
            "size-full border-0",
            status !== "ready" && "invisible"
          )}
        />
      ) : null}
      {status !== "ready" ? (
        <div
          className="absolute inset-0 flex flex-col items-center justify-center gap-3 p-6 text-center text-sm text-muted-foreground"
          role="status"
        >
          <p>
            {status === "loading"
              ? "Loading interactive GPUI preview…"
              : status === "unsupported"
                ? "Interactive previews need a browser with WebGPU. You can still read and copy the Rust code."
                : "The preview could not start. Try loading it again."}
          </p>
          {status === "failed" ? (
            <Button
              variant="outline"
              size="sm"
              onClick={() => {
                setStatus("loading")
                setAttempt((value) => value + 1)
              }}
            >
              Retry preview
            </Button>
          ) : null}
        </div>
      ) : null}
    </>
  )
}

export function InstallationTabs({
  slug,
  modules = ["theme", rustModule(slug)],
}: {
  slug: string
  modules?: string[]
}) {
  const command = `gpuicn add ${slug}`
  const manual = `// src/ui/mod.rs\n${modules.map((module) => `pub mod ${module};`).join("\n")}\n\n// src/main.rs\nmod ui;`

  return (
    <section id="installation" className="scroll-mt-20 pt-10">
      <h2 className="font-heading text-2xl font-semibold tracking-tight">
        Installation
      </h2>
      <p className="mt-3 text-sm leading-6 text-muted-foreground">
        Complete the{" "}
        <Link className="underline underline-offset-4" to="/installation">
          one-time Rust setup
        </Link>
        , then add the source. The native CLI maintains the installed module declarations.
      </p>
      <Tabs defaultValue="command" className="mt-4">
        <TabsList>
          <TabsTrigger value="command">Command</TabsTrigger>
          <TabsTrigger value="manual">Modules</TabsTrigger>
        </TabsList>
        <TabsContent value="command">
          <CodeBlock className="mt-2" value={command} language="bash" />
        </TabsContent>
        <TabsContent value="manual">
          <CodeBlock className="mt-2" value={manual} />
        </TabsContent>
      </Tabs>
    </section>
  )
}

export function CodeBlock({
  value,
  className,
  language = "rust",
}: {
  value: string
  className?: string
  language?: CodeLanguage
}) {
  const html = React.useMemo(
    () => highlight(value, language),
    [value, language]
  )
  const [copied, setCopied] = React.useState(false)
  const [copyFailed, setCopyFailed] = React.useState(false)

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(value)
      setCopyFailed(false)
      setCopied(true)
      window.setTimeout(() => setCopied(false), 1200)
    } catch {
      setCopyFailed(true)
    }
  }

  return (
    <div
      className={cn("relative overflow-hidden rounded-xl bg-muted", className)}
    >
      <pre
        tabIndex={0}
        className="max-h-[32rem] overflow-auto p-4 pr-12 font-mono text-xs leading-6 focus-visible:outline-2 focus-visible:outline-ring"
      >
        <code
          className={`syntax-code language-${language}`}
          dangerouslySetInnerHTML={{ __html: html }}
        />
      </pre>
      <Button
        variant="ghost"
        size="icon-sm"
        className="absolute top-2 right-2"
        onClick={copy}
        aria-label={copied ? "Copied" : "Copy code"}
      >
        {copied ? <CheckIcon /> : <ClipboardIcon />}
      </Button>
      <span
        role="status"
        className={
          copyFailed ? "block px-4 pb-3 text-xs text-destructive" : "sr-only"
        }
      >
        {copyFailed
          ? "Copy failed. Select the code and copy it manually."
          : copied
            ? "Copied to clipboard."
            : ""}
      </span>
    </div>
  )
}
