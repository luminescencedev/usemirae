# ADR-0020 — Encoders Behind Stable Interfaces

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Mirae may use software encoders and multiple platform/vendor hardware encoders.

Each exposes different APIs, capabilities, reconfiguration behavior, and memory interop.

---

## Decision

All encoders will implement Mirae-owned stable interfaces and capability descriptors.

Vendor, platform, and FFmpeg types remain inside adapters.

---

## Consequences

### Positive

- encoder replacement;
- common output architecture;
- hardware fallback;
- test doubles;
- contained platform complexity.

### Negative

- lowest-common-denominator pressure;
- advanced features need namespaced extensions;
- adapter maintenance.

---

## Alternatives Considered

### Direct vendor SDK use in output code

Rejected because output logic would become vendor-specific.

### FFmpeg encoder types as public contract

Rejected because it would couple the domain to FFmpeg ABI and semantics.

---

## Related Specifications

- `03-media/309-encoder-system.md`
- `03-media/310-output-architecture.md`
