# 817 — Documentation Maintenance

**Status:** Proposed  
**Audience:** All contributors and coding agents  
**Canonical:** Yes  
**Required context:** `00-foundations/000-documentation-contract.md`

---

## 1. Purpose

This document keeps the documentation usable as implementation begins.

---

## 2. Document Status

Use:

- Draft;
- Proposed;
- Accepted;
- Deprecated;
- Superseded.

A document should not remain `Proposed` forever after implementation stabilizes.

---

## 3. Change Triggers

Documentation must change when:

- public behavior changes;
- schema changes;
- lifecycle changes;
- dependency direction changes;
- new failure mode appears;
- performance budget changes;
- permission/capability changes;
- platform support changes;
- ticket workflow changes.

---

## 4. ADR Rules

Create an ADR when:

- choosing a long-lived technology;
- changing process boundaries;
- changing persisted semantics;
- changing public compatibility;
- introducing a security trust decision;
- accepting a significant trade-off.

Do not create ADRs for ordinary implementation detail.

---

## 5. Link and Structure Checks

CI validates:

- SUMMARY links;
- required metadata;
- duplicate document IDs;
- ADR references;
- nonexistent paths;
- stale planned-section markers;
- generated schema docs.

---

## 6. Implementation Status

Each subsystem may maintain a separate implementation-status document or machine-readable matrix.

Do not rewrite architecture specs into changelogs.

Status should point to tickets/PRs.

---

## 7. AI Agent Rules

An agent:

- reads docs before code change;
- updates docs in same PR when contract changes;
- does not silently reinterpret `Proposed` text;
- calls out ambiguity;
- preserves canonical terminology;
- reports exact docs changed.

---

## 8. Invariants

1. Canonical docs and code do not knowingly diverge.
2. ADRs capture decisions, not daily work.
3. Status and architecture remain separate.
4. Links are validated.
5. Deprecated docs point to replacements.
6. Generated docs are reproducible.
7. Terminology stays consistent.
8. Contract changes update docs in same PR.
9. AI changes are traceable.
10. Documentation remains navigable.
