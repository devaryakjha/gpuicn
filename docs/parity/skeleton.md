# Skeleton parity

Visual reference: shadcn/ui 4.19.0 `skeleton`, Neutral Nova. Implementation:
`crates/ui/registry/skeleton/skeleton.rs`, using GPUI Kit 0.8.1.

The skeleton is a muted block with the medium radius that pulses between full and
half opacity every two seconds, like Tailwind's `animate-pulse`. Its default size
is full width and 16px high; set `w`, `h`, `size` and `rounded_full` to match the
content it replaces.

It takes a stable ID because GPUI keys the pulse animation on it. Under reduced
motion the block is static. Skeletons carry no accessible role, so announce the
loading state with a Spinner or label nearby.
