import * as React from "react"
import {
  Link,
  Outlet,
  useNavigate,
  useRouterState,
} from "@tanstack/react-router"
import { GitBranchIcon, MoonIcon, SearchIcon, SunIcon } from "lucide-react"

import { useTheme } from "@/components/theme-provider"
import { Button, buttonVariants } from "@/components/ui/button"
import {
  Command,
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command"
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarInset,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarProvider,
  SidebarRail,
  SidebarTrigger,
  useSidebar,
} from "@/components/ui/sidebar"
import { components, sections } from "@/lib/catalog"
import { cn } from "@/lib/utils"

export function SiteShell() {
  const [searchOpen, setSearchOpen] = React.useState(false)

  return (
    <SidebarProvider>
      <SiteHeader onSearch={() => setSearchOpen(true)} />
      <CatalogSidebar />
      <SidebarInset id="main-content" className="min-w-0 pt-14">
        <Outlet />
      </SidebarInset>
      <CatalogSearch open={searchOpen} onOpenChange={setSearchOpen} />
    </SidebarProvider>
  )
}

function SiteHeader({ onSearch }: { onSearch: () => void }) {
  const { theme, setTheme } = useTheme()
  const pathname = useRouterState({
    select: (state) => state.location.pathname,
  })

  const toggleTheme = () => {
    const dark = document.documentElement.classList.contains("dark")
    setTheme(dark ? "light" : "dark")
  }

  return (
    <header className="fixed inset-x-0 top-0 z-40 h-14 border-b bg-background/95 backdrop-blur supports-backdrop-filter:bg-background/80">
      <a
        href="#main-content"
        className="sr-only focus:not-sr-only focus:absolute focus:top-2 focus:left-3 focus:z-50 focus:rounded-md focus:bg-background focus:px-4 focus:py-2 focus:outline-2"
      >
        Skip to content
      </a>
      <div className="flex h-full items-center gap-2 px-3 sm:px-4">
        <SidebarTrigger className="md:hidden" />
        <Link
          to="/"
          className="mr-2 flex items-center gap-2 text-sm font-semibold tracking-tight"
        >
          <img
            src="/brand/gpuicn-panels.png"
            alt=""
            width={28}
            height={28}
            className="size-7 shrink-0 mix-blend-multiply contrast-125 hue-rotate-180 invert dark:mix-blend-screen dark:hue-rotate-0 dark:invert-0"
          />
          gpuicn
        </Link>
        <nav
          className="hidden items-center gap-1 md:flex"
          aria-label="Main navigation"
        >
          <Link
            to="/"
            className={cn(
              buttonVariants({ variant: "ghost", size: "sm" }),
              pathname !== "/" && "text-muted-foreground"
            )}
          >
            Docs
          </Link>
          <Link
            to="/"
            hash="components"
            className={cn(
              buttonVariants({ variant: "ghost", size: "sm" }),
              !pathname.startsWith("/components") && "text-muted-foreground"
            )}
          >
            Components
          </Link>
          <Link
            to="/registry"
            className={cn(
              buttonVariants({ variant: "ghost", size: "sm" }),
              pathname !== "/registry" && "text-muted-foreground"
            )}
          >
            Registry
          </Link>
          <Link
            to="/icons"
            className={cn(
              buttonVariants({ variant: "ghost", size: "sm" }),
              pathname !== "/icons" && "text-muted-foreground"
            )}
          >
            Icons
          </Link>
        </nav>
        <div className="ml-auto flex min-w-0 items-center gap-1.5">
          <Button
            variant="outline"
            className="hidden w-56 justify-start text-muted-foreground lg:inline-flex"
            onClick={onSearch}
          >
            <SearchIcon data-icon="inline-start" />
            Search documentation...
            <kbd className="ml-auto font-mono text-[10px]">⌘K</kbd>
          </Button>
          <Button
            variant="ghost"
            size="icon"
            aria-label="Search documentation"
            className="lg:hidden"
            onClick={onSearch}
          >
            <SearchIcon />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            aria-label="Open gpuicn on GitHub"
            render={
              <a
                href="https://github.com/devaryakjha/gpuicn"
                target="_blank"
                rel="noreferrer"
              />
            }
          >
            <GitBranchIcon />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            aria-label={`Switch from ${theme} theme`}
            onClick={toggleTheme}
          >
            <SunIcon className="hidden dark:block" />
            <MoonIcon className="dark:hidden" />
          </Button>
          <Link
            to="/installation"
            className={cn(buttonVariants(), "hidden sm:inline-flex")}
          >
            Get started
          </Link>
        </div>
      </div>
    </header>
  )
}

function CatalogSidebar() {
  const pathname = useRouterState({
    select: (state) => state.location.pathname,
  })

  return (
    <Sidebar className="top-14 h-[calc(100svh-3.5rem)]" collapsible="offcanvas">
      <SidebarContent className="py-5">
        <SidebarGroup>
          <SidebarGroupLabel>Sections</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {sections.map((section) => (
                <SidebarEntry
                  key={section.to}
                  to={section.to}
                  label={section.label}
                  active={pathname === section.to}
                />
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
        <SidebarGroup>
          <SidebarGroupLabel>Components</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {components.map((component) => (
                <SidebarEntry
                  key={component.slug}
                  to="/components/$slug"
                  params={{ slug: component.slug }}
                  label={component.name}
                  active={pathname === `/components/${component.slug}`}
                />
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarRail />
    </Sidebar>
  )
}

function SidebarEntry({
  to,
  params,
  label,
  active,
}: {
  to: string
  params?: { slug: string }
  label: string
  active: boolean
}) {
  const { setOpenMobile } = useSidebar()

  return (
    <SidebarMenuItem>
      <SidebarMenuButton
        isActive={active}
        render={
          <Link to={to} params={params} onClick={() => setOpenMobile(false)} />
        }
      >
        {label}
      </SidebarMenuButton>
    </SidebarMenuItem>
  )
}

function CatalogSearch({
  open,
  onOpenChange,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
}) {
  const navigate = useNavigate()

  React.useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault()
        onOpenChange(!open)
      }
    }
    window.addEventListener("keydown", onKeyDown)
    return () => window.removeEventListener("keydown", onKeyDown)
  }, [onOpenChange, open])

  const select = (slug: string) => {
    onOpenChange(false)
    void navigate({ to: "/components/$slug", params: { slug } })
  }

  return (
    <CommandDialog
      open={open}
      onOpenChange={onOpenChange}
      title="Search gpuicn"
    >
      <Command>
        <CommandInput placeholder="Search documentation..." autoFocus />
        <CommandList>
          <CommandEmpty>No results found.</CommandEmpty>
          <CommandGroup heading="Documentation">
            {sections.map((section) => (
              <CommandItem
                key={section.to}
                value={section.label}
                onSelect={() => {
                  onOpenChange(false)
                  void navigate({ to: section.to })
                }}
              >
                {section.label}
              </CommandItem>
            ))}
          </CommandGroup>
          <CommandGroup heading="Components">
            {components.map((component) => (
              <CommandItem
                key={component.slug}
                value={`${component.name} ${component.description}`}
                onSelect={() => select(component.slug)}
              >
                <span className="flex min-w-0 flex-col gap-0.5">
                  <span>{component.name}</span>
                  <span className="truncate text-xs text-muted-foreground">
                    {component.description}
                  </span>
                </span>
              </CommandItem>
            ))}
          </CommandGroup>
        </CommandList>
      </Command>
    </CommandDialog>
  )
}
