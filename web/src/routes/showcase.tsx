import * as React from "react"
import { createFileRoute } from "@tanstack/react-router"
import { ArrowDownToLineIcon } from "lucide-react"
import { DocsLayout, PageHeader } from "@/components/docs"
import { buttonVariants } from "@/components/ui/button"

export const Route = createFileRoute("/showcase")({ component: ShowcasePage })

type Download = {
  url: string
  architecture: "arm64" | "x86_64"
  bytes: number
  sha256: string
  build: string
  notarized: boolean
  version: string
}

function ShowcasePage() {
  const [preview, setPreview] = React.useState<"light" | "chat" | "dark">("light")
  const [download, setDownload] = React.useState<Download | null>(null)
  const [loading, setLoading] = React.useState(true)
  React.useEffect(() => {
    document.title = "Workspace example app - gpuicn"
    const controller = new AbortController()
    fetch("/downloads/showcase.json", { signal: controller.signal })
      .then((response) => {
        if (!response.ok) throw new Error("Download unavailable")
        return response.json()
      })
      .then((data: Download) => {
        if (!/^\/downloads\/gpuicn-workspace-macos-(arm64|x86_64)\.zip$/.test(data.url) ||
            !["arm64", "x86_64"].includes(data.architecture) ||
            !Number.isSafeInteger(data.bytes) || data.bytes <= 0 ||
            !/^[a-f0-9]{64}$/.test(data.sha256) || typeof data.build !== "string" ||
            typeof data.notarized !== "boolean" || typeof data.version !== "string") throw new Error("Invalid download")
        setDownload(data)
      })
      .catch(() => {})
      .finally(() => { if (!controller.signal.aborted) setLoading(false) })
    return () => controller.abort()
  }, [])

  return (
    <DocsLayout>
      <PageHeader eyebrow="Desktop app" title="A small workspace. A real native app."
        description="Plan projects, manage tasks, and keep a local conversation for each project. An example of gpuicn components working together in an app you can use." />
      <figure className="mt-8">
        <div className="mb-4 flex gap-2" aria-label="Native app previews">
          {([ ["light", "Tasks"], ["chat", "Local chat"], ["dark", "Dark appearance"] ] as const).map(([value, label]) => (
            <button key={value} type="button" aria-pressed={preview === value}
              className={buttonVariants({ variant: preview === value ? "secondary" : "ghost", size: "sm" })}
              onClick={() => setPreview(value)}>{label}</button>
          ))}
        </div>
        <a href={`/workspace-${preview}.jpg`} target="_blank" rel="noreferrer" aria-label="Open full-size native app screenshot">
          <img src={`/workspace-${preview}.jpg`} width="1188" height="768"
            alt={`The native gpuicn Workspace ${preview === "chat" ? "local chat" : `${preview} task view`}`}
            className="w-full rounded-xl border shadow-sm" />
        </a>
        <figcaption className="mt-3 text-xs text-muted-foreground">Captured in the native macOS app. Open the image to see it at full size.</figcaption>
      </figure>
      <div className="mt-10 rounded-xl border p-6 sm:p-8">
        <div className="flex items-center gap-3">
          <img src="/brand/gpuicn-panels.png" alt="" className="size-12" />
          <h2 className="text-xl font-semibold tracking-tight">gpuicn Workspace beta for macOS</h2>
        </div>
        <p className="mt-3 text-sm leading-6 text-muted-foreground">
          Create and edit tasks, mark them complete, and post project messages. Your workspace saves on your Mac. No account or development tools needed.
        </p>
        {download ? <>
          <a href={download.url} download className={buttonVariants({ size: "lg", className: "mt-6" })}>
            <ArrowDownToLineIcon /> Download for {download.architecture === "arm64" ? "Apple silicon" : "Intel Mac"}
          </a>
          <p className="mt-3 text-xs text-muted-foreground">v{download.version} · macOS 12 or later · ZIP · {(download.bytes / 1024 / 1024).toFixed(1)} MB</p>
          {!download.notarized && <p className="mt-5 text-sm leading-6">
            Developer preview. This build is not notarized by Apple, so macOS may block it from opening.
            A signed public build is still pending.
          </p>}
          <p className="mt-5 text-sm leading-6 text-muted-foreground">Unzip the download, move gpuicn Workspace to Applications, and open it.</p>
          <details className="mt-5 text-xs text-muted-foreground">
            <summary className="cursor-pointer">Verify download checksum</summary>
            <code className="mt-2 block break-all">SHA-256: {download.sha256}</code>
          </details>
        </> : <p className="mt-6 text-sm text-muted-foreground">{loading ? "Checking download…" : "The macOS download is being prepared. You can run the workspace from source below."}</p>}
      </div>
      <p className="mt-6 text-sm leading-6 text-muted-foreground">Local chat is a project message log on this Mac. It does not connect to other people or generate replies.</p>
      <section className="mt-10 space-y-3">
        <h2 className="text-xl font-semibold tracking-tight">Run from source</h2>
        <p className="text-sm leading-6 text-muted-foreground">From the gpuicn checkout, launch the workspace:</p>
        <pre className="overflow-x-auto rounded-lg border bg-muted/30 p-4 text-sm"><code>cargo run --release -p gpuicn-showcase --features gpui_platform/runtime_shaders</code></pre>
      </section>
    </DocsLayout>
  )
}
