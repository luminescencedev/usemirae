# 602 — Memory Model

**Status:** Proposed  
**Audience:** Runtime, rendering, media, project, SDK contributors  
**Canonical:** Yes  
**Required context:** `600-quality-overview.md`, `02-rendering/206-gpu-resource-model.md`, `03-media/303-media-data-model.md`  
**Related ADRs:** ADR-0039

---

## 1. Purpose

This document defines Mirae memory ownership, allocation classes, bounds, pooling, pressure response, and leak expectations.

---

## 2. Memory Domains

- Rust process heap;
- native toolkit allocations;
- GPU device memory;
- mapped/staging memory;
- shared memory;
- media payload pools;
- audio real-time pools;
- replay storage;
- file-system cache;
- extension memory;
- webview/UI memory.

Each domain has an owner and observability strategy.

---

## 3. Ownership Principles

1. one authoritative owner;
2. explicit leases for large shared payloads;
3. generation tracking for replaceable resources;
4. bounded retention;
5. no hidden cycles across service ownership;
6. no runtime object in project persistence;
7. asynchronous retirement for in-flight GPU/media work.

---

## 4. Allocation Classes

### Startup allocations

Long-lived services, registries, caches, and preallocated pools.

### Configuration allocations

Created when project, source, effect, output, or device changes.

### Per-frame allocations

Must be minimized and measured.

### Real-time audio allocations

General heap allocation is prohibited in steady-state callback.

### Burst allocations

Import, migration, bundle extraction, diagnostics export. These require explicit bounds.

---

## 5. Pools

Pools may manage:

- video frames;
- audio blocks;
- encoded packet buffers;
- staging buffers;
- GPU textures;
- command scratch buffers;
- serialization buffers;
- diagnostic events.

Every pool declares:

- maximum bytes;
- maximum item count;
- descriptor compatibility;
- idle eviction;
- pressure behavior;
- metrics.

---

## 6. Shared Ownership

`Arc` or equivalent is allowed for immutable snapshots and leases.

It must not be used to hide unclear ownership or permit unbounded retention.

Long-lived shared references are inspected through:

- strong-count diagnostics in development;
- retention metrics;
- leak tests;
- generation ownership.

---

## 7. Memory Pressure

Pressure levels:

- normal;
- elevated;
- high;
- critical.

Response order:

1. evict rebuildable caches;
2. shrink idle pools;
3. reduce nonessential preview/thumbnail resources;
4. stop optional background work;
5. refuse new nonessential allocations;
6. degrade explicitly;
7. stop affected output/source safely;
8. fail engine only if integrity cannot be preserved.

---

## 8. Large Projects

Large project state uses:

- structural sharing where appropriate;
- indexed lookup;
- bounded undo;
- lazy derived caches;
- paged/library metadata;
- no full deep clone per small mutation without evidence.

---

## 9. Native Toolkits

FFmpeg, webview, OS APIs, and vendor SDK allocations are wrapped.

Adapters report or estimate:

- active objects;
- buffer bytes;
- session count;
- leak-relevant lifecycle;
- shutdown cleanup.

---

## 10. Zeroization

Secret-bearing memory is minimized and zeroized where practical.

Zeroization is best effort and does not replace OS secure storage.

---

## 11. Leak Definition

A leak includes:

- unreachable memory not freed;
- reachable cache with no bound;
- retained generations that can never be used;
- worker/process resources not released after stop;
- GPU resources surviving device generation incorrectly;
- file handles, sockets, or native handles not closed.

---

## 12. Invariants

1. Every pool is byte-bounded.
2. Audio callback avoids general heap allocation.
3. Large payloads use explicit leases.
4. Shared ownership does not replace lifecycle contracts.
5. Memory pressure has ordered response.
6. Undo and snapshot history are bounded.
7. Device loss invalidates GPU resources.
8. Native handles have RAII-style wrappers.
9. Secrets are not retained unnecessarily.
10. Shutdown releases all owned resources within bounded time.

---

## 13. Required Tests

- idle memory plateau;
- repeated project open/close;
- source create/destroy loop;
- output start/stop loop;
- device loss;
- replay retention;
- undo bound;
- cache eviction;
- pressure simulation;
- extension host restart;
- secret lease lifetime;
- native-handle leak checks.

---

## 14. AI Implementation Notes

Do not introduce unbounded caches.

Do not use `Arc` to avoid deciding who owns a resource.

Do not allocate per audio callback.

Add byte accounting and eviction policy with every new pool or cache.
