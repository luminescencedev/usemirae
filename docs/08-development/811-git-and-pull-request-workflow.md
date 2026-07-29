# 811 — Git and Pull Request Workflow

**Status:** Proposed  
**Audience:** All contributors and coding agents  
**Canonical:** Yes  
**Required context:** `800-development-overview.md`  
**Related ADRs:** ADR-0058

---

## 1. Purpose

This document defines branch, commit, PR, review, and merge behavior.

---

## 2. Branches

Use short-lived branches:

```text
feat/<ticket>-<slug>
fix/<ticket>-<slug>
docs/<ticket>-<slug>
chore/<ticket>-<slug>
```

Branch starts from current main.

Avoid long-lived integration branches.

---

## 3. Commits

Commits should be intentional and buildable where practical.

Before PR, history may be cleaned, but review-relevant steps should not be obscured unnecessarily.

Secrets, generated junk, local config, and binary build outputs are prohibited.

---

## 4. Pull Request

PR contains:

- ticket link;
- problem;
- solution;
- architecture/docs consulted;
- exact files/contracts changed;
- validation commands/results;
- risk;
- screenshots for UI changes;
- known limitations;
- follow-up tickets.

---

## 5. Review

Review checks:

- ticket acceptance;
- architecture consistency;
- dependency direction;
- tests;
- failure/recovery;
- security/privacy;
- accessibility;
- performance;
- docs/ADR impact.

---

## 6. Merge

Default:

```text
gh pr merge <number> --squash --delete-branch
```

Squash commit message should reference ticket and summarize behavior.

Main remains linear and releasable.

---

## 7. Stacked Work

Stacked PRs are allowed only when:

- dependency is explicit;
- each PR is reviewable;
- base is updated after merge;
- final history remains clear.

Do not create a large hidden chain of unfinished architecture.

---

## 8. AI Agent Behavior

The agent may create branch, commit, and PR only when instructed by the active workflow.

It must:

- show changed scope;
- avoid committing unrelated files;
- run validation first;
- never force push shared main;
- never merge without user/reviewer decision unless explicitly authorized.

---

## 9. Invariants

1. Main is protected.
2. Branches are short-lived.
3. PRs map to tickets.
4. Validation evidence is present.
5. Squash merge is default.
6. Branch is deleted after merge.
7. Unrelated changes are excluded.
8. Architecture changes include docs.
9. Secrets never enter history.
10. AI publishing remains scoped and explicit.
