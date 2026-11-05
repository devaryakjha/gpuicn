# Avatar Group parity

Visual reference: shadcn/ui's avatar stack example (`-space-x-2` with a
`ring-background` ring), Neutral Nova. Implementation:
`crates/ui/registry/avatar_group/avatar_group.rs`, using GPUI Kit 0.9.0.

Each face is a gpuicn `Avatar` at the group's size (`Sm` by default). The
group overlaps faces by a third of their width and frames each in a 2px ring
of the `background` token; `ring_color` matches another surface. Past `max`
people (four by default) the rest collapse into a muted `+N` chip in the
last slot, so the row never grows past `max` items.

The group is one Image named with `aria_label`'s words and every name
("Acknowledged by Ana, Ben and Cy"), and a hover tooltip lists the names.
The application owns who is in it and what clicking it does.
