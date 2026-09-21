# Carousel

Visual reference: shadcn/ui 4.19.0, Neutral Nova. Behavior comes from GPUI Kit 0.6.4.

`CarouselState` owns selection, orientation, looping, snapping, and gesture state. Retain one state entity and pass it to `Carousel`, `CarouselContent`, every `CarouselItem`, and the optional controls. The application can subscribe to `CarouselEvent::Change` or update the state directly for controlled selection.

The content handles pointer drags, precise trackpad gestures, wheel steps, and horizontal or vertical snapping. The focused carousel uses Arrow keys, Home, and End. Looping uses GPUI Kit's runway layout rather than cloning slide elements. GPUI Kit reads the operating system reduced-motion setting at initialization and removes carousel travel when the preference is active. Carousel snapping uses GPUI Kit's semantic spring; gpuicn's `UiTheme.motion` does not retune that spring.

gpuicn styles the region, arrow controls, and pagination dots with the shared
Nova theme. Arrow and pagination actions retain GPUI Kit's frame placement,
focus handling and selection behavior. `UiTheme::set` projects the mode, focus
ring, shared colors, fonts and radii used by those controls and the carousel root
into GPUI Kit's component theme after a light/dark or custom theme change. Slide
content remains application-owned. Keep item IDs stable, give the region a
useful accessible label, and give custom slide content its own labels when the
generated “Slide N of M” name is not enough.

Native pointer and keyboard checks do not establish performance or full screen-reader parity on every desktop platform. Browser preview checks cover the same Rust component through the WASM renderer.
