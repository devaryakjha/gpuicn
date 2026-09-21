import * as React from "react"
import { createFileRoute } from "@tanstack/react-router"
import { Menu } from "@base-ui/react/menu"
import { ArrowDownToLineIcon, ChevronDownIcon } from "lucide-react"
import { DocsLayout } from "@/components/docs"
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
      <header>
        <div className="flex items-center gap-4">
          <img src="/brand/gpuicn-panels.png" alt="" className="size-14 shrink-0 sm:size-16" />
          <div>
            <p className="mb-1 text-xs text-muted-foreground">Example app · Beta</p>
            <h1 className="text-3xl font-medium tracking-tight sm:text-4xl">gpuicn Workspace</h1>
          </div>
        </div>
        <p className="mt-5 max-w-xl text-base leading-7 text-muted-foreground">
          A native example app built with gpuicn. Organize projects, tasks and notes, all saved on your computer.
        </p>
        <div className="mt-6 flex flex-wrap items-center gap-3">
          {download ? <a href={download.url} download className={buttonVariants({ size: "lg", variant: "outline", className: "h-10 w-full px-4 sm:w-auto" })}>
            <ArrowDownToLineIcon /> Download for macOS
          </a> : <span className="text-sm text-muted-foreground">{loading ? "Checking macOS download…" : "macOS download unavailable"}</span>}
          <Menu.Root>
            <Menu.Trigger className={buttonVariants({ size: "lg", variant: "outline", className: "h-10 w-full px-4 sm:w-auto" })}>
              <ArrowDownToLineIcon /> Download for Linux <ChevronDownIcon />
            </Menu.Trigger>
            <Menu.Portal>
              <Menu.Positioner sideOffset={8} align="start" className="z-50">
                <Menu.Popup className="w-72 max-w-[calc(100vw-2rem)] rounded-lg border bg-popover p-1 text-popover-foreground shadow-lg outline-none">
                  <Menu.LinkItem closeOnClick href="https://github.com/devaryakjha/gpuicn/releases/download/v0.5.0-beta.4/gpuicn-workspace-linux-x86_64.tar.gz" className="block rounded-md px-3 py-2.5 outline-none data-highlighted:bg-accent data-highlighted:text-accent-foreground">
                    <span className="block text-sm font-medium">x86-64 · Intel / AMD</span>
                    <span className="block text-xs leading-5 text-muted-foreground">Limited Ubuntu 24.04 startup checks</span>
                  </Menu.LinkItem>
                  <Menu.LinkItem closeOnClick href="https://github.com/devaryakjha/gpuicn/releases/download/v0.5.0-beta.4/gpuicn-workspace-linux-aarch64.tar.gz" className="block rounded-md px-3 py-2.5 outline-none data-highlighted:bg-accent data-highlighted:text-accent-foreground">
                    <span className="block text-sm font-medium">ARM64</span>
                    <span className="block text-xs leading-5 text-muted-foreground">Built, not runtime-tested</span>
                  </Menu.LinkItem>
                </Menu.Popup>
              </Menu.Positioner>
            </Menu.Portal>
          </Menu.Root>
        </div>
        <p className="mt-3 text-xs leading-5 text-muted-foreground">
          macOS 12+{download && ` · ${download.architecture === "arm64" ? "Apple silicon" : "Intel"}`}<span className="mx-2" aria-hidden="true">/</span>Linux preview · needs testing
        </p>
        {download && !download.notarized && <p className="mt-3 text-sm text-muted-foreground">The macOS build is not notarized. macOS may block it from opening.</p>}
      </header>
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
      <section className="mt-8 grid gap-6 border-t pt-6 sm:grid-cols-2 sm:gap-10">
        <div>
          <h2 className="text-sm font-medium">A local workspace</h2>
          <p className="mt-2 text-sm leading-6 text-muted-foreground">No account needed. Projects, tasks and messages stay on your computer. Local chat is a project message log; it does not connect to other people or generate replies.</p>
        </div>
        <div>
          <h2 className="text-sm font-medium">Linux is experimental</h2>
          <p className="mt-2 text-sm leading-6 text-muted-foreground">Requires glibc 2.39+ and a working Vulkan driver. x86-64 has had limited Ubuntu 24.04 startup checks; ARM64 has not been runtime-tested. Graphics performance, Wayland and other distributions still need testing.</p>
        </div>
      </section>
      <details className="mt-8 border-y py-4 text-sm">
        <summary className="cursor-pointer font-medium focus-visible:outline-ring">Installation and checksums</summary>
        <div className="mt-4 space-y-3 leading-6 text-muted-foreground">
          <p>On macOS, unzip the download, move gpuicn Workspace to Applications, and open it.</p>
          {download && <>
            <p className="text-xs">macOS v{download.version} · ZIP · {(download.bytes / 1024 / 1024).toFixed(1)} MB</p>
            <code className="block break-all text-xs">SHA-256: {download.sha256}</code>
          </>}
          <p>On Linux, extract the archive and run <code>./gpuicn-workspace</code> from its folder.</p>
          <a className="inline-block underline underline-offset-4" href="https://github.com/devaryakjha/gpuicn/releases/tag/v0.5.0-beta.4">Linux build details, dependencies and checksums</a>
        </div>
      </details>
      <details className="border-b py-4 text-sm">
        <summary className="cursor-pointer font-medium focus-visible:outline-ring">Run from source</summary>
        <div className="mt-4 space-y-3">
        <p className="text-sm leading-6 text-muted-foreground">From the gpuicn checkout, launch the workspace on macOS:</p>
        <pre className="overflow-x-auto rounded-lg border bg-muted/30 p-4 text-sm"><code>cargo run --release -p gpuicn-showcase --features gpui_platform/runtime_shaders</code></pre>
        <p className="text-sm leading-6 text-muted-foreground">On Linux, with the system build dependencies installed:</p>
        <pre className="overflow-x-auto rounded-lg border bg-muted/30 p-4 text-sm"><code>cargo run --release -p gpuicn-showcase --features gpui_platform/x11,gpui_platform/wayland</code></pre>
        </div>
      </details>
    </DocsLayout>
  )
}
