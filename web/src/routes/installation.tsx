import { createFileRoute } from "@tanstack/react-router"

import {
  BetaNotice,
  CodeBlock,
  DocsLayout,
  PageHeader,
} from "@/components/docs"
import setup from "@/generated/setup.json"

export const Route = createFileRoute("/installation")({
  component: InstallationPage,
})

function InstallationPage() {
  return (
    <DocsLayout>
      <PageHeader
        eyebrow="Getting started"
        title="Installation"
        description="Add editable Rust components to a GPUI app. Start with one button, then install only what you need."
      />
      <BetaNotice />
      <div className="mt-8 space-y-10">
        <section>
          <h2 className="text-xl font-semibold tracking-tight">
            1. Create a Rust app
          </h2>
          <p className="mt-3 leading-7 text-muted-foreground">
            Use Rust 1.97.1 or newer and the native build tools for your platform.
            Keep these dependency revisions together: GPUI types must come from
            the same revision across your app, Base GPUI, and the icon library.
          </p>
          <CodeBlock
            language="bash"
            className="mt-4"
            value="cargo new my-app\ncd my-app"
          />
          <p className="mt-4 text-sm text-muted-foreground">
            The GPUI compatibility patch belongs in your app’s root Cargo.toml.
            Add to Cargo.toml:
          </p>
          <CodeBlock
            className="mt-3"
            value={`[dependencies]\n${setup.dependencies}`}
            language="toml"
          />
        </section>

        <section>
          <h2 className="text-xl font-semibold tracking-tight">
            2. Configure the source installer
          </h2>
          <p className="mt-3 leading-7 text-muted-foreground">
            The native CLI is a local preview, not yet published to crates.io.
            Install it from a gpuicn checkout with Rust alone. No Node.js is required.
          </p>
          <CodeBlock language="bash" className="mt-4" value="cargo install --path /path/to/gpuicn/crates/cli --locked" />
          <p className="mt-4 text-sm leading-6 text-muted-foreground">
            From your app root, create gpuicn.toml. Replace the path with your
            checkout; you can also use an HTTP(S) registry snapshot.
          </p>
          <CodeBlock language="bash" className="mt-3" value="gpuicn --registry /path/to/gpuicn/site/pages/r init" />
          <CodeBlock language="toml" className="mt-3" value={'version = 1\nregistry = "/path/to/gpuicn/site/pages/r"\noutput = "src/ui"'} />
        </section>

        <section>
          <h2 className="text-xl font-semibold tracking-tight">
            3. Install your first component
          </h2>
          <CodeBlock
            className="mt-4"
            value="gpuicn add button"
            language="bash"
          />
          <p className="mt-3 leading-7 text-muted-foreground">
            This installs Button and its shared theme, and maintains{" "}
            <code className="text-sm">src/ui/mod.rs</code>:
          </p>
          <CodeBlock className="mt-4" value="pub mod button;\npub mod theme;" />
          <p className="mt-3 leading-7 text-muted-foreground">
            Add mod ui; to your crate root. Existing edits are kept by default. Your installed files
            belong to your app. Review local edits before using the installer’s
            <code className="text-sm"> --overwrite</code> option.
          </p>
        </section>

        <section>
          <h2 className="text-xl font-semibold tracking-tight">
            4. Add the fonts
          </h2>
          <p className="mt-3 leading-7 text-muted-foreground">
            The default style uses Geist. Download the pinned font files and
            their license, or load your own fonts and change the theme’s font
            families.
          </p>
          <CodeBlock
            className="mt-4"
            language="bash"
            value={
              'mkdir -p assets/fonts\nfor file in Geist-Regular.ttf Geist-Medium.ttf GeistMono-Regular.ttf OFL.txt; do\n  curl -fL "https://ui.imajha.com/fonts/$file" -o "assets/fonts/$file"\ndone'
            }
          />
        </section>

        <section>
          <h2 className="text-xl font-semibold tracking-tight">
            5. Open a window
          </h2>
          <p className="mt-3 leading-7 text-muted-foreground">
            Replace <code className="text-sm">src/main.rs</code> with this
            starter. It calls ui::theme::init(cx) to enable component actions and
            keyboard navigation, loads the fonts, and renders a working counter button.
          </p>
          <CodeBlock className="mt-4" value={setup.starter} />
          <a
            href="/examples/hello.rs"
            download
            className="mt-3 inline-block text-sm underline underline-offset-4"
          >
            Download starter source
          </a>
          <CodeBlock language="bash" className="mt-4" value="cargo run" />
        </section>
      </div>
    </DocsLayout>
  )
}
