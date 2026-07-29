# 603 — Resource Lifetimes

**Status:** Proposed  
**Audience:** Runtime, rendering, media, platform, SDK contributors  
**Canonical:** Yes  
**Required context:** `602-memory-model.md`, `01-runtime/102-engine-lifecycle.md`  
**Related ADRs:** ADR-0013

---

## 1. Purpose

This document defines lifecycle phases and destruction guarantees for resources that cross threads, callbacks, GPU submissions, process boundaries, or native APIs.

---

## 2. Lifetime Classes

- engine-session lifetime;
- active-project lifetime;
- source-runtime lifetime;
- output-runtime lifetime;
- device-generation lifetime;
- frame/submission lifetime;
- transaction lifetime;
- operation lifetime;
- extension-host lifetime;
- UI-view lifetime.

Every resource belongs to one primary lifetime class.

---

## 3. Creation

Creation flow:

1. validate owner state;
2. allocate native/logical resource;
3. establish generation;
4. register cleanup;
5. publish handle only after valid;
6. emit diagnostic ownership.

Partially created resources are cleaned before returning failure.

---

## 4. Use

A handle is valid only when:

- owner is alive;
- session/generation matches;
- capability remains valid;
- lease has not expired;
- resource is not retired;
- thread/process constraints are respected.

---

## 5. Retirement

Retirement phases may include:

- stop accepting new work;
- detach from registry;
- wait for bounded in-flight use;
- release external ownership;
- close native handle;
- return to pool or free;
- publish stopped state.

Destruction is not always immediate after logical removal.

---

## 6. Generation Changes

Generation changes occur on:

- engine restart;
- device loss;
- surface reconfiguration;
- source reconnect;
- encoder restart;
- worker restart;
- extension host restart;
- file seek where queued data invalidates.

Old handles fail validation rather than pointing to replacement resource.

---

## 7. Cancellation Trees

Parent cancellation propagates to children.

Examples:

- engine stop cancels project, source, output, and operation scopes;
- source stop cancels capture/decode workers;
- output stop cancels encoder/muxer/sink workers;
- extension disable cancels extension calls and media leases.

Children may perform bounded cleanup.

---

## 8. Cross-Thread Destruction

Resources with thread affinity are destroyed on owning executor.

A generic drop from another thread schedules cleanup and does not violate API constraints.

Shutdown ensures cleanup executor remains alive until affiliated resources retire.

---

## 9. Cross-Process Handles

Shared handles include:

- owner process;
- generation;
- access rights;
- release protocol;
- peer-death behavior.

A crashed peer must not keep unbounded resources indefinitely.

---

## 10. Invariants

1. Every resource has one primary lifetime owner.
2. Handles include required generation.
3. Partial creation cleans up.
4. Logical removal precedes bounded physical retirement.
5. Thread-affine destruction occurs on owner.
6. Parent cancellation propagates.
7. Old generation handles never alias replacements.
8. Peer death releases or times out resources.
9. Cleanup is observable.
10. Shutdown order preserves cleanup executors.

---

## 11. Required Tests

- partial creation failure;
- stale generation;
- in-flight retirement;
- thread-affine cleanup;
- parent cancellation;
- peer crash;
- device loss;
- source reconnect;
- output restart;
- engine shutdown ordering;
- timeout escalation;
- double-release prevention.

---

## 12. AI Implementation Notes

Do not rely on ordinary object drop when the native API requires thread-affine cleanup.

Do not reuse resource IDs without generation.

Do not publish handles before construction fully succeeds.

Model cancellation and retirement separately.
