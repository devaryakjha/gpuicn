# gpuicn

An open-code component catalog that brings shadcn's visual system to GPUI applications while keeping the installed source owned and editable by each application.

## Language

**Visual port**:
A GPUI component that preserves a shadcn component's visual identity, themes, variants, and interaction states through an idiomatic GPUI API.
_Avoid_: API port, React port

**Component catalog**:
The pinned set of standard shadcn UI components that gpuicn intends to make available for GPUI.
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
