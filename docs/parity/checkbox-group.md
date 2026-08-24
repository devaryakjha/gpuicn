# Checkbox Group parity

- Upstream visual source: shadcn/ui `4.19.0` Nova Checkbox; behavior source: Base GPUI Checkbox Group.
- gpuicn: `registry/checkbox_group/checkbox_group.rs`.
- Base GPUI supplies group value ownership, controlled/uncontrolled values, and disabled propagation. The wrapper exposes individual styled group items; labelled field composition stays in the Field layer.
