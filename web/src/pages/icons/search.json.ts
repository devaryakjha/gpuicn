import type { APIRoute } from "astro"
import { icons } from "../../lib/docs"

// name -> searchable text; the icons page fetches it on the first keystroke.
export const GET: APIRoute = () =>
  new Response(
    JSON.stringify(
      Object.fromEntries(
        icons.map((icon) => [
          icon.name,
          [icon.name, ...icon.tags, ...icon.categories, ...icon.aliases].join(" ").toLowerCase(),
        ])
      )
    ),
    { headers: { "content-type": "application/json" } }
  )
