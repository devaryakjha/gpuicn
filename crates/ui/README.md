# gpuicn

45 shadcn-styled components for [GPUI](https://www.gpui.rs/), from Button and
Dialog to Sidebar, Resizable and Virtual List. Behavior comes from
[GPUI Kit](https://github.com/longbridge/gpui-kit).

This crate is gpuicn's *crate mode*. The usual way to set it up is the CLI:

```sh
cargo install gpuicn-cli --version 0.6.0 --locked
gpuicn new my-app --crate
```

That adds this crate with matching GPUI pins and fonts, and writes
`src/ui/mod.rs` as `pub use gpuicn::*;`. Then `gpuicn eject <component>` copies
one component's source into your app when you want to change it, without
changing any call site.

To use the crate directly, depend on it next to the GPUI Kit version it pins,
and call `gpuicn::theme::init(cx)` once at startup:

```rust
use gpuicn::button::Button;

Button::new("save").label("Save")
```

See the [catalog](https://ui.imajha.com/) for every component, and the
[repository](https://github.com/devaryakjha/gpuicn) for source mode, theming
and the changelog.
