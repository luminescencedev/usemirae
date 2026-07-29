# 708 — Command, Event, and State Integration

**Status:** Proposed  
**Audience:** SDK, runtime, domain, UI contributors  
**Canonical:** Yes  
**Required context:** `01-runtime/104-command-system.md`, `01-runtime/105-event-system.md`, `01-runtime/106-state-store.md`  
**Related ADRs:** ADR-0007, ADR-0009, ADR-0048

---

## 1. Purpose

This document defines how extensions invoke commands, subscribe to events, read state projections, and store their own project data without bypassing engine authority.

---

## 2. Command Invocation

An extension may invoke only commands exposed to its capabilities.

Command request includes:

- command kind;
- extension actor context;
- expected generation when relevant;
- idempotency key;
- payload;
- correlation ID.

The engine performs ordinary validation and transaction flow.

---

## 3. Custom Extension Commands

An extension may register custom commands that operate only on:

- extension-owned runtime state;
- extension project namespace;
- provider instances it owns.

A custom command cannot directly mutate core project entities.

Cross-domain changes require host-defined approved commands.

---

## 4. Event Subscription

Subscriptions specify:

- event types;
- entity scope;
- project scope;
- delivery priority;
- rate limit;
- queue capacity;
- coalescing;
- replay requirement.

The host filters every event.

---

## 5. State Projections

Extensions receive explicit projections, not the entire engine state.

Examples:

- selected scene summary;
- source status for owned sources;
- output status for owned outputs;
- extension namespace data;
- current project metadata;
- capability snapshot.

Projection schemas are versioned.

---

## 6. Generations

State projection includes:

- engine session ID;
- project ID;
- state generation;
- projection version;
- capability generation.

On gap or new session, the extension requests a new snapshot.

---

## 7. Extension Project Data

An extension may mutate its namespaced project data through transactions.

Data must satisfy:

- schema;
- size bound;
- version;
- no secrets;
- portability policy;
- migration support;
- undo policy.

---

## 8. Undo and Redo

Extension project mutations declare:

- undoable;
- non-undoable with explanation;
- coalescing key;
- inverse or prior-state strategy.

Undo remains an engine command and transaction.

---

## 9. Reentrancy

Extensions must not assume synchronous event delivery during command execution.

Events are delivered after commit.

An event handler invoking a new command creates a new transaction.

---

## 10. Invariants

1. Engine remains mutation authority.
2. Extension commands are capability-filtered.
3. Custom commands cannot bypass core domain rules.
4. State projections are filtered and versioned.
5. Events follow commit.
6. Gaps trigger snapshot refresh.
7. Extension project data is namespaced and bounded.
8. Undo uses engine transaction architecture.
9. Event handlers do not run inside original transaction.
10. Actor identity is preserved in audit/diagnostics.

---

## 11. Required Tests

- approved command;
- denied command;
- generation conflict;
- custom namespace command;
- forbidden core mutation;
- event filtering;
- event gap;
- projection snapshot;
- extension data undo;
- extension absent data preservation;
- event-handler reentrancy;
- actor audit.

---

## 12. AI Implementation Notes

Do not pass mutable state references to extensions.

Do not publish pre-commit events.

Do not let custom extension commands modify core entities through arbitrary JSON patches.

Keep projections minimal and capability-scoped.
