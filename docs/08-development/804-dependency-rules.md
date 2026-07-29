# 804 — Dependency Rules

**Status:** Proposed  
**Audience:** All contributors and coding agents  
**Canonical:** Yes  
**Required context:** `801-monorepo-architecture.md`  
**Related ADRs:** ADR-0056

---

## 1. Purpose

This document defines allowed dependency direction.

---

## 2. Layer Order

```text
foundation/types/contracts
        ↓
domain
        ↓
application services
        ↓
subsystem interfaces
        ↓
platform/toolkit implementations
        ↓
deployable applications
```

Dependencies flow downward only.

---

## 3. Forbidden Dependencies

Examples:

- domain → platform;
- domain → `wgpu`;
- domain → FFmpeg;
- domain → React/UI;
- project schema → runtime handles;
- UI → native SDK;
- renderer backend → UI;
- media adapter → project persistence internals;
- extension public SDK → engine internals;
- shared library → deployable app.

---

## 4. Interface Ownership

The layer that needs a behavior owns the interface.

Example:

- media domain needs capture → media/platform boundary defines capture interface;
- project service needs filesystem operations → project/application boundary defines safe filesystem interface;
- output needs credentials → output/service boundary consumes credential broker interface.

Implementation lives outward.

---

## 5. Cross-Subsystem Communication

Use:

- commands;
- events;
- immutable snapshots;
- typed interfaces;
- operation handles;
- resource leases.

Do not reach into another subsystem's internal registry.

---

## 6. Dependency Enforcement

Use:

- workspace metadata;
- crate/package linting;
- architecture tests;
- forbidden-import rules;
- code review;
- generated dependency graph;
- CI gate.

Exceptions require ADR.

---

## 7. Shared Types

A type may move downward only if:

- semantics are truly shared;
- no higher-layer behavior leaks;
- the lower crate remains cohesive;
- the type does not force broad dependencies.

Do not move code into `types` simply to break a cycle.

---

## 8. Invariants

1. Domain is independent from implementation frameworks.
2. Interface ownership points inward.
3. Apps assemble; they do not define reusable domain logic.
4. No cycles.
5. Cross-subsystem communication is explicit.
6. Generated contracts do not import runtime implementations.
7. UI depends on client/contracts, not engine crates.
8. Platform adapters do not redefine domain semantics.
9. Exceptions require ADR.
10. CI enforces rules.
