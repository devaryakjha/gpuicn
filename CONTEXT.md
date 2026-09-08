# gpuicn

Open-code UI components for GPUI, with shadcn-inspired styling and native desktop interactions. Applications own and can edit the installed source.

## Language

**Visual port**:
A GPUI component that preserves a shadcn component's visual identity, themes, variants, and interaction states through an idiomatic GPUI API.
_Avoid_: API port, React port

**Component catalog**:
Styled Base GPUI controls and reusable larger desktop components. A larger component may compose existing controls or add a missing reusable interaction; it must keep application data, routing and domain operations caller-owned.
_Avoid_: Blocks, templates, examples

**Component preview**:
The website surface that renders a catalog component and exposes its real interactions using the actual GPUI component code.
_Avoid_: Example app, JavaScript imitation

**Style**:
A coherent visual treatment that can be applied across the component catalog. The initial style is shadcn's pinned default visual baseline.
_Avoid_: Theme

**Theme**:
The configurable design tokens, including color, typography, radius, and spacing, used by a style and its components.
_Avoid_: Style

## Desktop scope

A sidebar is a navigation composition; a splitter is an interactive boundary that resizes adjacent panes. They can be used together or separately. Lists, trees, diff views and graph views may belong in gpuicn; Git execution, parsing, patch generation, graph layout algorithms, provider APIs and persistence do not.

The local expansion plan is in `docs/plans/desktop-components.md`. New components require native interaction and performance evidence, followed by user testing before publication.
