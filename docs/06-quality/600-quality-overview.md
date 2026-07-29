# 600 — Quality Overview

**Status:** Proposed  
**Audience:** All contributors and coding agents  
**Canonical:** Yes  
**Required context:** `00-foundations/003-design-principles.md`, `00-foundations/007-ai-implementation-contract.md`  
**Related ADRs:** ADR-0039, ADR-0040, ADR-0041, ADR-0042, ADR-0043, ADR-0044, ADR-0045, ADR-0046

---

## 1. Purpose

Quality in Mirae is an architectural property, not a final validation phase.

This section defines measurable expectations for performance, memory, concurrency, reliability, security, testability, accessibility, compatibility, and privacy.

---

## 2. Quality Attributes

Mirae prioritizes:

1. correctness;
2. determinism;
3. reliability and recovery;
4. security and privacy;
5. real-time safety;
6. performance;
7. maintainability;
8. accessibility;
9. compatibility;
10. visual polish.

A change that improves one attribute by violating a higher-priority invariant is not acceptable without an ADR.

---

## 3. Quality Ownership

Every subsystem specification must define:

- owner;
- performance budget;
- memory behavior;
- queue bounds;
- failure behavior;
- diagnostics;
- required tests;
- compatibility impact;
- security impact;
- accessibility impact where user-facing.

Quality is owned by the subsystem, not delegated entirely to a central QA phase.

---

## 4. Quality Evidence

Acceptable evidence includes:

- unit and integration tests;
- deterministic fixtures;
- benchmark results;
- traces;
- memory profiles;
- fault-injection runs;
- security review;
- accessibility audit;
- compatibility matrix;
- crash-free session metrics when opt-in data exists;
- reproducible issue cases.

Claims such as “fast,” “safe,” “stable,” or “accessible” are insufficient without evidence.

---

## 5. Risk Classes

Changes are classified:

### Low risk

- local pure logic;
- documentation;
- non-critical UI copy;
- isolated test-only changes.

### Medium risk

- ordinary UI behavior;
- project command;
- non-critical platform adapter;
- bounded background task.

### High risk

- persistence;
- migration;
- IPC;
- audio callback;
- render scheduling;
- output lifecycle;
- credentials;
- updater;
- extension permissions;
- native/unsafe code.

Risk class determines review, test, and rollout requirements.

---

## 6. Quality Gates

A change may require:

- compile and static analysis;
- unit tests;
- integration tests;
- platform tests;
- performance comparison;
- memory leak check;
- fault injection;
- security review;
- accessibility review;
- migration fixture;
- rollback verification.

The release-gate document defines minimum release criteria.

---

## 7. Quality Invariants

1. Every critical path has a budget.
2. Every queue is bounded.
3. Every failure has a structured category.
4. Every cross-process interaction is observable.
5. Every persisted contract has compatibility tests.
6. Every unsafe boundary has documented safety invariants.
7. Every critical recovery path is fault-tested.
8. Every user-facing workflow is keyboard-operable.
9. Every telemetry path is opt-in and privacy-minimized.
10. No release bypasses mandatory gates without a recorded exception.

---

## 8. Required Tests

This overview is validated indirectly through:

- documentation completeness checks;
- architecture linting;
- release gate automation;
- subsystem test matrices;
- ADR compliance review.

---

## 9. AI Implementation Notes

Do not claim quality based on implementation appearance.

For every change, identify the relevant quality documents and provide actual validation commands or measurements.

When evidence is missing, state that clearly rather than assuming success.
