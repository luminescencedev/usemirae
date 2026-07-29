# 812 — Ticket and Sprint Workflow

**Status:** Proposed  
**Audience:** Project owner, contributors, coding agents  
**Canonical:** Yes  
**Required context:** `811-git-and-pull-request-workflow.md`, `813-implementation-roadmap.md`  
**Related ADRs:** ADR-0059

---

## 1. Purpose

Tickets are the executable planning layer between architecture and code.

---

## 2. Ticket Template

Every implementation ticket includes:

```text
ID:
Title:
Goal:
User/System Value:
Canonical Docs:
Scope:
Out of Scope:
Dependencies:
Implementation Notes:
Acceptance Criteria:
Required Tests:
Required Diagnostics:
Performance/Security/Accessibility Notes:
Validation Commands:
Deliverables:
```

---

## 3. Ticket Size

A normal ticket should fit one small PR.

Split when it combines:

- schema plus multiple subsystem implementations;
- platform support for all OSes;
- UI plus engine plus packaging without a vertical minimum;
- unrelated cleanup;
- several independent behaviors.

---

## 4. Vertical Slice

A preferred ticket delivers a visible or testable path through necessary layers.

Example:

```text
Create empty project
→ command
→ transaction
→ persisted project snapshot
→ UI action
→ integration test
```

Avoid building months of disconnected infrastructure without a running slice.

---

## 5. Ticket Status

- backlog;
- ready;
- in progress;
- review;
- blocked;
- done.

A ticket becomes `ready` only when dependencies and acceptance criteria are clear.

---

## 6. “Next Ticket” Command

When the user says `next ticket` or `ticket suivant`, the coding agent should:

1. open the sprint tracker;
2. select the first ready unblocked ticket;
3. read linked docs;
4. create branch;
5. implement;
6. validate;
7. summarize;
8. create PR if configured;
9. update tracker.

---

## 7. Sprint Tracker

The canonical sprint tracker should include:

- current milestone;
- ordered tickets;
- dependencies;
- status;
- PR;
- validation result;
- deferred follow-ups.

Completed tickets remain recorded.

---

## 8. Invariants

1. One ticket has one clear goal.
2. Acceptance criteria are testable.
3. Out-of-scope is explicit.
4. Tickets link canonical docs.
5. Dependencies are explicit.
6. Done means merged and validated.
7. Follow-ups become separate tickets.
8. Agent does not silently expand scope.
9. Vertical slices are preferred.
10. Tracker is updated after completion.
