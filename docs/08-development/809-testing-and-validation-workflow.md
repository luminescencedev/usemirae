# 809 — Testing and Validation Workflow

**Status:** Proposed  
**Audience:** All contributors and coding agents  
**Canonical:** Yes  
**Required context:** `06-quality/609-testing-strategy.md`, `816-definition-of-done.md`

---

## 1. Purpose

This document maps change types to required local validation.

---

## 2. Baseline Checks

Every code change:

```text
cargo xtask generate --check
cargo xtask fmt --check
cargo xtask lint
cargo xtask test-affected
cargo xtask docs --check
```

Equivalent command names may be implemented through one `cargo xtask check`.

---

## 3. Change Matrix

### Domain/command/state

- unit tests;
- transaction tests;
- generation/patch tests;
- undo tests if applicable.

### Project schema/persistence

- schema fixtures;
- migration fixtures;
- interrupted-write test;
- deterministic serialization;
- recovery test.

### Rendering/media/audio

- component tests;
- performance smoke;
- queue/drop tests;
- fault injection;
- resource lifetime checks.

### Platform/native

- platform test;
- capability diagnostics;
- permission/failure path;
- unsafe review.

### UI

- component tests;
- keyboard tests;
- accessibility checks;
- fake-engine integration;
- reconnect behavior.

### SDK

- schema compatibility;
- permission denial;
- quota;
- host crash;
- package validation.

---

## 4. Bug Fix Rule

Every bug fix should include:

- reproducing test;
- fix;
- regression assertion;
- explanation of root cause.

If automation is impractical, include a documented manual reproduction checklist.

---

## 5. Evidence

PR description lists:

- commands run;
- platforms tested;
- benchmark changes;
- screenshots only where UI-relevant;
- known untested areas;
- risk classification.

---

## 6. Invariants

1. Validation matches change risk.
2. Bug fixes add regression coverage.
3. Generated-contract drift is checked.
4. High-risk changes test failure paths.
5. Manual validation is explicit.
6. Passing tests do not hide untested platforms.
7. Performance-sensitive work includes measurements.
8. Accessibility-sensitive work includes keyboard/semantic checks.
9. Migration work includes historical fixtures.
10. Evidence is attached to PR.
