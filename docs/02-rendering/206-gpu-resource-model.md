# 206 — GPU Resource Model

**Status:** Proposed  
**Audience:** GPU, rendering, performance contributors  
**Canonical:** Yes  
**Required context:** `203-render-graph.md`, `205-renderer-backend.md`  
**Related ADRs:** ADR-0013

---

## 1. Purpose

This document defines ownership, lifetime, pooling, generation, budgeting, and retirement of GPU resources.

---

## 2. Resource Categories

### 2.1 Persistent resources

Examples:

- cached text atlas;
- long-lived source texture;
- shader module;
- pipeline;
- persistent lookup table;
- surface resources.

### 2.2 Transient graph resources

Exist for one or more in-flight graph executions.

Examples:

- intermediate composition texture;
- blur ping-pong texture;
- temporary mask;
- transition source target.

### 2.3 Imported resources

Owned externally but usable by renderer under contract.

### 2.4 Exported resources

Created or managed by renderer and handed to another owner.

---

## 3. Handle Model

Conceptual handle:

```rust
pub struct GpuHandle<T> {
    pub id: ResourceId,
    pub device_generation: DeviceGeneration,
    pub resource_generation: ResourceGeneration,
    marker: PhantomData<T>,
}
```

A handle is validated before use.

Resource IDs are not raw indices without generation.

---

## 4. Ownership

Every resource has one owning manager.

Other components receive handles or leases.

A lease defines:

- usage;
- lifetime;
- synchronization;
- release;
- whether export is allowed.

Shared ownership through `Arc` does not replace explicit GPU lifetime semantics.

---

## 5. Resource Pooling

Pools are keyed by compatible descriptor:

- dimensions;
- format;
- usage flags;
- sample count;
- mip count;
- memory class;
- device generation.

Pools define:

- maximum bytes;
- maximum resource count;
- idle expiry;
- pressure eviction order;
- metrics.

A pool must not retain unlimited large textures.

---

## 6. Budgeting

Budget categories:

- active imported sources;
- active persistent renderer resources;
- transient graph peak;
- caches;
- surfaces;
- encoder interop;
- safety reserve.

The renderer tracks estimated and observed use.

On pressure:

1. evict rebuildable caches;
2. shrink idle pool;
3. reduce non-production preview resources;
4. reject nonessential surface work;
5. report degraded state;
6. stop with structured error if safe operation is impossible.

---

## 7. In-Flight Frames

The renderer supports a bounded number of in-flight submissions.

Resources cannot be reused until the GPU completion point guarantees they are no longer referenced.

Retirement is associated with submission IDs or fences abstracted by the backend.

---

## 8. Resource Retirement

Retirement queue contains:

- resource handle;
- last submission ID;
- release action;
- device generation;
- diagnostic label.

Queue is bounded by active submissions and cleanup policy.

Device loss may invalidate all resources immediately without normal completion.

---

## 9. Texture Uploads

Upload path defines:

- staging buffer or mapped upload;
- row alignment;
- source format;
- conversion location;
- upload queue capacity;
- reuse of staging resources;
- completion and source ownership.

Repeated allocation for every source frame should be avoided.

---

## 10. Readback

GPU readback is prohibited on the normal composition path.

Allowed explicit use cases:

- screenshot;
- thumbnail export when no GPU path exists;
- diagnostics;
- tests;
- CPU-only sink.

Readback must be asynchronous and bounded.

---

## 11. Cache Types

Possible caches:

- pipelines;
- bind-group layouts;
- samplers;
- text glyphs;
- vector tessellation;
- effect kernels;
- static image uploads;
- scene compile fragments.

Every cache declares:

- key;
- invalidation;
- byte accounting;
- eviction;
- generation scope.

---

## 12. Device Loss

All resource handles become invalid when device generation changes.

Persistent semantic descriptors remain and are used to rebuild.

The resource manager must not attempt to reuse old backend objects.

---

## 13. Invariants

1. Every handle includes generation.
2. Every resource has one owner.
3. Pools and caches are byte-bounded.
4. In-flight reuse waits for completion.
5. Readback is explicit and asynchronous.
6. Device loss invalidates all device-owned resources.
7. Imported resources have acquire/release contract.
8. Resource accounting includes transient peak.
9. Staging resources are reused where safe.
10. Domain state stores descriptors, not resources.

---

## 14. Required Tests

- stale handle rejection;
- pool reuse;
- incompatible descriptor rejection;
- pressure eviction;
- in-flight retirement;
- device generation invalidation;
- upload alignment;
- readback bound;
- cache byte accounting;
- imported resource release;
- large-scene peak estimate.

---

## 15. AI Implementation Notes

Do not use plain integer indices as durable GPU handles.

Do not let `Arc` lifetime alone decide when GPU work has completed.

Do not keep an unlimited texture pool.

Do not add synchronous readback to solve a local integration shortcut.
