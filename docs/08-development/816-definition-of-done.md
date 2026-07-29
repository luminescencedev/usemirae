# 816 — Definition of Done

**Status:** Proposed  
**Audience:** All contributors and coding agents  
**Canonical:** Yes  
**Required context:** `809-testing-and-validation-workflow.md`, `812-ticket-and-sprint-workflow.md`

---

## 1. Ticket Completion

A ticket is done only when:

- acceptance criteria are satisfied;
- code is merged;
- required tests pass;
- required diagnostics exist;
- docs are updated;
- generated contracts are synchronized;
- no unresolved mandatory review;
- tracker is updated;
- follow-ups are separate tickets.

---

## 2. Code Quality

- dependency direction preserved;
- no dead temporary implementation;
- no hidden unbounded resource;
- no recoverable panic;
- no secret/log privacy issue;
- public API documented;
- unsafe code reviewed;
- naming follows terminology.

---

## 3. Testing

- behavior test;
- failure test;
- regression test for bug;
- platform test where relevant;
- accessibility test for user-facing UI;
- compatibility fixture for contract changes;
- benchmark for performance-sensitive changes.

---

## 4. Documentation

Update:

- canonical spec when behavior contract changes;
- ADR when decision changes;
- ticket acceptance notes;
- public API docs;
- migration docs;
- troubleshooting if operational behavior changed.

---

## 5. Pull Request

PR:

- links ticket;
- explains change;
- lists exact validation;
- states risks;
- has no unrelated files;
- passes mandatory CI;
- is approved according to risk;
- is squash-merged;
- deletes branch.

---

## 6. Not Done

A ticket is not done when:

- it works only in a manual happy path;
- tests are skipped without accepted exception;
- generated files are stale;
- docs contradict code;
- failure/recovery is undefined;
- branch is unmerged;
- TODO hides acceptance criterion;
- UI is inaccessible;
- performance regression is unexplained;
- secrets or private data leak.

---

## 7. Invariants

1. “Implemented” is not the same as “done.”
2. Merge is required.
3. Validation is required.
4. Documentation remains synchronized.
5. Follow-up work is explicit.
6. Risk-specific quality gates apply.
7. Accessibility and security are not optional.
8. Failure behavior is part of behavior.
9. Generated contracts stay clean.
10. Tracker reflects reality.
