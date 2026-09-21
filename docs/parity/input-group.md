# Input Group parity

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Implementation:
`registry/input_group/input_group.rs`, using GPUI Kit 0.6.4 editing states through
gpuicn's `Input` and `Textarea`.

Input Group provides one shared frame for a single-line input or textarea, with
inline start/end addons and block start/end toolbars. `InputGroupButton` keeps
gpuicn Button keyboard and accessibility behavior and inherits the group's
disabled state. The caller retains the editing state and owns validation and
submission. Input and Textarea paste hooks remain available inside the group.
`Field::from_control` can label and validate the whole composition.

The Rust source and Usage example define the supported API. Browser DOM/CSS
behavior, arbitrary nested addon controls and cross-platform screen-reader
parity are not implied by the native component. See
[migration qualification](../validation/gpui-kit-migration.md) for the checks
completed on this revision.
