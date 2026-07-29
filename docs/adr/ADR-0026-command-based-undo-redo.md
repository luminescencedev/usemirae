# ADR-0026 — Command-Based Undo and Redo

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Undo implemented as arbitrary field restoration would bypass validation, transactions, events, and UI synchronization.

---

## Decision

Undo and redo will execute as commands and transactions using inverse operations or bounded prior-state records.

---

## Consequences

### Positive

- invariants remain enforced;
- generation and event ordering remain valid;
- consistent UI synchronization;
- auditable history;
- deterministic testing.

### Negative

- each command needs undo policy;
- memory-bound history design;
- external operations remain separate.

---

## Alternatives Considered

### Snapshot entire project after every edit

Rejected as the primary method because memory use is excessive for large projects.

### Direct mutable rollback

Rejected because it bypasses transaction architecture.

---

## Related Specifications

- `04-project/405-command-history-and-undo-redo.md`
- `01-runtime/104-command-system.md`
- `01-runtime/107-transactions.md`
