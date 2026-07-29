# 106 — State Store

**Status:** Proposed  
**Audience:** Runtime, domain, UI synchronization, project contributors  
**Canonical:** Yes  
**Required context:** `005-domain-model.md`, `104-command-system.md`, `105-event-system.md`  
**Related ADRs:** ADR-0009, ADR-0010

---

## 1. Purpose

The state store owns authoritative in-memory domain state for the active project and selected engine-level state.

It provides immutable reads and transactional writes.

---

## 2. State Partitions

State is partitioned into:

### 2.1 Project domain state

Persistable intent:

- scenes;
- source definitions;
- scene items;
- output profiles;
- audio configuration;
- assets;
- project metadata.

### 2.2 Session production state

Non-persisted production state:

- preview scene;
- program scene;
- active transition;
- selected operational modes;
- active project identity;
- command history cursor where appropriate.

### 2.3 Runtime registry state

References or summaries of active runtime instances:

- source runtime status;
- output runtime status;
- operation registry;
- extension status.

Heavy handles remain in owning services and are referenced through IDs, not embedded freely into domain snapshots.

### 2.4 Capability state

Platform, device, encoder, and extension capabilities with separate generation.

---

## 3. Read Model

Readers obtain:

- immutable generation-stamped snapshot;
- entity-specific projection;
- bounded read transaction;
- subscription to future patches.

Readers must not receive mutable references to authoritative state.

Conceptual API:

```rust
pub trait StateReader {
    fn generation(&self) -> StateGeneration;
    fn project_snapshot(&self) -> Arc<ProjectState>;
    fn projection<P: Projection>(&self) -> P::Output;
}
```

---

## 4. Write Model

Only transaction coordinator may commit writes.

A transaction:

1. reads current generation;
2. validates preconditions;
3. builds candidate state or delta;
4. validates invariants;
5. computes events and patches;
6. atomically swaps committed state;
7. increments generation;
8. publishes commit outputs.

---

## 5. Copy-on-Write and Structural Sharing

The implementation SHOULD use immutable snapshots or structural sharing where it improves:

- lock duration;
- reader safety;
- snapshot generation;
- UI projection;
- undo/redo.

The choice must be benchmarked for large projects.

Deep cloning the entire project for every small command is not an acceptable permanent architecture without evidence that budgets are met.

---

## 6. Entity Indexes

The store maintains derived indexes for efficient lookup:

- entity ID to entity;
- scene to scene items;
- source to referencing scene items;
- asset to consumers;
- nested-scene dependency graph;
- output profile by ID.

Indexes are derived and validated against canonical entity collections.

They are not independently persisted unless the project schema explicitly includes them.

---

## 7. Snapshot Semantics

A snapshot is immutable and tied to:

- engine session;
- active project ID;
- state generation;
- schema projection version.

Consumers may retain snapshots, but services must avoid unbounded historical retention.

---

## 8. Patch Semantics

A patch advances exactly one known generation range.

A patch includes:

- from generation;
- to generation;
- operations;
- projection schema version;
- checksum or validation metadata where useful.

A consumer that cannot apply a patch requests a new snapshot.

---

## 9. Concurrency

The authoritative commit path is serialized.

Reads may occur concurrently through immutable snapshots.

Long operations do not hold write access.

A transaction may compute candidate changes outside the final commit lock if it revalidates generation and preconditions before commit.

---

## 10. State Ownership

The store owns domain values.

Services own runtime resources.

Example:

```text
State store:
SourceRuntimeStatus { source_id, state, generation, health_summary }

Capture service:
Actual camera handle, callback, buffers, platform objects
```

This prevents state snapshots from retaining heavy or thread-affine objects.

---

## 11. Persistence Boundary

Persistence receives a stable project-domain snapshot.

It excludes:

- active output state;
- live transition progress;
- runtime source handles;
- metrics;
- queue depths;
- engine session IDs;
- UI selection unless stored as separate workspace preference.

---

## 12. Memory Bounds

The state system bounds:

- retained snapshots;
- recent patches;
- idempotency results;
- command history;
- undo records;
- derived cache generations.

Memory pressure may evict rebuildable derived state, never authoritative current state.

---

## 13. Invariants

1. One committed generation is globally authoritative per engine session.
2. Readers never mutate committed state.
3. Commit is atomic.
4. Generation increments once per committed domain transaction.
5. Runtime handles are not serialized.
6. Derived indexes match canonical entities.
7. Patches identify source and destination generation.
8. Snapshot retention is bounded.
9. Persistence receives project intent, not runtime state.
10. Capability generation is distinct from project state generation.

---

## 14. Required Tests

- concurrent immutable reads;
- atomic commit;
- generation conflict;
- index consistency;
- snapshot/patch equivalence;
- patch gap recovery;
- bounded snapshot retention;
- serialization boundary;
- runtime handle exclusion;
- large-project mutation benchmark;
- undo interaction.

---

## 15. AI Implementation Notes

Do not expose `&mut ProjectState` outside the transaction layer.

Do not store capture, decoder, GPU, or socket objects in serializable state.

Do not retain every historical snapshot indefinitely.

Preserve separate generations for domain state and capabilities.
