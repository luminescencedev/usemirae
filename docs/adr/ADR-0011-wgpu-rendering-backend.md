# ADR-0011 — WGPU Rendering Backend

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Mirae needs a modern cross-platform GPU abstraction supporting Windows, macOS, and Linux while preserving access to explicit resource, pipeline, and synchronization concepts.

---

## Decision

Mirae will use `wgpu` as the initial primary rendering backend.

`wgpu` types remain isolated inside renderer and platform implementation crates.

The domain, scene graph, frame compiler, and project schema will use Mirae-owned types.

---

## Consequences

### Positive

- cross-platform modern graphics abstraction;
- WGSL shader path;
- access to Vulkan, Metal, Direct3D 12, and related backends;
- Rust ecosystem integration;
- consistent validation model.

### Negative

- backend-specific interop still requires platform work;
- not every native API feature is exposed uniformly;
- dependency evolution requires compatibility management;
- driver behavior still differs.

---

## Alternatives Considered

### Direct Vulkan, Metal, and D3D12 implementations

Rejected initially because implementation and maintenance cost would be too high.

### OpenGL

Rejected as the primary backend because it provides weaker modern resource and synchronization semantics.

---

## Related Specifications

- `02-rendering/205-renderer-backend.md`
- `02-rendering/207-shader-system.md`
- `02-rendering/212-device-loss-and-recovery.md`
