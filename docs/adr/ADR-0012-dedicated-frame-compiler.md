# ADR-0012 — Dedicated Frame Compiler

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Semantic scene state and executable render work have different lifetimes, dependencies, and validation rules.

Directly rendering from scene objects would couple persisted state to GPU execution.

---

## Decision

Mirae will use a dedicated frame compiler between scene graph and render graph.

The compiler produces a renderer-independent, immutable frame render plan.

---

## Consequences

### Positive

- renderer-independent scene semantics;
- deterministic snapshot testing;
- multi-surface planning;
- clear caching and invalidation;
- better diagnostics;
- device recovery without scene mutation.

### Negative

- additional intermediate representation;
- compile-time CPU cost;
- cache complexity;
- mapping across layers requires IDs and generations.

---

## Alternatives Considered

### Scene nodes issuing render commands

Rejected because it leaks execution and backend details into domain state.

### Render graph built directly by UI or source plugins

Rejected because authority and validation would be fragmented.

---

## Related Specifications

- `02-rendering/201-scene-graph.md`
- `02-rendering/202-frame-compiler.md`
- `02-rendering/203-render-graph.md`
