import { createFileRoute } from "@tanstack/react-router"

import { DocsLayout, InstallationTabs, PageHeader } from "@/components/docs"

export const Route = createFileRoute("/installation")({
  component: InstallationPage,
})

function InstallationPage() {
  return (
    <DocsLayout>
      <PageHeader
        eyebrow="Sections"
        title="Installation"
        description="Install a component from the registry or copy its Rust source into your application."
      />
      <InstallationTabs slug="button" />
    </DocsLayout>
  )
}
