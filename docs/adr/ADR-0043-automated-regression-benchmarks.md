# ADR-0043 — Automated Regression Benchmarks

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Performance regressions often enter gradually and are difficult to detect through manual testing.

---

## Decision

Mirae will maintain automated benchmark suites, hardware-specific baselines, regression thresholds, soak tests, and machine-readable comparison reports.

---

## Consequences

### Positive

- earlier regression detection;
- measurable trade-offs;
- historical data;
- support for performance bisect.

### Negative

- dedicated runner cost;
- noise management;
- baseline review work.

---

## Alternatives Considered

### Benchmark only before major releases

Rejected because regression sources become harder to identify.

---

## Related Specifications

- `06-quality/601-performance-budgets.md`
- `06-quality/610-benchmarking-and-regressions.md`
