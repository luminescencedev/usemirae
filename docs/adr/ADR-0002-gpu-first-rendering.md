# ADR-0002 — GPU-First Rendering

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Live production requires real-time composition of multiple dynamic video sources, graphics, masks, effects, scaling operations, and output surfaces.

A CPU-first composition path would create:

- high memory bandwidth use;
- repeated frame copies;
- limited effect scalability;
- expensive high-resolution workflows;
- difficult hardware-encoder interop.

---

## Decision

Mirae will use a GPU-first rendering architecture.

`wgpu` is the initial graphics abstraction.

Video frames should remain GPU-resident from acquisition or upload through composition and encoder handoff when platform capabilities permit.

The design will use:

- semantic scene graph;
- frame compiler;
- render graph;
- pooled resources;
- explicit color processing;
- asynchronous GPU timing and diagnostics;
- platform-specific interop behind renderer abstractions.

---

## Consequences

### Positive

- scalable composition;
- lower steady-state CPU copy cost;
- modern shader effects;
- reusable intermediate resources;
- strong path to hardware encoder interop;
- measurable pass-level performance.

### Negative

- device loss and driver behavior require explicit recovery;
- cross-platform interop is complex;
- GPU memory pressure must be managed;
- shader validation and compatibility require investment;
- some sources still require CPU upload.

---

## Alternatives Considered

### CPU compositor

Rejected as the primary architecture. It may remain as a testing or fallback tool for restricted cases.

### Platform-specific renderer per OS

Rejected as the domain architecture because it would fragment semantics. Platform interop remains specialized behind a shared rendering model.

### OpenGL

Rejected as the primary graphics API because modern explicit resource and synchronization behavior is preferable.

---

## Implementation Notes

- Avoid implicit per-frame allocation.
- Use bounded texture and buffer pools.
- Track resource generation.
- Do not expose `wgpu` types in domain crates.
- Define device-loss recovery.
- Keep render graph and scene graph separate.
- Instrument GPU pass timings.

---

## Related Specifications

- `00-foundations/003-design-principles.md`
- `00-foundations/004-system-overview.md`
- future `02-rendering/202-render-graph.md`
- future `02-rendering/205-gpu-resource-model.md`
