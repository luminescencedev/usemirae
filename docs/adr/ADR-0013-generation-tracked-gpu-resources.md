# ADR-0013 — Generation-Tracked GPU Resources

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

GPU resources become invalid after device recreation, surface reconfiguration, external-handle replacement, and some pool reuse scenarios.

Raw indices or shared pointers do not reliably prevent stale use.

---

## Decision

Every GPU-facing handle will include device and resource generation.

Pools, caches, surfaces, imported textures, and exported resources validate generations before use.

---

## Consequences

### Positive

- stale-handle detection;
- safer device-loss recovery;
- explicit surface reconfiguration;
- improved diagnostics;
- reduced accidental use-after-recreate behavior.

### Negative

- more metadata in handles;
- validation overhead;
- implementation complexity in pools and caches.

---

## Alternatives Considered

### Raw backend handles

Rejected because validity depends on hidden lifetime assumptions.

### Plain reference counting

Rejected because CPU lifetime does not represent GPU completion or device validity.

---

## Related Specifications

- `02-rendering/205-renderer-backend.md`
- `02-rendering/206-gpu-resource-model.md`
- `02-rendering/212-device-loss-and-recovery.md`
