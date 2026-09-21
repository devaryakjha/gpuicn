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

const release = "v0.5.0-beta.4"
const releaseSource = `https://raw.githubusercontent.com/devaryakjha/gpuicn/${release}`
const registry = `${releaseSource}/site/pages/r`

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
            Use Rust 1.97.1 or newer and the native build tools for your
            platform. This release supports GPUI Kit 0.6.4 on its pinned
            gpui-pre 0.3.5 runtime. Import GPUI types through gpui_kit so your
            app and the components use the same types. Apps on another GPUI
            revision must migrate to this runtime before installing the
            components.
          </p>
          <CodeBlock
            language="bash"
            className="mt-4"
            value={"cargo new my-app\ncd my-app"}
          />
          <p className="mt-4 text-sm text-muted-foreground">
            Add these dependencies to Cargo.toml:
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
            The native CLI is distributed as source and is not yet published to
            crates.io. Install the current published beta directly from its Git
            tag. No Node.js or gpuicn checkout is required.
          </p>
          <CodeBlock
            language="bash"
            className="mt-4"
            value={`cargo install gpuicn-cli --git https://github.com/devaryakjha/gpuicn --tag ${release} --locked`}
          />
          <p className="mt-4 text-sm leading-6 text-muted-foreground">
            From your app root, create gpuicn.toml against the same release
            snapshot. This pins the CLI and component source to one release;
            keep the dependency versions above together with it.
          </p>
          <CodeBlock
            language="bash"
            className="mt-3"
            value={`gpuicn --registry ${registry} init`}
          />
          <CodeBlock
            language="toml"
            className="mt-3"
            value={`version = 1\nregistry = "${registry}"\noutput = "src/ui"`}
          />
          <p className="mt-4 text-sm leading-6 text-muted-foreground">
            {release} is the current published beta. Fixes made after that tag
            are not included in this snapshot.
          </p>
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
          <CodeBlock
            className="mt-4"
            value={"pub mod button;\npub mod theme;"}
          />
          <p className="mt-3 leading-7 text-muted-foreground">
            Add mod ui; to your crate root. Existing edits are kept by default.
            Your installed files belong to your app. Review local edits before
            using the installer’s
            <code className="text-sm"> --overwrite</code> option.
          </p>
          <p className="mt-3 leading-7 text-muted-foreground">
            To update, install the CLI with{" "}
            <code className="text-sm">--force</code>
            and change the registry URL to the same newer release. Commit your
            current app-specific edits first. A dry run reports which files
            would change; after opting into{" "}
            <code className="text-sm">--overwrite</code>, review the resulting
            source diff and reapply the edits you still need.
          </p>
          <CodeBlock
            className="mt-4"
            language="bash"
            value={
              "gpuicn add button --dry-run --overwrite\ngpuicn add button --overwrite\ngit diff -- src/ui"
            }
          />
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
            value={`mkdir -p assets/fonts\nfor file in Geist-Regular.ttf Geist-Medium.ttf GeistMono-Regular.ttf; do\n  curl -fL "${releaseSource}/site/assets/fonts/$file" -o "assets/fonts/$file"\ndone\ncurl -fL "${releaseSource}/LICENSES/Geist-OFL-1.1" -o assets/fonts/OFL.txt`}
          />
        </section>

        <section>
          <h2 className="text-xl font-semibold tracking-tight">
            5. Open a window
          </h2>
          <p className="mt-3 leading-7 text-muted-foreground">
            Replace <code className="text-sm">src/main.rs</code> with this
            starter. It calls ui::theme::init(cx) to enable component actions
            and keyboard navigation, loads the fonts, and renders a working
            counter button.
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
