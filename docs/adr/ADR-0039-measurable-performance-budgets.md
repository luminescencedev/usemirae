# ADR-0039 — Measurable Performance Budgets

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Real-time software can appear fast in simple cases while failing under load or at tail latency.

Unmeasured performance claims are not actionable.

---

## Decision

Mirae will define workload-specific budgets and measure median, p95, p99, drops, throughput, memory, and startup behavior.

Regressions will be compared against versioned baselines.

---

## Consequences

### Positive

- objective performance decisions;
- early regression detection;
- explicit headroom;
- better hardware support claims.

### Negative

- benchmark infrastructure;
- hardware runner maintenance;
- noise analysis;
- pressure to maintain fixtures.

---

## Alternatives Considered

### Manual subjective testing only

Rejected because it cannot detect small or tail-latency regressions reliably.

---

## Related Specifications

- `06-quality/601-performance-budgets.md`
- `06-quality/610-benchmarking-and-regressions.md`
