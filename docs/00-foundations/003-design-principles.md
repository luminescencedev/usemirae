# 003 — Design Principles

**Status:** Proposed  
**Audience:** All contributors and coding agents  
**Canonical:** Yes  
**Required context:** `001-project-overview.md`, `002-product-and-system-boundaries.md`

---

## 1. Purpose

These principles resolve trade-offs when detailed subsystem specifications do not yet cover a local decision.

They are ordered. A lower principle does not override a higher one without an ADR.

---

## 2. Principle Order

1. Correctness
2. Determinism
3. Reliability and recoverability
4. Security and privacy
5. Real-time safety
6. Performance
7. Maintainability
8. User clarity
9. Extensibility
10. Convenience
11. Visual polish

Visual polish is important, but it does not justify hidden state, unsafe recovery, or unstable output.

---

## 3. Native Core, Replaceable Surfaces

The media engine and critical system integration are native.

The UI is a replaceable control surface. It communicates through typed contracts and does not become the engine.

Implications:

- React state is not authoritative engine state.
- UI crashes should not corrupt the project.
- render cadence does not follow UI cadence.
- native platform features remain accessible without routing them through browser abstractions.

---

## 4. GPU-First, Not GPU-Only

Video frames should remain GPU-resident through composition and output whenever the platform path supports it.

The design should minimize:

- GPU-to-CPU readback;
- duplicate textures;
- format conversion;
- synchronization stalls;
- resource recreation;
- hidden copies between subsystems.

CPU work remains appropriate for:

- scheduling;
- project state;
- validation;
- command processing;
- metadata;
- many codecs;
- networking;
- persistence;
- diagnostics.

---

## 5. Local-First and Offline-Capable

The user's project exists independently of Mirae-operated services.

Implications:

- project open and save are local operations;
- account login is not required for core use;
- network failure cannot block editing;
- credentials use OS secure storage;
- cloud features must degrade gracefully;
- exported projects remain intelligible without a remote database.

---

## 6. Explicit State over Hidden Magic

A user and developer should be able to determine:

- what is active;
- what is pending;
- why output is degraded;
- which scene is previewed;
- which scene is live;
- which source failed;
- which retry policy is running;
- whether a command committed or failed.

The system should avoid implicit mutation triggered by unrelated UI actions.

---

## 7. One Authoritative Owner

Each mutable domain state has one authoritative owner.

Replicas may exist for:

- UI projection;
- preview;
- metrics;
- diagnostics;
- caching.

Replicas must define:

- synchronization mechanism;
- version or generation;
- stale-state behavior;
- conflict behavior.

Shared mutable ownership across unrelated threads is prohibited unless justified and encapsulated.

---

## 8. Commands Mutate, Events Inform

State changes occur through validated commands and transactions.

Events describe committed changes or runtime observations.

An event handler MUST NOT mutate authoritative state through an undocumented side channel.

This separation supports:

- auditability;
- undo/redo;
- IPC;
- deterministic tests;
- recovery;
- extension safety.

---

## 9. Bounded Work

Real-time systems must bound:

- queue lengths;
- retry counts;
- memory growth;
- frame latency;
- log volume;
- extension execution;
- resource pools;
- background indexing;
- shutdown duration.

An unbounded queue is a delayed failure.

When work exceeds capacity, the subsystem must apply an explicit policy such as:

- drop newest;
- drop oldest;
- coalesce;
- backpressure;
- degrade quality;
- isolate the output;
- stop with a diagnostic.

The policy is subsystem-specific and must be documented.

---

## 10. Failure Isolation

A local failure should remain local where possible.

Examples:

- one streaming endpoint failing should not stop local recording;
- one camera failing should preserve the rest of the scene;
- one extension crashing should not terminate the engine;
- one corrupt asset should not make the entire project unreadable;
- UI disconnection should not immediately stop active output.

Failure isolation is not silent failure. Diagnostics must remain visible.

---

## 11. Recovery Is a Feature

Recovery paths are designed, not added after crashes occur.

The architecture must consider:

- partial writes;
- abrupt process termination;
- device removal;
- driver reset;
- network interruption;
- encoder failure;
- extension crash;
- unavailable assets;
- schema migration failure.

Recovery should prefer preserving the current production and user work over restoring optional features.

---

## 12. Deterministic Domain Behavior

Given the same:

- accepted command;
- authoritative state;
- schema version;
- platform-independent inputs;

the domain transition should produce the same result.

Nondeterministic operations such as time, random identifiers, device order, and external state should enter through explicit interfaces.

---

## 13. Stable Contracts, Replaceable Implementations

Domain interfaces should remain stable while implementation changes behind adapters.

Examples:

- FFmpeg is a toolkit behind codec and container abstractions;
- `wgpu` is behind renderer and GPU resource interfaces;
- OS capture APIs are behind capture source interfaces;
- webview IPC is behind a typed transport interface.

A third-party library type must not become the canonical domain model.

---

## 14. Measure Before Optimizing

Performance work should begin with:

- a budget;
- instrumentation;
- a reproducible workload;
- baseline results;
- regression tests when practical.

Speculative complexity is not accepted solely because a path is “real-time.”

However, known architectural hazards such as blocking in an audio callback or per-frame heap growth must be prevented before profiling.

---

## 15. Security by Capability

Components receive only the capabilities they need.

This applies to:

- extensions;
- capture permissions;
- credentials;
- filesystem access;
- network endpoints;
- project migration tools;
- updater;
- crash reporting.

Global unrestricted access is a design smell.

---

## 16. Compatibility Is Explicit

Compatibility is not accidental.

The project defines versions for:

- project schema;
- IPC protocol;
- extension manifests;
- SDK API;
- diagnostics;
- generated schemas.

Migration behavior and deprecation windows are documented.

---

## 17. Accessibility Is Structural

Accessibility is not a final UI audit.

Control surfaces must support:

- keyboard operation;
- visible focus;
- semantic labels;
- screen-reader-compatible structure;
- reduced motion;
- scalable interface text;
- non-color-only state communication.

Engine diagnostics must be representable in accessible UI form.

---

## 18. No Premature General Platform

Mirae should not build a generic framework where a focused internal API is sufficient.

Extensibility is added around stable domain concepts. It is not used to avoid making product decisions.

---

## 19. AI Implementation Notes

When a local requirement is missing, choose the option that:

- preserves one authoritative owner;
- keeps work bounded;
- keeps failure local;
- avoids leaking third-party types;
- remains reversible;
- adds tests;
- does not create new public behavior.

Do not use “temporary” global state, unbounded channels, or direct cross-layer imports. Temporary choices have a strong tendency to become architecture.
