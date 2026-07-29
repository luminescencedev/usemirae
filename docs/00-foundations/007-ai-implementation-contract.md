# 007 — AI Implementation Contract

**Status:** Proposed  
**Audience:** AI coding agents and human reviewers supervising them  
**Canonical:** Yes  
**Required context:** All foundation documents relevant to the task

---

## 1. Purpose

This document defines how an AI coding agent must use Mirae documentation when planning, implementing, refactoring, testing, or reviewing code.

The goal is not to eliminate implementation judgment. The goal is to prevent undocumented architectural invention.

---

## 2. Required Workflow

Before modifying code, the agent MUST:

1. identify the requested behavior;
2. locate the canonical specification;
3. read all documents listed under `Required context`;
4. identify related ADRs;
5. inspect existing implementation and tests;
6. produce a scoped implementation plan;
7. list assumptions and unresolved conflicts;
8. implement the smallest complete change;
9. add or update tests;
10. verify architecture boundaries;
11. update documentation when a contract changes;
12. report any specification gap.

The agent MUST NOT begin implementation from the user request alone when a canonical specification exists.

---

## 3. Source Priority

The agent uses this priority:

1. Accepted ADRs
2. Accepted canonical specifications
3. Proposed specifications explicitly selected for the current implementation
4. Public schemas and API contracts
5. Approved implementation plan
6. Tests
7. Current code
8. Comments, issues, and informal notes

When sources conflict, the agent must report the conflict. It must not silently choose the easiest source.

---

## 4. Scope Discipline

The agent MUST implement only the requested scope and prerequisites required for correctness.

It MUST NOT:

- add unrelated abstractions;
- redesign adjacent systems without need;
- create broad generic frameworks;
- expose temporary internals as public API;
- introduce new dependencies without justification;
- change persisted schemas without migration;
- bypass command, transaction, or permission systems;
- merge logical layers for convenience;
- create unbounded queues;
- store secrets in plain text;
- add cloud dependencies to core behavior.

---

## 5. Missing Detail Policy

When implementation details are missing, classify them.

### 5.1 Reversible internal detail

The agent MAY choose a small internal design when:

- it is not externally observable;
- it preserves all invariants;
- it is covered by tests;
- it does not establish a cross-subsystem contract;
- it can be replaced without migration.

The choice must be noted in the implementation summary.

### 5.2 Contract-level ambiguity

The agent MUST stop and request or propose a specification update when ambiguity affects:

- public API;
- project schema;
- IPC;
- SDK;
- process boundary;
- thread ownership;
- resource lifetime;
- failure recovery;
- security or permissions;
- output behavior;
- synchronization;
- compatibility.

---

## 6. Architecture Guardrails

The agent MUST preserve:

- one authoritative owner for mutable domain state;
- command-based mutation;
- event-based notification;
- typed versioned IPC;
- process isolation intent;
- platform abstraction;
- domain independence from third-party toolkit types;
- bounded queues and memory;
- atomic persistence;
- explicit credentials;
- extension capability enforcement;
- real-time thread safety.

---

## 7. Dependency Policy

Before adding a dependency, the agent must answer:

1. What requirement does it satisfy?
2. Why is the standard library or existing dependency insufficient?
3. Is it on a critical path?
4. Does it add native code or unsafe code?
5. What is its license?
6. How actively is it maintained?
7. Can its types be contained behind an adapter?
8. What is the replacement cost?
9. Does it affect binary size, startup, or security?
10. Does it support all target platforms?

A dependency used as a toolkit must not become the domain contract.

---

## 8. Code Generation Rules

Generated code MUST be reproducible.

The generator, schema, command, and expected output location must be documented.

Generated files should include a header indicating:

- generated status;
- source schema;
- generator version where relevant;
- instruction not to edit manually.

---

## 9. Test Requirements

For each implementation, the agent must identify:

- unit tests;
- integration tests;
- failure-path tests;
- concurrency tests where relevant;
- serialization or migration fixtures;
- platform tests;
- performance or benchmark coverage;
- regression tests.

A successful happy-path test alone is insufficient for lifecycle, persistence, media, output, or concurrency changes.

---

## 10. Performance Rules

The agent MUST NOT claim performance improvements without measurement.

For critical paths, the plan must identify:

- expected frequency;
- allocation behavior;
- queue behavior;
- blocking behavior;
- lock behavior;
- copy behavior;
- instrumentation.

Prohibited in real-time callbacks unless explicitly specified:

- blocking locks;
- filesystem I/O;
- network I/O;
- IPC waits;
- unbounded allocation;
- logging with synchronous formatting or sinks;
- process launch;
- device enumeration.

---

## 11. Error Handling Rules

The agent must:

- preserve structured errors;
- add context at subsystem boundaries;
- avoid swallowing failures;
- avoid panics for recoverable runtime conditions;
- distinguish user configuration, external failure, internal bug, and compatibility failure;
- emit diagnostics without leaking secrets;
- implement bounded recovery policies.

---

## 12. Documentation Update Rules

The agent MUST update documentation in the same change when it changes:

- a contract;
- an invariant;
- process or thread ownership;
- serialized data;
- command or event semantics;
- an SDK surface;
- failure behavior;
- performance budget;
- capability or permission behavior.

Implementation-neutral bug fixes do not require specification changes unless the specification was ambiguous or wrong.

---

## 13. Required Completion Report

The agent's completion report should include:

```text
Implemented
Files changed
Tests added or updated
Specifications followed
ADRs followed
Assumptions made
Known limitations
Documentation changes
Validation commands
```

The report must distinguish completed work from recommended future work.

---

## 14. Refactoring Rules

A refactor MUST preserve behavior unless the task explicitly changes behavior.

Before a large refactor, the agent must establish:

- baseline tests;
- dependency boundaries;
- performance baseline for critical paths;
- migration strategy;
- rollback strategy.

Do not mix major architecture refactoring with unrelated feature work.

---

## 15. Security Rules

The agent MUST NOT:

- log credentials;
- persist tokens in project files;
- expose unrestricted filesystem paths to extensions;
- execute downloaded native code in the engine;
- disable certificate validation;
- trust extension manifests without validation;
- deserialize untrusted data into unsafe structures without bounds;
- accept arbitrary command names through IPC without schema validation.

---

## 16. AI Implementation Notes

This document is the controlling contract for AI-driven implementation.

When asked to “just make it work,” preserve architecture and reduce scope rather than bypassing invariants.

When implementation pressure conflicts with the specification, report the conflict and propose the smallest safe path.

Never claim a file, test, build, benchmark, or integration exists unless it was actually created or executed.
