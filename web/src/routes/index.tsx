import * as React from "react"
import { createFileRoute, Link } from "@tanstack/react-router"
import { ArrowRightIcon, ArrowUpRightIcon } from "lucide-react"

import { GpuPreview } from "@/components/docs"
import { buttonVariants } from "@/components/ui/button"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { components } from "@/lib/catalog"
import { cn } from "@/lib/utils"

export const Route = createFileRoute("/")({ component: LandingPage })

const featured = ["button", "input", "slider", "checkbox"].map((slug) =>
  components.find((component) => component.slug === slug)!
)

function LandingPage() {
  React.useEffect(() => {
    document.title = "gpuicn — Native GPUI components · Beta"
  }, [])

  return (
    <div className="mx-auto w-full max-w-7xl px-5 sm:px-8 lg:px-12">
      <section className="flex flex-col items-center pt-16 pb-14 text-center sm:pt-24 sm:pb-20">
        <Link
          to="/installation"
          hash="beta"
          className="mb-7 inline-flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground"
        >
          <span className="rounded-md border px-1.5 py-0.5 text-[10px] font-medium tracking-wide uppercase">
            Beta
          </span>
          A growing library for GPUI
          <ArrowRightIcon className="size-3.5" />
        </Link>
        <h1 className="max-w-4xl text-4xl font-semibold tracking-[-0.055em] text-balance sm:text-6xl lg:text-7xl">
          Build native apps.
          <br />
          Make them your own.
        </h1>
        <p className="mt-6 max-w-xl text-base leading-7 text-pretty text-muted-foreground sm:text-lg sm:leading-8">
          Familiar components, built with Rust and GPUI. Add the source, shape
          the details, and build an app that feels like yours.
        </p>
        <div className="mt-8 flex flex-wrap justify-center gap-3">
          <Link
            to="/installation"
            className={cn(buttonVariants({ size: "lg" }), "px-5")}
          >
            Get started <ArrowRightIcon data-icon="inline-end" />
          </Link>
          <Link
            to="/showcase"
            className={cn(
              buttonVariants({ variant: "outline", size: "lg" }),
              "px-5"
            )}
          >
            Download example app
          </Link>
        </div>
      </section>
      <section
        aria-label="Try the components"
        className="overflow-hidden rounded-xl border"
      >
        <Tabs defaultValue="button" className="gap-0">
          <div className="flex flex-wrap items-center justify-between gap-3 border-b px-4 py-3 sm:px-6">
            <TabsList variant="line" aria-label="Featured components">
              {featured.map((component) => (
                <TabsTrigger
                  key={component.slug}
                  value={component.slug}
                  className="px-3"
                >
                  {component.name}
                </TabsTrigger>
              ))}
            </TabsList>
            <span className="hidden text-xs text-muted-foreground sm:block">
              Live GPUI · Try it out
            </span>
          </div>
          {featured.map((component) => (
            <TabsContent key={component.slug} value={component.slug}>
              <GpuPreview component={component} />
              <div className="flex items-center justify-between gap-4 border-t px-4 py-4 sm:px-6">
                <p className="text-xs text-muted-foreground sm:text-sm">
                  {component.description}
                </p>
                <Link
                  to="/components/$slug"
                  params={{ slug: component.slug }}
                  className="inline-flex shrink-0 items-center gap-1 text-sm font-medium underline-offset-4 hover:underline"
                >
                  View code <ArrowUpRightIcon className="size-4" />
                </Link>
              </div>
            </TabsContent>
          ))}
        </Tabs>
      </section>
      <section
        className="grid gap-8 py-14 sm:grid-cols-3 sm:gap-10 sm:py-20"
        aria-label="Explore gpuicn"
      >
        <Link to="/introduction" hash="components" className="group">
          <h2 className="flex items-center gap-2 text-sm font-medium">
            {components.length} component families{" "}
            <ArrowUpRightIcon className="size-3.5 text-muted-foreground group-hover:text-foreground" />
          </h2>
          <p className="mt-3 text-sm leading-6 text-muted-foreground">
            From buttons to command menus. Familiar shadcn styling with native
            GPUI behavior.
          </p>
        </Link>
        <Link to="/registry" className="group">
          <h2 className="flex items-center gap-2 text-sm font-medium">
            Your source code{" "}
            <ArrowUpRightIcon className="size-3.5 text-muted-foreground group-hover:text-foreground" />
          </h2>
          <p className="mt-3 text-sm leading-6 text-muted-foreground">
            Add only what you need from the registry. Read it, change it, and
            keep it in your app.
          </p>
        </Link>
        <Link to="/icons" className="group">
          <h2 className="flex items-center gap-2 text-sm font-medium">
            Lucide, ready for GPUI{" "}
            <ArrowUpRightIcon className="size-3.5 text-muted-foreground group-hover:text-foreground" />
          </h2>
          <p className="mt-3 text-sm leading-6 text-muted-foreground">
            Browse the icon library and copy the Rust code. One visual language
            for your whole interface.
          </p>
        </Link>
      </section>
      <footer className="flex flex-wrap items-center justify-between gap-4 border-t py-7 text-xs text-muted-foreground">
        <p>Built with Rust and GPUI. Open source, from the start.</p>
        <a
          href="https://github.com/devaryakjha/gpuicn"
          className="hover:text-foreground"
        >
          Source on GitHub ↗
        </a>
      </footer>
    </div>
  )
}
