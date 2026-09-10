import { createFileRoute } from "@tanstack/react-router"

import {
  BetaNotice,
  ComponentGrid,
  DocsLayout,
  PageHeader,
} from "@/components/docs"

export const Route = createFileRoute("/introduction")({
  component: ComponentsIndex,
})

function ComponentsIndex() {
  return (
    <DocsLayout>
      <PageHeader
        eyebrow="GPUI Kit"
        title="Introduction"
        description="Editable shadcn visual ports backed by native GPUI behavior. Add the source you need, then own it in your application."
      />
      <BetaNotice />
      <div className="mt-10">
        <ComponentGrid />
      </div>
    </DocsLayout>
  )
}
