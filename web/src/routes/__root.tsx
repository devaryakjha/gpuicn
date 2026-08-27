import { createRootRoute } from "@tanstack/react-router"

import { SiteShell } from "@/components/site-shell"

export const Route = createRootRoute({
  component: SiteShell,
  notFoundComponent: () => (
    <main className="grid min-h-[calc(100svh-3.5rem)] place-items-center px-6 text-center">
      <div>
        <p className="text-sm text-muted-foreground">404</p>
        <h1 className="mt-2 text-2xl font-semibold">Page not found</h1>
      </div>
    </main>
  ),
})
