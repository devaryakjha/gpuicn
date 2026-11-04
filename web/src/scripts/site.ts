// Everything the site needs on the client: the theme switch, copy buttons,
// the search palette, tab groups and the GPUI preview frames. Each piece is
// independent and reads its state from the DOM.

const root = document.documentElement

// Theme ----------------------------------------------------------------------

for (const button of document.querySelectorAll<HTMLButtonElement>("[data-theme-toggle]")) {
  button.addEventListener("click", () => {
    const next = root.dataset.theme === "dark" ? "light" : "dark"
    root.dataset.theme = next
    try {
      localStorage.setItem("gpuicn-theme", next)
    } catch {}
  })
}

// Docs rail on small screens ---------------------------------------------------

const railToggle = document.querySelector<HTMLButtonElement>("[data-rail-toggle]")
railToggle?.addEventListener("click", () => {
  railToggle.setAttribute("aria-expanded", String(railToggle.getAttribute("aria-expanded") !== "true"))
})

// Copy buttons ---------------------------------------------------------------

document.addEventListener("click", async (event) => {
  const button = (event.target as Element).closest<HTMLButtonElement>("[data-copy]")
  if (!button) return
  const block = button.closest(".code")
  const code = block?.querySelector("pre")?.textContent ?? ""
  const status = block?.querySelector<HTMLElement>("[data-copy-status]")
  try {
    await navigator.clipboard.writeText(code)
    button.dataset.copied = ""
    button.setAttribute("aria-label", "Copied")
    if (status) status.textContent = ""
    window.setTimeout(() => {
      delete button.dataset.copied
      button.setAttribute("aria-label", "Copy code")
    }, 1400)
  } catch {
    if (status) status.textContent = "Copy failed. Select the code and copy it manually."
  }
})

// Tabs -----------------------------------------------------------------------

for (const tabs of document.querySelectorAll<HTMLElement>("[data-tabs]")) {
  const buttons = [...tabs.querySelectorAll<HTMLButtonElement>("[role=tab]")]
  const panels = [...tabs.querySelectorAll<HTMLElement>("[role=tabpanel]")]
  const select = (index: number) => {
    buttons.forEach((button, i) => {
      button.setAttribute("aria-selected", String(i === index))
      button.tabIndex = i === index ? 0 : -1
    })
    panels.forEach((panel, i) => {
      panel.hidden = i !== index
    })
    tabs.dispatchEvent(new CustomEvent("tabchange"))
  }
  buttons.forEach((button, index) => {
    button.addEventListener("click", () => select(index))
    button.addEventListener("keydown", (event) => {
      const delta = event.key === "ArrowRight" ? 1 : event.key === "ArrowLeft" ? -1 : 0
      if (!delta) return
      event.preventDefault()
      const next = (index + delta + buttons.length) % buttons.length
      select(next)
      buttons[next]?.focus()
    })
  })
}

// Palette --------------------------------------------------------------------

const palette = document.querySelector<HTMLDialogElement>("[data-palette]")
if (palette) {
  const input = palette.querySelector<HTMLInputElement>("[data-palette-input]")!
  const list = palette.querySelector<HTMLElement>("[data-palette-list]")!
  const template = palette.querySelector<HTMLTemplateElement>("[data-palette-items]")!
  let items: HTMLAnchorElement[] = []
  let groups: HTMLElement[] = []
  let empty: HTMLElement | null = null
  let active = -1

  // The list lives in a template until the first open.
  const mount = () => {
    if (empty) return
    list.append(template.content.cloneNode(true))
    items = [...list.querySelectorAll<HTMLAnchorElement>(".palette__item")]
    groups = [...list.querySelectorAll<HTMLElement>(".palette__group")]
    empty = list.querySelector<HTMLElement>("[data-palette-empty]")!
  }

  const visible = () => items.filter((item) => !item.hidden)
  const highlight = (index: number) => {
    const shown = visible()
    active = shown.length ? Math.max(0, Math.min(index, shown.length - 1)) : -1
    items.forEach((item) => delete item.dataset.active)
    const item = shown[active]
    if (item) {
      item.dataset.active = ""
      item.scrollIntoView({ block: "nearest" })
    }
  }
  const filter = () => {
    const terms = input.value.toLowerCase().trim().split(/\s+/).filter(Boolean)
    for (const item of items) {
      const text = item.dataset.terms ?? ""
      item.hidden = !terms.every((term) => text.includes(term))
    }
    for (const group of groups) {
      let sibling = group.nextElementSibling
      let any = false
      while (sibling && sibling.classList.contains("palette__item")) {
        if (!(sibling as HTMLElement).hidden) any = true
        sibling = sibling.nextElementSibling
      }
      group.hidden = !any
    }
    empty!.hidden = visible().length > 0
    highlight(0)
  }
  const open = () => {
    if (palette.open) return
    mount()
    palette.showModal()
    input.value = ""
    filter()
    input.focus()
  }

  input.addEventListener("input", filter)
  palette.addEventListener("keydown", (event) => {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault()
      highlight(active + (event.key === "ArrowDown" ? 1 : -1))
    } else if (event.key === "Enter" && document.activeElement === input) {
      const item = visible()[active]
      if (item) {
        event.preventDefault()
        item.click()
      }
    }
  })
  palette.addEventListener("click", (event) => {
    if (event.target === palette) palette.close()
  })
  for (const trigger of document.querySelectorAll("[data-open-palette]")) {
    trigger.addEventListener("click", open)
  }
  window.addEventListener("keydown", (event) => {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
      event.preventDefault()
      palette.open ? palette.close() : open()
    }
  })
}

