(function () {
  const components = [
    ["accordion", "Accordion", "A vertically stacked set of interactive headings that reveal content."],
    ["alert-dialog", "Alert Dialog", "A modal dialog that expects a response from the user."],
    ["autocomplete", "Autocomplete", "A text input that suggests matching values."],
    ["avatar", "Avatar", "An image element with a fallback for representing a user."],
    ["button", "Button", "Displays a button or a component that looks like a button."],
    ["checkbox", "Checkbox", "A control that toggles between checked and unchecked."],
    ["checkbox-group", "Checkbox Group", "Groups related checkboxes under shared state."],
    ["collapsible", "Collapsible", "An interactive panel that expands and collapses."],
    ["combobox", "Combobox", "An input paired with a searchable list of choices."],
    ["context-menu", "Context Menu", "A menu opened by a secondary pointer action."],
    ["dialog", "Dialog", "A window overlaid on the primary content."],
    ["drawer", "Drawer", "A panel that slides from an edge of the window."],
    ["field", "Field", "Composes a control with its label, description, and errors."],
    ["fieldset", "Fieldset", "Groups related form controls with a legend."],
    ["form", "Form", "Coordinates validation and submission for fields."],
    ["input", "Input", "A styled text input backed by native GPUI editing."],
    ["menu", "Dropdown Menu", "Displays a menu of actions from a trigger."],
    ["menubar", "Menubar", "A persistent horizontal set of menus."],
    ["meter", "Meter", "Shows a scalar value within a known range."],
    ["navigation-menu", "Navigation Menu", "A collection of links for navigating a site or app."],
    ["number-field", "Number Field", "A numeric input with step controls."],
    ["otp-field", "Input OTP", "A segmented input for one-time passcodes."],
    ["popover", "Popover", "Displays rich content in a portal from a trigger."],
    ["preview-card", "Hover Card", "Previews content behind a link or trigger."],
    ["progress", "Progress", "Displays completion progress for a task."],
    ["radio-group", "Radio Group", "A set of checkable options where one value is selected."],
    ["scroll-area", "Scroll Area", "Adds styled scrolling to overflowing content."],
    ["select", "Select", "Displays a list of options for the user to pick from."],
    ["separator", "Separator", "Visually separates content."],
    ["slider", "Slider", "Selects a value or range along a track."],
    ["switch", "Switch", "Toggles a setting on or off."],
    ["tabs", "Tabs", "Shows one panel from a set of layered sections."],
    ["toast", "Toast", "A brief notification shown above the application."],
    ["toggle", "Toggle", "A two-state button that can be on or off."],
    ["toggle-group", "Toggle Group", "Groups two-state buttons with shared selection."],
    ["toolbar", "Toolbar", "Groups controls for a related set of actions."],
    ["tooltip", "Tooltip", "Shows short information on hover or keyboard focus."]
  ];

  const app = document.getElementById("app");
  const page = document.body.dataset.page;
  const isComponent = page === "component";
  const prefix = isComponent ? "../../" : "";
  const slug = document.body.dataset.component || "";
  const current = components.find(([key]) => key === slug);
  const savedTheme = localStorage.getItem("gpuicn-theme");
  const dark = savedTheme ? savedTheme === "dark" : matchMedia("(prefers-color-scheme: dark)").matches;
  document.documentElement.classList.toggle("dark", dark);

  function header() {
    return `<header class="site-header"><div class="header-inner">
      <button class="icon-button mobile-menu" type="button" data-menu aria-label="Toggle menu">☰</button>
      <a class="brand" href="${prefix}">gpuicn</a>
      <nav class="top-nav" aria-label="Main navigation">
        <a class="top-link" href="${prefix}">Docs</a>
        <a class="top-link" href="${prefix}" aria-current="page">Components</a>
        <a class="top-link" href="${prefix}r/registry.json">Registry</a>
      </nav>
      <div class="header-actions">
        <button class="search" type="button" data-search>Search documentation… <kbd>⌘K</kbd></button>
        <span class="divider"></span>
        <a class="icon-button" href="https://github.com/devaryakjha/gpuicn" aria-label="GitHub"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3.28-.36 6.72-1.61 6.72-7.25A5.65 5.65 0 0 0 19.22 3.3 5.4 5.4 0 0 0 19.08 1S17.9.65 15 2.48a13.38 13.38 0 0 0-7 0C5.1.65 3.92 1 3.92 1a5.4 5.4 0 0 0-.14 2.3A5.65 5.65 0 0 0 2.28 7.25c0 5.63 3.44 6.88 6.72 7.25A4.8 4.8 0 0 0 8 18v4"/><path d="M8 19c-3 .9-3-1.5-4-2"/></svg></a>
        <span class="divider"></span>
        <button class="icon-button" type="button" data-theme aria-label="Toggle theme"><svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="9"/><path d="M12 3v18M12 9l4.65-4.65M12 14.3l7.37-7.37M12 19.6l8.85-8.85"/></svg></button>
        <a class="new-button" href="${prefix}r/registry.json">Get started</a>
      </div>
    </div></header>`;
  }

  function sidebar() {
    const links = components.map(([key, title]) => `<a class="side-link" href="${prefix}components/${key}/" ${key === slug ? 'aria-current="page"' : ""}>${title}</a>`).join("");
    return `<aside class="sidebar" data-sidebar>
      <div class="side-group"><div class="side-label">Sections</div>
        <a class="side-link" href="${prefix}">Introduction</a>
        <a class="side-link" href="${prefix}" ${!isComponent ? 'aria-current="page"' : ""}>Components</a>
        <a class="side-link" href="${prefix}r/registry.json">Installation</a>
        <a class="side-link" href="${prefix}r/theme.json">Theming</a>
        <a class="side-link" href="${prefix}r/registry.json">Registry</a>
      </div>
      <div class="side-group"><div class="side-label">Components</div>${links}</div>
    </aside>`;
  }

  function indexPage() {
    const cards = components.map(([key, title, description]) => `<a class="component-tile" href="components/${key}/"><div>${title}</div><div class="paragraph">${description}</div></a>`).join("");
    return `${header()}<div class="docs-layout">${sidebar()}<main class="component-index"><h1>Components</h1><p class="description">Editable shadcn visual ports backed by native GPUI behavior.</p><div class="component-grid">${cards}</div></main></div>`;
  }

  function commandMenu() {
    const links = components.map(([key, title, description]) => `<a class="command-result" href="${prefix}components/${key}/" data-result><strong>${title}</strong><span>${description}</span></a>`).join("");
    return `<dialog class="command-menu" data-command><div class="command-panel">
      <div class="command-search"><span aria-hidden="true">⌕</span><input type="search" placeholder="Search components…" aria-label="Search components" data-command-input></div>
      <div class="command-results">${links}<p class="command-empty" hidden data-command-empty>No components found.</p></div>
    </div></dialog>`;
  }

  function componentPage() {
    if (!current) return `${header()}<main class="component-index"><h1>Component not found</h1></main>`;
    const index = components.indexOf(current);
    const previous = components[index - 1];
    const next = components[index + 1];
    const [key, title, description] = current;
    const rustModule = key.replaceAll("-", "_");
    const source = `use gpuicn::${rustModule};\n\ngpuicn::init(cx);\n// The installed ${title} source is yours to edit.`;
    return `${header()}<div class="docs-layout">${sidebar()}<main class="doc"><article class="article">
      <header class="article-header"><div><h1>${title}</h1><p class="description">${description}</p></div><nav class="pager" aria-label="Page navigation">
        ${previous ? `<a class="icon-button" href="../${previous[0]}/" aria-label="Previous">←</a>` : ""}
        ${next ? `<a class="icon-button" href="../${next[0]}/" aria-label="Next">→</a>` : ""}
      </nav></header>
      <section class="preview-card"><div class="preview"><iframe data-demo="${key}" title="Interactive ${title} GPUI preview" hidden></iframe><div data-status="loading">Preview waits until visible.</div></div>
        <div class="code-peek"><pre>${escapeHtml(source)}</pre><button class="view-code" type="button">View Code</button></div>
      </section>
      <h2 class="section-title" id="installation">Installation</h2>
      <div class="install-tabs" role="tablist"><button class="tab" role="tab" aria-selected="true">Command</button><button class="tab" role="tab" aria-selected="false">Manual</button></div>
      <div class="command">npx shadcn@latest add https://devaryakjha.github.io/gpuicn/r/${key}.json<button class="icon-button copy" type="button" data-copy aria-label="Copy command">⧉</button></div>
      <h2 class="section-title" id="usage">Usage</h2><p class="paragraph">Install the source, register Base GPUI during app startup, and keep each component ID stable across renders.</p>
      <div class="command">${escapeHtml(source)}</div>
      <h2 class="section-title" id="api-reference">API Reference</h2><p class="paragraph">The visual port keeps Base GPUI behavior and exposes an idiomatic Rust API. The source remains editable after installation.</p>
    </article><aside class="toc"><div class="toc-title">On This Page</div><a href="#installation">Installation</a><a href="#usage">Usage</a><a href="#api-reference">API Reference</a></aside></main></div>`;
  }

  function escapeHtml(value) {
    return value.replace(/[&<>]/g, (character) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;" })[character]);
  }

  app.innerHTML = `${isComponent ? componentPage() : indexPage()}${commandMenu()}`;

  document.querySelector("[data-theme]")?.addEventListener("click", () => {
    const enabled = document.documentElement.classList.toggle("dark");
    localStorage.setItem("gpuicn-theme", enabled ? "dark" : "light");
    location.reload();
  });
  document.querySelector("[data-menu]")?.addEventListener("click", () => document.querySelector("[data-sidebar]")?.classList.toggle("open"));
  document.querySelector(".view-code")?.addEventListener("click", (event) => event.currentTarget.parentElement.classList.add("expanded"));
  document.querySelector("[data-copy]")?.addEventListener("click", async (event) => {
    await navigator.clipboard.writeText(event.currentTarget.parentElement.firstChild.textContent);
    event.currentTarget.textContent = "✓";
  });

  const command = document.querySelector("[data-command]");
  const commandInput = document.querySelector("[data-command-input]");
  const openCommand = () => {
    command.showModal();
    commandInput.value = "";
    commandInput.dispatchEvent(new Event("input"));
    commandInput.focus();
  };
  document.querySelector("[data-search]")?.addEventListener("click", openCommand);
  command?.addEventListener("click", (event) => {
    if (event.target === command) command.close();
  });
  commandInput?.addEventListener("input", () => {
    const query = commandInput.value.trim().toLowerCase();
    let visible = 0;
    document.querySelectorAll("[data-result]").forEach((result) => {
      const matches = result.textContent.toLowerCase().includes(query);
      result.hidden = !matches;
      visible += Number(matches);
    });
    document.querySelector("[data-command-empty]").hidden = visible !== 0;
  });
  addEventListener("keydown", (event) => {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
      event.preventDefault();
      command.open ? command.close() : openCommand();
    }
  });

  const frame = document.querySelector("iframe[data-demo]");
  const status = document.querySelector("[data-status]");
  if (!frame || !status) return;
  let timer;
  function setStatus(value, detail) {
    status.dataset.status = value;
    status.textContent = detail || value;
    frame.hidden = value !== "ready";
  }
  function loadPreview() {
    if (!("gpu" in navigator)) return setStatus("unsupported", "This browser does not expose WebGPU.");
    setStatus("loading", "Loading interactive GPUI preview…");
    frame.src = `${prefix}demo/index.html?demo=${frame.dataset.demo}&theme=${document.documentElement.classList.contains("dark") ? "dark" : "light"}`;
    timer = setTimeout(() => setStatus("failed", "The GPUI preview did not start."), 30000);
  }
  addEventListener("message", (event) => {
    if (event.source !== frame.contentWindow || !event.data?.gpuicn) return;
    clearTimeout(timer);
    if (event.data.gpuicn === "preview-ready") setStatus("ready");
    if (event.data.gpuicn === "preview-error") setStatus("failed", event.data.message);
  });
  addEventListener("pagehide", () => { clearTimeout(timer); frame.src = "about:blank"; });
  new IntersectionObserver((entries, observer) => {
    if (!entries.some((entry) => entry.isIntersecting)) return;
    observer.disconnect();
    loadPreview();
  }).observe(frame.parentElement);
})();
