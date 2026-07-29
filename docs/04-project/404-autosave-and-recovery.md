# 404 — Autosave and Recovery

**Status:** Proposed  
**Audience:** Project, runtime, UI contributors  
**Canonical:** Yes  
**Required context:** `403-persistence.md`, `01-runtime/102-engine-lifecycle.md`  
**Related ADRs:** ADR-0025

---

## 1. Purpose

Autosave preserves recent committed project work without changing the meaning of explicit save.

Recovery restores work after crash, forced termination, or save failure.

---

## 2. Separate Recovery Store

Autosave writes to a recovery store separate from the canonical project file.

A recovery record includes:

- project ID;
- recovery ID;
- base explicit-save identity;
- state generation;
- timestamp;
- app/schema version;
- project snapshot or delta chain;
- integrity metadata;
- crash/session context safe for persistence.

---

## 3. Autosave Triggers

Autosave may trigger by:

- elapsed time while dirty;
- committed command count;
- high-value operation;
- application backgrounding;
- output start;
- project close;
- pre-migration checkpoint;
- shutdown drain.

Triggers are coalesced.

---

## 4. Autosave Scheduling

Autosave must not:

- block control thread;
- block audio callback;
- block renderer;
- create unbounded pending saves;
- continually serialize unchanged state.

One latest pending generation is sufficient unless a checkpoint policy requires more.

---

## 5. Recovery Retention

Retention is bounded by:

- records per project;
- total bytes;
- age;
- explicit-save relationship;
- successful project close;
- user policy.

At least one recent valid recovery candidate should survive an interrupted cleanup.

---

## 6. Recovery Detection

On startup or project open:

1. locate recovery records by project ID;
2. validate integrity;
3. compare base explicit-save identity;
4. compare generations/timestamps;
5. classify candidate;
6. present restore, inspect, save-copy, or discard actions.

Mirae must not auto-overwrite the canonical project.

---

## 7. Recovery Modes

- restore into active unsaved state;
- open as copy;
- inspect differences;
- export recovery package;
- discard;
- repair partial record.

Restore should preserve the original explicit-save file.

---

## 8. Clean Shutdown

On successful explicit save and clean close:

- mark recovery records superseded;
- retain according to short safety window;
- remove only after required durability steps;
- update library metadata.

---

## 9. Crash Context

Recovery metadata may include:

- last engine session;
- lifecycle state;
- last committed generation;
- active project ID;
- active output summary;
- last save failure;
- bounded recent diagnostic references.

It excludes credentials and raw media.

---

## 10. Delta Autosave

A future optimization may store deltas.

If used:

- base snapshot is explicit;
- delta order is validated;
- chain length is bounded;
- compaction exists;
- any missing delta invalidates only dependent tail;
- restore result is deterministic.

A full-snapshot implementation is acceptable initially.

---

## 11. Invariants

1. Autosave never overwrites canonical project directly.
2. Autosave records committed generations only.
3. Recovery records are integrity-checked.
4. Recovery retention is bounded.
5. Restore does not overwrite original automatically.
6. Clean close cleanup is staged safely.
7. Pending autosave work is bounded.
8. Recovery excludes secrets.
9. Explicit save and autosave states remain distinct.
10. Invalid recovery records do not block opening canonical project.

---

## 12. Required Tests

- timed autosave;
- coalesced autosave;
- crash after commit;
- crash during recovery write;
- newer recovery detection;
- stale recovery;
- base file externally changed;
- restore as copy;
- discard;
- retention cleanup;
- secret scanning;
- clean close safety window.

---

## 13. AI Implementation Notes

Do not save autosave data over the project file.

Do not autosave uncommitted optimistic UI state.

Do not delete all recovery records immediately after one successful write.

Keep recovery candidate comparison based on project identity and base save identity.
