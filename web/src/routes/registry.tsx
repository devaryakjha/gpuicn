import { createFileRoute, Link } from "@tanstack/react-router"

import { CodeBlock, DocsLayout, PageHeader } from "@/components/docs"
import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { components } from "@/lib/catalog"

export const Route = createFileRoute("/registry")({ component: RegistryPage })

function RegistryPage() {
  return (
    <DocsLayout>
      <PageHeader
        eyebrow="Sections"
        title="Registry"
        description="The public registry powers installation tools. This page remains the human-readable catalog."
      />
      <CodeBlock
        language="bash"
        className="mt-8"
        value="npx shadcn@4.19.0 add https://ui.imajha.com/r/button.json"
      />
      <section className="mt-10 grid grid-cols-1 gap-3 sm:grid-cols-2">
        {components.map((component) => (
          <Link
            key={component.slug}
            to="/components/$slug"
            params={{ slug: component.slug }}
            className="group outline-none"
          >
            <Card
              className="h-full transition-colors group-hover:bg-muted/50 group-focus-visible:ring-ring"
              size="sm"
            >
              <CardHeader>
                <CardTitle>{component.name}</CardTitle>
                <CardDescription>/r/{component.slug}.json</CardDescription>
              </CardHeader>
            </Card>
          </Link>
        ))}
      </section>
    </DocsLayout>
  )
}
