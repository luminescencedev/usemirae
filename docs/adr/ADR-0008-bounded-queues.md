# ADR-0008 — Bounded Queues

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Unbounded queues convert overload into delayed memory exhaustion and latency collapse.

Mirae contains real-time and bursty pipelines across IPC, media, rendering, encoding, diagnostics, and extensions.

---

## Decision

Every production queue will have:

- explicit capacity;
- overflow policy;
- ownership;
- metrics;
- shutdown behavior.

No unbounded queue is permitted in a long-lived runtime path without a specific Accepted ADR.

---

## Consequences

### Positive

- bounded memory;
- predictable overload;
- diagnosable drops;
- clearer backpressure;
- safer extension and IPC behavior.

### Negative

- every subsystem must choose a loss or backpressure policy;
- overload handling becomes explicit implementation work;
- poor capacity choices may cause avoidable drops.

---

## Alternatives Considered

### Unbounded async channels

Rejected because they hide overload.

### Global blocking backpressure

Rejected because blocking critical paths can cascade failures.

---

## Related Specifications

- `01-runtime/100-runtime-overview.md`
- `01-runtime/103-frame-scheduler.md`
- `01-runtime/105-event-system.md`
- `01-runtime/108-ipc-protocol.md`
