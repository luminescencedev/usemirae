# 924 — UI Performance and Virtualization

**Status:** Proposed  
**Audience:** Product design, UI engineering, desktop-shell, QA, and coding agents  
**Canonical:** Yes

---

## Performance Goals

- direct manipulation remains responsive at 60 Hz;
- no UI main-thread task above 50 ms without mitigation;
- command feedback appears immediately while authoritative result is pending;
- large lists remain smooth;
- hidden panels stop unnecessary work.

## Architecture

Separate stores for:

- durable engine projections;
- local form drafts;
- selection/viewport state;
- high-frequency meters;
- transient operations;
- panel layout.

## Virtualization

Use TanStack Virtual for large scenes, sources, assets, logs, device lists, and command results when thresholds justify it.

## Rendering

- memoize stable rows;
- avoid one global state object;
- batch/coalesce engine patches;
- use CSS transforms for drag/motion;
- imperative canvas/meter rendering where appropriate;
- profile before adding complexity.
