# Tag parity

shadcn/ui has no Tag. gpuicn's Tag follows GPUI Kit 0.8.1's `Tag` role, drawn
with Nova tokens. Implementation: `crates/ui/registry/tag/tag.rs`.

A tag is a 24px rectangular chip with the medium radius, a hairline border and
12px medium text, which sets it apart from the pill-shaped Badge. Secondary,
Outline and Destructive treatments come from the theme. `Tag::dot` adds a status
dot in an application color, for presence such as online or away; `Tag::icon`
adds a Lucide icon.

`Tag::on_remove` adds a ghost icon button that takes keyboard focus and is
announced as "Remove" plus the label. The application removes the tag from its
own state in the handler.
