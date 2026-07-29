# ADR-0004 — Scene Graph and Render Graph Separation

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

A live production composition has two different representations:

1. semantic user intent;
2. executable GPU work for a specific frame and surface.

Combining them would make project state depend on renderer details and would complicate preview/program divergence, transitions, output-specific rendering, device recovery, and testing.

---

## Decision

Mirae will maintain separate scene graph and render graph models.

The scene graph stores semantic composition:

- hierarchy;
- source instances;
- transforms;
- visibility;
- masks;
- effects;
- grouping;
- nested scenes.

The frame compiler resolves scene state and live runtime inputs into one or more render graphs.

The render graph stores frame-specific execution:

- passes;
- resources;
- dependencies;
- barriers;
- temporary resource requirements;
- surface targets.

---

## Consequences

### Positive

- domain state remains renderer-independent;
- multiple surfaces can compile from one scene;
- render optimization does not mutate project intent;
- device resources can be rebuilt;
- scene behavior is easier to serialize and test;
- transitions can compile source and destination states.

### Negative

- compiler layer adds complexity;
- invalidation and caching must be designed;
- mapping diagnostics back to scene items requires metadata;
- duplicate representations require generation tracking.

---

## Alternatives Considered

### Scene nodes directly issue GPU commands

Rejected because domain state would become tied to execution and backend lifetimes.

### One universal graph

Rejected because semantic hierarchy and execution dependency graph have different invariants and lifecycles.

---

## Implementation Notes

- Scene graph entities use stable persistent IDs.
- Render graph nodes use frame or generation-scoped IDs.
- Compiler output is derived state.
- Renderer must not mutate scene graph.
- Diagnostics should retain source scene item references.
- Compilation caching must use explicit generations.

---

## Related Specifications

- `00-foundations/005-domain-model.md`
- future `02-rendering/201-scene-graph.md`
- future `02-rendering/202-render-graph.md`
- future `02-rendering/203-compositor.md`
