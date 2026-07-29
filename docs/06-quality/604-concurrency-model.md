# 604 — Concurrency Model

**Status:** Proposed  
**Audience:** Runtime, rendering, media, audio, platform contributors  
**Canonical:** Yes  
**Required context:** `01-runtime/100-runtime-overview.md`, `603-resource-lifetimes.md`  
**Related ADRs:** ADR-0008

---

## 1. Purpose

This document defines thread/executor roles, synchronization rules, lock ordering, queue policies, cancellation, and race-prevention requirements.

---

## 2. Execution Domains

Expected domains:

- engine control executor;
- IPC tasks;
- blocking I/O pool;
- media decode/encode workers;
- scheduler thread/executor;
- render submission executor;
- GPU completion callbacks;
- audio real-time callback;
- network runtime;
- platform event thread;
- extension host process/tasks;
- UI main thread.

A subsystem must state which domain owns mutation.

---

## 3. Ownership Rule

Mutable authoritative state has one serialized owner.

Concurrency is achieved through:

- immutable snapshots;
- message passing;
- bounded queues;
- prepared state swaps;
- resource leases;
- dedicated workers.

Shared mutable state is exceptional.

---

## 4. Locks

When locks are required:

- scope is minimal;
- lock owner is documented;
- blocking while holding lock is prohibited;
- lock ordering is defined;
- poisoning/recovery behavior is explicit;
- real-time callbacks avoid blocking locks.

---

## 5. Lock Order

Canonical broad order:

```text
Lifecycle
→ State transaction
→ Subsystem registry
→ Runtime instance
→ Local cache/pool
```

Code should avoid acquiring more than one broad lock.

A more specific lock-order table may be defined per subsystem.

---

## 6. Channels

Every channel declares:

- producer;
- consumer;
- capacity;
- message type;
- ordering;
- overflow;
- shutdown behavior;
- metrics.

Unbounded production channels are prohibited.

---

## 7. Async and Blocking Work

Blocking filesystem, process, device, or native operations must not run on shared async executors without offloading.

Conversely, short async state operations should not spawn unnecessary threads.

---

## 8. Cancellation

Cancellation is cooperative and scoped.

Requirements:

- every long operation has cancellation path;
- cancellation cannot imply rollback after commit;
- cleanup is bounded;
- cancellation state is observable;
- child operations inherit parent cancellation.

---

## 9. Deadlock Prevention

Required practices:

- avoid callback into unknown code while holding lock;
- avoid synchronous IPC while holding lock;
- avoid lock inversion across services;
- time out external waits;
- instrument blocked duration;
- test shutdown races.

---

## 10. Real-Time Audio

Audio callback receives:

- immutable/prepared graph;
- lock-free or bounded parameter updates;
- preallocated buffers;
- monotonic clock data;
- no general service calls.

Control thread never waits for audio callback while holding state transaction lock.

---

## 11. Render and Media

Render submission and media workers communicate through generation-stamped leases.

The renderer does not wait on capture callback.

Capture callback does not wait on renderer.

---

## 12. Deterministic Testing

Concurrency tests should support:

- virtual clock;
- controlled executor;
- injected yields;
- deterministic queue capacities;
- race repetition;
- loom-like state exploration where practical;
- fault-triggered shutdown.

---

## 13. Invariants

1. Authoritative mutation has one owner.
2. Channels are bounded.
3. Real-time callbacks do not block.
4. No synchronous external call while holding broad lock.
5. Cancellation is scoped.
6. Lock order is documented.
7. Blocking work is isolated.
8. Shutdown cannot deadlock on child cleanup indefinitely.
9. Generations prevent stale cross-thread work.
10. Race tests exist for critical lifecycles.

---

## 14. Required Tests

- concurrent commands;
- source stop during callback;
- output stop during reconnect;
- shutdown during save;
- device loss during render;
- UI disconnect during transaction;
- queue saturation;
- lock-order instrumentation;
- cancellation propagation;
- extension callback timeout;
- audio graph swap;
- repeated race stress.

---

## 15. AI Implementation Notes

Do not add a global mutex around the engine.

Do not use unbounded channels.

Do not call arbitrary extension or OS code while holding state locks.

Document owner, capacity, and overflow for every new queue.
