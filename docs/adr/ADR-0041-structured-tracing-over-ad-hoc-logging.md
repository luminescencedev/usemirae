# ADR-0041 — Structured Tracing over Ad Hoc Logging

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Mirae spans multiple processes, threads, frames, outputs, and recovery operations.

Unstructured text logs cannot reliably correlate these flows.

---

## Decision

Mirae will use structured logs and tracing spans with stable event names, correlation IDs, bounded storage, and redaction classes.

---

## Consequences

### Positive

- cross-process diagnostics;
- performance analysis;
- machine-readable support data;
- safer redaction;
- incident coalescing.

### Negative

- instrumentation discipline;
- schema maintenance;
- overhead measurement required.

---

## Alternatives Considered

### Free-form console logging

Rejected because it is difficult to correlate, parse, redact, and bound.

---

## Related Specifications

- `06-quality/606-logging-and-tracing.md`
- `06-quality/607-observability-and-diagnostics.md`
