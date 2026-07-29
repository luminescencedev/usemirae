# ADR-0010 — Explicit Transaction Boundaries

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Many user actions affect multiple entities and derived indexes.

Partial visibility would corrupt invariants and make undo, persistence, and UI synchronization unreliable.

---

## Decision

Authoritative mutations will commit through explicit in-memory transactions.

A transaction validates, prepares, revalidates, atomically commits, increments state generation once, and publishes resulting patches and events.

External I/O and long-running work remain outside the commit-critical section.

---

## Consequences

### Positive

- atomic state changes;
- strong invariants;
- reliable undo records;
- consistent state patches;
- short serialized mutation path;
- explicit conflict handling.

### Negative

- transaction coordinator complexity;
- external side effects require operation patterns;
- candidate-state construction must be efficient.

---

## Alternatives Considered

### Direct mutable state changes

Rejected because partial state becomes observable.

### Holding transaction open across external work

Rejected because it blocks progress and cannot provide true external atomicity.

---

## Related Specifications

- `01-runtime/104-command-system.md`
- `01-runtime/106-state-store.md`
- `01-runtime/107-transactions.md`