// GPUI previews --------------------------------------------------------------
//
// Each `.gpu` host renders a WASM build of the component in an iframe. The
// frame reports `preview-ready` or `preview-error` by postMessage; the host
// forwards theme and icon changes with `preview-update`.

type Status = "loading" | "ready" | "failed" | "unsupported"

const theme = () => (root.dataset.theme === "dark" ? "dark" : "light")

const afterLoad = (run: () => void) => {
  const idle = () =>
    "requestIdleCallback" in window ? requestIdleCallback(run, { timeout: 1500 }) : setTimeout(run, 200)
  if (document.readyState === "complete") idle()
  else window.addEventListener("load", idle, { once: true })
}
const hosts = new Set<Preview>()

class Preview {
  frame: HTMLIFrameElement | null = null
  timeout = 0
  constructor(public host: HTMLElement) {
    hosts.add(this)
    host.querySelector("[data-retry]")?.addEventListener("click", () => this.mount())
    // The preview draws with WebGPU, or WebGL 2 where WebGPU is missing.
    if (!("gpu" in navigator) && typeof WebGL2RenderingContext === "undefined") {
      this.status("unsupported")
      return
    }
    // Mount only when visible, so a page with several previews loads one
    // WASM instance at a time, and only once the page itself has loaded and
    // gone idle, so the preview never delays the text or the first click.
    const observer = new IntersectionObserver((entries) => {
      if (entries.some((entry) => entry.isIntersecting) && !this.frame) {
        observer.disconnect()
        afterLoad(() => this.frame || this.mount())
      }
    })
    observer.observe(host)
  }
  status(value: Status) {
    this.host.dataset.status = value
  }
  src() {
    const query = new URLSearchParams({
      demo: this.host.dataset.demo ?? "button",
      theme: theme(),
      width: String(Math.max(1, Math.round(this.host.getBoundingClientRect().width))),
      height: this.host.dataset.height ?? "288",
    })
    const icon = this.host.dataset.icon
    const example = this.host.dataset.example
    if (icon) query.set("icon", icon)
    if (example) query.set("example", example)
    // The preview falls back to WebGL by itself. `?backend=webgl` on the page
    // skips WebGPU, which the screenshot scripts use.
    const backend = new URLSearchParams(location.search).get("backend")
    if (backend === "webgl" || !("gpu" in navigator)) query.set("backend", "webgl")
    // `/demo/`, not `/demo/index.html`: the host forces trailing slashes and
    // redirects the latter, and a cross-origin-isolated page blocks a framed
    // document that arrives through a redirect.
    return `/demo/?${query}`
  }
  mount() {
    this.frame?.remove()
    window.clearTimeout(this.timeout)
    this.status("loading")
    const frame = document.createElement("iframe")
    frame.title = `Interactive ${this.host.dataset.name ?? "component"} GPUI preview`
    frame.src = this.src()
    frame.addEventListener("error", () => this.status("failed"))
    this.frame = frame
    this.host.prepend(frame)
    this.timeout = window.setTimeout(() => this.status("failed"), 45_000)
  }
  update() {
    if (this.host.dataset.status !== "ready") return
    this.frame?.contentWindow?.postMessage(
      { gpuicn: "preview-update", theme: theme(), icon: this.host.dataset.icon ?? "" },
      window.location.origin
    )
  }
}

for (const host of document.querySelectorAll<HTMLElement>(".gpu[data-demo]")) {
  new Preview(host)
}

window.addEventListener("message", (event) => {
  if (event.origin !== window.location.origin) return
  const kind = event.data?.gpuicn
  if (kind !== "preview-ready" && kind !== "preview-error") return
  for (const preview of hosts) {
    if (preview.frame?.contentWindow === event.source) {
      window.clearTimeout(preview.timeout)
      preview.status(kind === "preview-ready" ? "ready" : "failed")
      preview.update()
    }
  }
})

new MutationObserver(() => {
  for (const preview of hosts) preview.update()
}).observe(root, { attributes: true, attributeFilter: ["data-theme"] })

// Pages that change a preview's icon or example set the attribute and
// dispatch this event on the host.
document.addEventListener("gpuicn:preview", (event) => {
  const host = event.target as HTMLElement
  for (const preview of hosts) {
    if (preview.host !== host) continue
    if ((event as CustomEvent).detail === "remount") preview.mount()
    else preview.update()
  }
})
