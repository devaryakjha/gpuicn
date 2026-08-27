export type CatalogComponent = {
  slug: string
  name: string
  description: string
}

export const components: CatalogComponent[] = [
  {
    slug: "accordion",
    name: "Accordion",
    description:
      "A vertically stacked set of interactive headings that reveal content.",
  },
  {
    slug: "alert-dialog",
    name: "Alert Dialog",
    description: "A modal dialog that expects a response from the user.",
  },
  {
    slug: "autocomplete",
    name: "Autocomplete",
    description: "A text input that suggests matching values.",
  },
  {
    slug: "avatar",
    name: "Avatar",
    description: "An image element with a fallback for representing a user.",
  },
  {
    slug: "button",
    name: "Button",
    description: "Displays a button or a component that looks like a button.",
  },
  {
    slug: "checkbox",
    name: "Checkbox",
    description: "A control that toggles between checked and unchecked.",
  },
  {
    slug: "checkbox-group",
    name: "Checkbox Group",
    description: "Groups related checkboxes under shared state.",
  },
  {
    slug: "collapsible",
    name: "Collapsible",
    description: "An interactive panel that expands and collapses.",
  },
  {
    slug: "combobox",
    name: "Combobox",
    description: "An input paired with a searchable list of choices.",
  },
  {
    slug: "context-menu",
    name: "Context Menu",
    description: "A menu opened by a secondary pointer action.",
  },
  {
    slug: "dialog",
    name: "Dialog",
    description: "A window overlaid on the primary content.",
  },
  {
    slug: "drawer",
    name: "Drawer",
    description: "A panel that slides from an edge of the window.",
  },
  {
    slug: "field",
    name: "Field",
    description: "Composes a control with its label, description, and errors.",
  },
  {
    slug: "fieldset",
    name: "Fieldset",
    description: "Groups related form controls with a legend.",
  },
  {
    slug: "form",
    name: "Form",
    description: "Coordinates validation and submission for fields.",
  },
  {
    slug: "input",
    name: "Input",
    description: "A styled text input backed by native GPUI editing.",
  },
  {
    slug: "menu",
    name: "Dropdown Menu",
    description: "Displays a menu of actions from a trigger.",
  },
  {
    slug: "menubar",
    name: "Menubar",
    description: "A persistent horizontal set of menus.",
  },
  {
    slug: "meter",
    name: "Meter",
    description: "Shows a scalar value within a known range.",
  },
  {
    slug: "navigation-menu",
    name: "Navigation Menu",
    description: "A collection of links for navigating a site or app.",
  },
  {
    slug: "number-field",
    name: "Number Field",
    description: "A numeric input with step controls.",
  },
  {
    slug: "otp-field",
    name: "Input OTP",
    description: "A segmented input for one-time passcodes.",
  },
  {
    slug: "popover",
    name: "Popover",
    description: "Displays rich content in a portal from a trigger.",
  },
  {
    slug: "preview-card",
    name: "Hover Card",
    description: "Previews content behind a link or trigger.",
  },
  {
    slug: "progress",
    name: "Progress",
    description: "Displays completion progress for a task.",
  },
  {
    slug: "radio-group",
    name: "Radio Group",
    description: "A set of checkable options where one value is selected.",
  },
  {
    slug: "scroll-area",
    name: "Scroll Area",
    description: "Adds styled scrolling to overflowing content.",
  },
  {
    slug: "select",
    name: "Select",
    description: "Displays a list of options for the user to pick from.",
  },
  {
    slug: "separator",
    name: "Separator",
    description: "Visually separates content.",
  },
  {
    slug: "slider",
    name: "Slider",
    description: "Selects a value or range along a track.",
  },
  {
    slug: "switch",
    name: "Switch",
    description: "Toggles a setting on or off.",
  },
  {
    slug: "tabs",
    name: "Tabs",
    description: "Shows one panel from a set of layered sections.",
  },
  {
    slug: "toast",
    name: "Toast",
    description: "A brief notification shown above the application.",
  },
  {
    slug: "toggle",
    name: "Toggle",
    description: "A two-state button that can be on or off.",
  },
  {
    slug: "toggle-group",
    name: "Toggle Group",
    description: "Groups two-state buttons with shared selection.",
  },
  {
    slug: "toolbar",
    name: "Toolbar",
    description: "Groups controls for a related set of actions.",
  },
  {
    slug: "tooltip",
    name: "Tooltip",
    description: "Shows short information on hover or keyboard focus.",
  },
]

export const sections = [
  { to: "/", label: "Introduction" },
  { to: "/installation", label: "Installation" },
  { to: "/theming", label: "Theming" },
  { to: "/registry", label: "Registry" },
] as const

export function findComponent(slug: string) {
  return components.find((component) => component.slug === slug)
}

export function previewHeight(slug: string) {
  if (
    [
      "alert-dialog",
      "context-menu",
      "dialog",
      "drawer",
      "menu",
      "menubar",
      "navigation-menu",
    ].includes(slug)
  )
    return 432
  if (
    ["autocomplete", "combobox", "popover", "preview-card", "select"].includes(
      slug
    )
  )
    return 360
  return 288
}

export function rustModule(slug: string) {
  return slug.replaceAll("-", "_")
}
