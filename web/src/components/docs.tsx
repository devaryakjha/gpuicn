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
import { cn } from "@/lib/utils"

const registryOrigin = "https://ui.imajha.com"

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
    <div className="mx-auto grid w-full max-w-6xl grid-cols-1 xl:grid-cols-[minmax(0,1fr)_13rem]">
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
          className="group outline-none"
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

export function ComponentPage({ component }: { component: CatalogComponent }) {
  const index = components.findIndex((item) => item.slug === component.slug)
  const previous = components[index - 1]
  const next = components[index + 1]
  const source = `use gpuicn::${rustModule(component.slug)};\n\ngpuicn::init(cx);\n// The installed ${component.name} source is yours to edit.`

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

      <ComponentExample component={component} source={source} />
      <InstallationTabs slug={component.slug} />

      <section id="usage" className="scroll-mt-20 pt-10">
        <h2 className="font-heading text-2xl font-semibold tracking-tight">
          Usage
        </h2>
        <p className="mt-3 leading-7 text-muted-foreground">
          Install the source, register gpuicn during app startup, and keep each
          component ID stable across renders.
        </p>
        <CodeBlock className="mt-4" value={source} />
      </section>

      <section id="api-reference" className="scroll-mt-20 pt-10">
        <h2 className="font-heading text-2xl font-semibold tracking-tight">
          API Reference
        </h2>
        <p className="mt-3 leading-7 text-muted-foreground">
          The visual port keeps Base GPUI behavior and exposes an idiomatic Rust
          API. The installed source remains editable.
        </p>
      </section>
    </DocsLayout>
  )
}

function ComponentExample({
  component,
  source,
}: {
  component: CatalogComponent
  source: string
}) {
  return (
    <Tabs defaultValue="preview" className="mt-8">
      <TabsList variant="line">
        <TabsTrigger value="preview">Preview</TabsTrigger>
        <TabsTrigger value="code">Code</TabsTrigger>
      </TabsList>
      <TabsContent
        value="preview"
        className="mt-2 overflow-hidden rounded-xl border bg-background"
      >
        <GpuPreview component={component} />
      </TabsContent>
      <TabsContent value="code" className="mt-2">
        <CodeBlock value={source} />
      </TabsContent>
    </Tabs>
  )
}

function GpuPreview({ component }: { component: CatalogComponent }) {
  const hostRef = React.useRef<HTMLDivElement>(null)
  const frameRef = React.useRef<HTMLIFrameElement>(null)
  const [width, setWidth] = React.useState(720)
  const [theme, setTheme] = React.useState<"light" | "dark">(() =>
    document.documentElement.classList.contains("dark") ? "dark" : "light"
  )
  const [status, setStatus] = React.useState<
    "loading" | "ready" | "failed" | "unsupported"
  >(() => ("gpu" in navigator ? "loading" : "unsupported"))
  const height = previewHeight(component.slug)

  React.useEffect(() => {
    const host = hostRef.current
    if (!host) return
    const observer = new ResizeObserver(([entry]) => {
      const next = Math.max(1, Math.round(entry.contentRect.width))
      setWidth((current) => (Math.abs(current - next) >= 16 ? next : current))
    })
    observer.observe(host)
    return () => observer.disconnect()
  }, [])

  React.useEffect(() => {
    const observer = new MutationObserver(() => {
      const next = document.documentElement.classList.contains("dark")
        ? "dark"
        : "light"
      setTheme(next)
    })
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["class"],
    })
    return () => observer.disconnect()
  }, [])

  React.useEffect(() => {
    const onMessage = (event: MessageEvent) => {
      if (event.source !== frameRef.current?.contentWindow) return
      if (event.data?.gpuicn === "preview-ready") setStatus("ready")
      if (event.data?.gpuicn === "preview-error") setStatus("failed")
    }
    window.addEventListener("message", onMessage)
    return () => window.removeEventListener("message", onMessage)
  }, [])

  const src = `/demo/index.html?demo=${component.slug}&theme=${theme}&width=${width}&height=${height}`
  const message = {
    loading: "Loading interactive GPUI preview...",
    ready: "",
    failed: "The GPUI preview did not start.",
    unsupported: "This browser does not expose WebGPU.",
  }[status]

  return (
    <div
      ref={hostRef}
      className="relative flex w-full items-center justify-center"
      style={{ height }}
    >
      <iframe
        ref={frameRef}
        src={src}
        title={`Interactive ${component.name} GPUI preview`}
        className={cn("size-full border-0", status !== "ready" && "invisible")}
      />
      {status !== "ready" ? (
        <div className="absolute inset-0 grid place-items-center p-6 text-center text-sm text-muted-foreground">
          {message}
        </div>
      ) : null}
    </div>
  )
}

export function InstallationTabs({ slug }: { slug: string }) {
  const command = `npx shadcn@latest add ${registryOrigin}/r/${slug}.json`
  const manual = `1. Download ${registryOrigin}/r/${slug}.json\n2. Copy its Rust source into src/ui/\n3. Add the module and initialize gpuicn::init(cx)`

  return (
    <section id="installation" className="scroll-mt-20 pt-10">
      <h2 className="font-heading text-2xl font-semibold tracking-tight">
        Installation
      </h2>
      <Tabs defaultValue="command" className="mt-4">
        <TabsList>
          <TabsTrigger value="command">Command</TabsTrigger>
          <TabsTrigger value="manual">Manual</TabsTrigger>
        </TabsList>
        <TabsContent value="command">
          <CodeBlock className="mt-2" value={command} />
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
}: {
  value: string
  className?: string
}) {
  const [copied, setCopied] = React.useState(false)

  const copy = async () => {
    await navigator.clipboard.writeText(value)
    setCopied(true)
    window.setTimeout(() => setCopied(false), 1200)
  }

  return (
    <div
      className={cn("relative overflow-hidden rounded-xl bg-muted", className)}
    >
      <pre className="overflow-x-auto p-4 pr-12 font-mono text-xs leading-6">
        <code>{value}</code>
      </pre>
      <Button
        variant="ghost"
        size="icon-sm"
        className="absolute top-2 right-2"
        onClick={copy}
        aria-label="Copy code"
      >
        {copied ? <CheckIcon /> : <ClipboardIcon />}
      </Button>
    </div>
  )
}
