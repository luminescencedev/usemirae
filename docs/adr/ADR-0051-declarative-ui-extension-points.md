# ADR-0051 — Declarative UI Extension Points

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Arbitrary UI code in the trusted control origin could read state, interfere with accessibility, inject styles, or compromise the shell.

---

## Decision

Preferred extension UI will use declarative host-rendered component schemas.

Rich extension UI, when supported, runs in an isolated origin with a scoped bridge.

---

## Consequences

### Positive

- consistent design;
- accessibility;
- localization;
- security isolation;
- easier compatibility.

### Negative

- limited custom UI freedom;
- schema evolution;
- rich tools may require isolated views.

---

## Alternatives Considered

### Load arbitrary extension JavaScript into the control UI

Rejected because it breaks origin and trust isolation.

---

## Related Specifications

- `07-sdk/710-ui-extension-points.md`
- `05-platform/501-desktop-shell.md`
