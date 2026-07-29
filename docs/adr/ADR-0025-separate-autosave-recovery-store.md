# ADR-0025 — Separate Autosave Recovery Store

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Autosave must preserve recent work without changing the user's explicit-save contract or repeatedly replacing the canonical project file.

---

## Decision

Autosave and crash recovery will use a separate bounded recovery store keyed by project identity and base explicit-save identity.

---

## Consequences

### Positive

- clear explicit-save semantics;
- safer crash recovery;
- multiple recovery candidates;
- canonical project remains untouched.

### Negative

- additional storage and cleanup;
- candidate comparison logic;
- recovery UI required.

---

## Alternatives Considered

### Autosave directly over project file

Rejected because it destroys the distinction between user save and background recovery and increases corruption risk.

### Keep recovery only in memory

Rejected because it does not survive crashes or power loss.

---

## Related Specifications

- `04-project/404-autosave-and-recovery.md`
- `04-project/402-project-library.md`
