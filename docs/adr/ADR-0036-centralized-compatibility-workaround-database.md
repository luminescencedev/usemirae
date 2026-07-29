# ADR-0036 — Centralized Compatibility Workaround Database

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

GPU drivers, capture APIs, encoder implementations, and packaging modes may require targeted workarounds.

Scattered vendor checks become permanent and untestable.

---

## Decision

Mirae will centralize compatibility workarounds in a typed, versioned, diagnosable database with review and removal criteria.

Optional remote updates must be signed and non-executable.

---

## Consequences

### Positive

- consistent behavior;
- support visibility;
- removable workarounds;
- testable selectors;
- safer remote updates.

### Negative

- database maintenance;
- hardware/version matching complexity;
- risk of stale rules.

---

## Alternatives Considered

### Inline vendor/driver conditionals

Rejected because they spread undocumented behavior across the codebase.

---

## Related Specifications

- `05-platform/515-platform-diagnostics-and-workarounds.md`
- `02-rendering/205-renderer-backend.md`
