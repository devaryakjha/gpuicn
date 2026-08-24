# Toolbar parity

- Upstream: no shadcn/ui Toolbar component exists in the pinned `4.19.0` base registry.
- gpuicn: `registry/toolbar/toolbar.rs`.
- Difference type: visual adaptation.

Base GPUI owns toolbar role semantics, roving focus, keyboard activation, disabled cascading, and composite input behavior. The port applies the same Neutral Nova controls as Button, Tabs, and Navigation Menu. This is intentionally a visual adaptation rather than a claimed shadcn API or source port.
