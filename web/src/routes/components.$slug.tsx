import { createFileRoute, notFound } from "@tanstack/react-router"

import { ComponentPage } from "@/components/docs"
import { findComponent } from "@/lib/catalog"
import { loadComponentDocs } from "@/lib/component-docs"

export const Route = createFileRoute("/components/$slug")({
  loader: async ({ params }) => {
    const component = findComponent(params.slug)
    if (!component) throw notFound()
    return { component, docs: await loadComponentDocs(component.slug) }
  },
  component: CatalogComponentPage,
})

function CatalogComponentPage() {
  const { component, docs } = Route.useLoaderData()
  return (
    <ComponentPage key={component.slug} component={component} docs={docs} />
  )
}
