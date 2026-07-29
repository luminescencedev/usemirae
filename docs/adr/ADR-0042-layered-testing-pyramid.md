# ADR-0042 — Layered Testing Pyramid

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Mirae combines pure domain logic, native adapters, real-time media, GPU execution, multiple processes, and UI.

No single test layer can validate all contracts efficiently.

---

## Decision

Mirae will use layered unit, component, integration, platform, end-to-end, performance, fault, security, and accessibility tests.

Critical dependencies will be injectable for deterministic tests.

---

## Consequences

### Positive

- faster local feedback;
- realistic integration coverage;
- deterministic recovery tests;
- platform-specific confidence;
- clearer failure localization.

### Negative

- more test infrastructure;
- fixture maintenance;
- matrix complexity;
- some hardware tests remain expensive.

---

## Alternatives Considered

### End-to-end tests only

Rejected because they are slow, brittle, and poor at isolating failures.

### Unit tests only

Rejected because process, platform, GPU, and lifecycle behavior would remain unverified.

---

## Related Specifications

- `06-quality/609-testing-strategy.md`
- `06-quality/611-fault-injection.md`
