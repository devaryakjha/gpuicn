# Sidebar local review — 2026-09-08

Scope: Sidebar source, six working catalog examples, source distribution, and README refresh. Changes remain local and uncommitted.

## Reference and typography

Compared the pinned Nova Sidebar source and live shadcn workspace/mail blocks. The live mail block reports Geist, 14px/20px navigation, 12px/16px secondary text, and 14px/17.5px mail titles. Bundled TTF names identify Geist Regular and Geist Medium. Sidebar now sets its family, normal weight, size and line height explicitly; selected rows use medium. Example headings use 20px/28px medium and mail headings 18px/24px. All sizes scale with the theme. This is an adapted native layout, not a claim of identical rasterization.

## Automated checks

- Library and showcase tests pass, including mobile state independence, modal Tab traversal/Escape/focus return, and search updates without an App reborrow.
- Six Sidebar examples render at 360px and 960px in light, dark, and custom spacing/type/radius themes, with open/closed states.
- Clippy passes for library/showcase all targets with warnings denied.
- Native release and WASM release builds pass. This machine needs `gpui_platform/runtime_shaders`; the installed Xcode lacks the offline Metal compiler.
- A fresh temporary app installed Sidebar with the native CLI, including its transitive sources, and compiled with the locked dependencies and runtime shader feature.
- Registry generation, catalog source sync, web build/lint, and whitespace checks pass.

These render tests detect crashes, not visual correctness or screen-reader usability.

## Real interface checks

Browser: workspace collapse/restore, team switching, keyboard toggle, mobile open/Escape, documentation rendering, mail search and filtered selection, unread filtering, right-side floating collapse, and loading/loaded navigation. Reviewed light and dark typography. Search `ali` filters to Alice and selecting the row updates the reader without losing the query.

macOS: wide 960px workspace, narrow 360px layout, modal navigation, Tab/Shift-Tab wrap, Escape, and return focus to the trigger. Inspected native accessible names and collapsed navigation visibility. The narrow sheet fits the viewport and preserves the desktop state.

Only the active catalog example mounts a GPUI preview. Skeletons have no idle animation; collapse uses the shared reduced-motion-aware transition. No frame-time or memory benchmark is claimed.

## Remaining platform limits

- In the embedded browser, Shift-Tab can leave the iframe despite GPUI's internal modal focus traversal. Native traversal and its regression test pass. Do not treat the browser preview as proof of native accessibility, or claim complete browser focus containment.
- Full VoiceOver, Windows and Linux qualification remain open. Base button current/expanded states are described in names because its public wrapper lacks those state setters.
- Custom themes passed render checks; complete visual acceptance of arbitrary fonts and density settings remains application work.
