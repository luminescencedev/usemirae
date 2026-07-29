# 815 — AI Build Workflow

**Status:** Proposed  
**Audience:** Claude Code, Codex, and other coding agents  
**Canonical:** Yes  
**Required context:** `CLAUDE.md`, `812-ticket-and-sprint-workflow.md`, `816-definition-of-done.md`

---

## 1. Purpose

This document defines how an AI coding agent turns one ticket into one validated pull request.

---

## 2. Required Input

The agent needs:

- repository;
- current ticket;
- active sprint tracker;
- canonical docs;
- available tools;
- branch/PR permissions.

Minor missing details should be resolved through existing docs and code rather than unnecessary questions.

---

## 3. Start Procedure

1. read `CLAUDE.md`;
2. read ticket;
3. read every linked canonical document;
4. inspect related crates/packages/tests;
5. check git status;
6. identify dependency and contract impact;
7. write a short implementation checklist;
8. create ticket branch.

---

## 4. Implementation Procedure

The agent must:

- implement the smallest compliant change;
- preserve dependency direction;
- use generated contracts;
- add bounds and diagnostics;
- avoid unrelated refactors;
- add tests as behavior is added;
- update docs if contract changes;
- create ADR only when the decision is genuinely architectural.

---

## 5. Validation Procedure

Run:

1. formatting;
2. lint/typecheck;
3. generated-contract check;
4. affected unit tests;
5. affected integration tests;
6. documentation check;
7. risk-specific checks;
8. full check if foundation/contracts changed.

Capture command results.

---

## 6. Completion Report

The agent reports:

- ticket;
- files changed;
- behavior added;
- tests added;
- commands run;
- results;
- assumptions;
- unresolved risks;
- follow-up tickets;
- PR link if created.

Do not say “done” when required validation failed.

---

## 7. Failure Handling

If blocked:

- preserve working tree;
- explain exact blocker;
- provide current evidence;
- do not invent unavailable API behavior;
- do not weaken architecture to force progress;
- create or propose a focused unblock ticket.

---

## 8. Scope Rules

The agent must not:

- redesign unrelated architecture;
- upgrade dependencies without need;
- change public schemas casually;
- add unbounded queues/caches;
- silence lints globally;
- remove tests to pass CI;
- store secrets;
- commit generated build artifacts;
- merge without authorization.

---

## 9. “Next Ticket” Behavior

When instructed `next ticket`:

1. choose first ready ticket;
2. mark in progress;
3. execute workflow;
4. create PR if publishing is enabled;
5. update tracker;
6. stop after one ticket unless instructed otherwise.

---

## 10. Invariants

1. One agent run targets one ticket.
2. Canonical docs are read first.
3. Scope remains bounded.
4. Tests and diagnostics accompany code.
5. Validation evidence is explicit.
6. Architecture is not bypassed.
7. Failed checks are reported honestly.
8. Tracker is updated.
9. PR is reviewable.
10. Merge remains an explicit decision.
