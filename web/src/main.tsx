import { StrictMode } from "react"
import { createRoot } from "react-dom/client"
import { createRouter, RouterProvider } from "@tanstack/react-router"

import "./index.css"
import { ThemeProvider } from "@/components/theme-provider"
import { TooltipProvider } from "@/components/ui/tooltip"
import { routeTree } from "./routeTree.gen"

const router = createRouter({
  routeTree,
  defaultPreload: "intent",
  scrollRestoration: true,
})

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router
  }
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ThemeProvider storageKey="gpuicn-theme">
      <TooltipProvider>
        <RouterProvider router={router} />
      </TooltipProvider>
    </ThemeProvider>
  </StrictMode>
)
