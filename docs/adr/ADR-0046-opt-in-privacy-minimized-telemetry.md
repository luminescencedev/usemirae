# ADR-0046 — Opt-In Privacy-Minimized Telemetry

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Diagnostics can improve reliability, but Mirae handles sensitive projects, media, paths, and credentials.

Mandatory or broad telemetry would conflict with local-first operation and user trust.

---

## Decision

Usage and diagnostic telemetry will be opt-in, purpose-scoped, bounded, inspectable, and privacy-minimized.

Core operation will not depend on telemetry.

---

## Consequences

### Positive

- stronger privacy;
- user control;
- alignment with local-first design;
- reduced data-risk exposure.

### Negative

- less complete population data;
- consent and settings UX;
- more careful event design;
- lower automatic crash visibility.

---

## Alternatives Considered

### Mandatory analytics

Rejected because it is unnecessary for core operation and conflicts with local-first privacy goals.

---

## Related Specifications

- `06-quality/617-privacy-and-telemetry.md`
- `06-quality/608-crash-reporting.md`
