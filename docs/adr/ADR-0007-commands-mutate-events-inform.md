# ADR-0007 — Commands Mutate, Events Inform

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

A system where any component mutates shared state or where events are used as implicit commands becomes difficult to reason about, test, undo, synchronize, and secure.

---

## Decision

Authoritative state mutation occurs through validated commands and explicit transactions.

Events are published after commit to describe changes or runtime observations.

Event subscribers do not gain hidden mutation authority.

---

## Consequences

### Positive

- clear ownership;
- deterministic mutation path;
- permission enforcement;
- undo/redo support;
- easier IPC;
- ordered replication;
- better audit and diagnostics.

### Negative

- additional command and event types;
- more explicit plumbing;
- some local interactions require optimistic UI handling.

---

## Alternatives Considered

### Shared mutable stores

Rejected because ownership and ordering become ambiguous.

### Event-sourced mutation through arbitrary subscribers

Rejected as the default because it obscures authority and failure semantics.

---

## Related Specifications

- `01-runtime/104-command-system.md`
- `01-runtime/105-event-system.md`
- `01-runtime/107-transactions.md`
