import { createFileRoute } from "@tanstack/react-router"

import { ComponentGrid, DocsLayout, PageHeader } from "@/components/docs"

export const Route = createFileRoute("/")({ component: ComponentsIndex })

function ComponentsIndex() {
  return (
    <DocsLayout>
      <PageHeader
        eyebrow="Base GPUI"
        title="Components"
        description="Editable shadcn visual ports backed by native GPUI behavior. Add the source you need, then own it in your application."
      />
      <div className="mt-10">
        <ComponentGrid />
      </div>
    </DocsLayout>
  )
}
