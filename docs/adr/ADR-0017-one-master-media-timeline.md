# ADR-0017 — One Master Media Timeline

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Capture devices, files, audio hardware, networks, and output transports each expose different clocks and timebases.

Without one conceptual timeline, transitions, A/V sync, replay, and multi-output timestamps become inconsistent.

---

## Decision

Mirae will maintain one master media timeline per active production session, based on monotonic time and rational timestamp representation.

Source and device clocks map into this timeline.

Wall clock will not schedule continuous media.

---

## Consequences

### Positive

- coherent A/V synchronization;
- deterministic transitions;
- stable long-running timing;
- explicit drift correction;
- consistent output timestamps.

### Negative

- clock-mapping complexity;
- discontinuity handling required;
- device feedback must be modeled carefully.

---

## Alternatives Considered

### Wall-clock scheduling

Rejected because wall clock can jump.

### Independent source timelines without master mapping

Rejected because cross-source and output synchronization would be ambiguous.

---

## Related Specifications

- `03-media/305-master-clock-and-timebase.md`
- `03-media/308-synchronization.md`
- `01-runtime/103-frame-scheduler.md`
