export type ComponentDocs = {
  preview: string
  usage: string
  usageCall: string
  source: string
  api: { signature: string; description: string }[]
  parity: string
  modules: string[]
}

const documents = import.meta.glob<ComponentDocs>(
  "../generated/components/*.json",
  { import: "default" }
)

export async function loadComponentDocs(slug: string) {
  return documents[`../generated/components/${slug}.json`]()
}
