# ADR-0038 — Hardware Acceleration Is Negotiated, Not Assumed

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

GPU, decoder, encoder, capture, and interop capabilities vary by hardware, driver, OS, package mode, format, and concurrent-session use.

---

## Decision

Every hardware-accelerated path will be capability-negotiated for the specific pipeline.

Mirae will expose explicit fallback and diagnostics.

---

## Consequences

### Positive

- fewer hidden compatibility failures;
- correct multi-GPU behavior;
- safer fallback;
- measurable zero-copy paths;
- user-visible limitations.

### Negative

- more probing and negotiation;
- complex fallback matrices;
- additional test infrastructure.

---

## Alternatives Considered

### Prefer hardware whenever nominally available

Rejected because availability does not guarantee compatibility or stability.

---

## Related Specifications

- `05-platform/506-hardware-acceleration.md`
- `05-platform/514-platform-capability-registry.md`
- `03-media/309-encoder-system.md`
