import { createFileRoute, notFound } from "@tanstack/react-router"

import { ComponentPage } from "@/components/docs"
import { findComponent } from "@/lib/catalog"

export const Route = createFileRoute("/components/$slug")({
  loader: ({ params }) => {
    const component = findComponent(params.slug)
    if (!component) throw notFound()
    return component
  },
  component: CatalogComponentPage,
})

function CatalogComponentPage() {
  const component = Route.useLoaderData()
  return <ComponentPage component={component} />
}
