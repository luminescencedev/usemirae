# 008 — Architecture Change Process

**Status:** Proposed  
**Audience:** Maintainers, architects, reviewers, coding agents  
**Canonical:** Yes  
**Required context:** `000-documentation-contract.md`, `007-ai-implementation-contract.md`

---

## 1. Purpose

This document defines how Mirae changes architectural direction without losing consistency or historical reasoning.

---

## 2. Change Categories

### 2.1 Local implementation change

Examples:

- internal helper extraction;
- private algorithm replacement;
- local data structure optimization;
- error-context improvement.

Requirements:

- no public contract change;
- no schema change;
- no new subsystem dependency;
- tests updated.

No ADR is required.

### 2.2 Specification refinement

Examples:

- clarify an invariant;
- define an overflow policy;
- add a missing failure mode;
- document an existing interface.

Requirements:

- specification review;
- no incompatible behavior hidden as clarification;
- tests aligned.

An ADR is optional unless the refinement selects between meaningful alternatives.

### 2.3 Architectural change

Examples:

- process boundary change;
- authoritative owner change;
- renderer replacement;
- project format semantic change;
- plugin security model change;
- master clock change;
- new cross-subsystem dependency;
- new long-lived technology dependency.

Requirements:

- ADR;
- specification updates;
- migration and compatibility analysis;
- implementation plan;
- tests and rollback strategy.

---

## 3. Proposal Content

An architecture proposal must include:

1. Context
2. Problem
3. Current behavior
4. Proposed behavior
5. Alternatives
6. Trade-offs
7. Compatibility
8. Security
9. Performance
10. Migration
11. Testing
12. Rollout
13. Rollback
14. Documentation impact
15. Open questions

---

## 4. ADR Lifecycle

ADR statuses:

- Proposed
- Accepted
- Rejected
- Deprecated
- Superseded

An Accepted ADR is immutable in its decision record. Later corrections may fix wording, but a changed decision requires a new ADR that supersedes it.

---

## 5. Required Reviewers

An architectural change should be reviewed by owners of every affected area.

Examples:

- runtime;
- rendering;
- media;
- audio;
- persistence;
- platform;
- security;
- SDK;
- UI.

A reviewer must evaluate consequences for their subsystem, not only code style.

---

## 6. Compatibility Analysis

The proposal must identify impact on:

- existing project files;
- autosave and recovery files;
- IPC protocol;
- extension manifests;
- SDK;
- user settings;
- credentials;
- recording and output behavior;
- supported platforms;
- packaging;
- diagnostics and telemetry schemas.

“No compatibility impact” must be justified.

---

## 7. Performance Analysis

For critical-path changes, include:

- workload;
- baseline;
- expected result;
- measurement tooling;
- regression threshold;
- CPU, GPU, memory, latency, and copy impact;
- low-end hardware effect where relevant.

---

## 8. Security Analysis

The proposal must identify:

- new trust boundaries;
- new privileges;
- filesystem or network access;
- secret handling;
- untrusted input;
- code execution;
- update or signature impact;
- extension capability impact.

---

## 9. Rollout Strategy

Possible rollout methods:

- feature flag;
- internal-only build;
- nightly channel;
- dual-read or dual-write migration;
- compatibility adapter;
- staged platform rollout;
- opt-in experimental setting.

A feature flag must have an owner and removal condition. Permanent ambiguous flags are prohibited.

---

## 10. Rollback Strategy

The change must define what happens if:

- data was already migrated;
- a new project was saved;
- an output was created with the new behavior;
- an extension used the new API;
- a user downgrades;
- a platform-specific bug appears.

Irreversible migrations require stronger review.

---

## 11. Implementation Plan

The implementation plan should break work into dependency-ordered units.

Each unit should define:

- specification;
- code areas;
- tests;
- feature flag state;
- compatibility behavior;
- completion signal.

Large architectural work SHOULD be mergeable incrementally without leaving the default branch broken.

---

## 12. Acceptance

An architectural change is accepted only when:

- ADR is Accepted;
- affected specifications are updated;
- unresolved high-risk questions are closed or explicitly deferred;
- owners approve;
- migration and rollback are credible;
- required tests are defined;
- security and performance impact are reviewed.

---

## 13. AI Implementation Notes

An AI agent may draft an ADR and proposal but MUST NOT treat it as Accepted without explicit project approval.

Do not modify canonical architecture merely to fit existing code.

Do not hide an architecture change inside a refactor PR.

When discovering that implementation requires a new cross-subsystem contract, stop, document the contract, and route it through this process.
