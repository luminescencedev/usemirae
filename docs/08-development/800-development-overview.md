# 800 — Development Overview

**Status:** Proposed  
**Audience:** All contributors and coding agents  
**Canonical:** Yes  
**Required context:** All architecture sections, especially `00-foundations/007-ai-implementation-contract.md`  
**Related ADRs:** ADR-0055 through ADR-0060

---

## 1. Purpose

This section converts the architecture into an executable development system.

It defines:

- repository layout;
- crate and package boundaries;
- allowed dependencies;
- code-generation rules;
- toolchains;
- local workflows;
- tests and validation;
- CI/CD;
- Git and PR conventions;
- ticket structure;
- implementation order;
- AI-agent behavior.

---

## 2. Development Principles

1. Build in vertical slices.
2. Preserve dependency direction.
3. Keep contracts generated and versioned.
4. Keep every branch small and reviewable.
5. Make every ticket independently verifiable.
6. Never bypass quality gates to make a demo work.
7. Prefer one authoritative implementation over temporary duplicates.
8. Add observability with the feature.
9. Add tests with the feature.
10. Update canonical docs when contracts change.

---

## 3. Source of Truth

Priority order:

1. accepted ADR;
2. canonical subsystem specification;
3. public schema or generated contract;
4. implementation;
5. tests;
6. issue/ticket description;
7. comments.

If implementation and accepted documentation conflict, the conflict must be resolved explicitly.

---

## 4. Delivery Unit

The normal delivery unit is one ticket that produces:

- one coherent behavior;
- one small branch;
- implementation;
- tests;
- diagnostics;
- documentation update where required;
- local validation evidence;
- one pull request;
- squash merge.

---

## 5. Initial Technology Direction

The documented architecture assumes:

- Rust native engine and platform layers;
- `wgpu` renderer;
- FFmpeg behind contained adapters;
- React + TypeScript control UI;
- native desktop shell with embedded local web UI;
- Cargo workspace;
- pnpm workspace;
- generated IPC/SDK schemas;
- GitHub Actions or equivalent CI;
- short-lived feature branches.

Specific libraries may change only when they preserve the documented boundary.

---

## 6. Repository Health

The repository must keep:

- clean root commands;
- pinned toolchains;
- deterministic generation;
- no committed secrets;
- no undocumented binary blobs;
- no circular package dependencies;
- no platform checks scattered in domain code;
- no temporary “legacy/new” architecture without migration plan;
- no generated-code edits by hand.

---

## 7. AI Agent Contract

An AI agent must:

1. read `CLAUDE.md`;
2. read the ticket;
3. read linked canonical docs;
4. inspect existing implementation;
5. identify affected contracts;
6. implement the smallest compliant slice;
7. add tests;
8. run required validation;
9. report exact changes and unresolved gaps;
10. never claim success without command evidence.

---

## 8. Global Invariants

1. One repository contains all first-party code and schemas.
2. Engine/domain code does not depend on UI.
3. Platform code remains behind interfaces.
4. Generated contracts are never edited manually.
5. Every ticket names acceptance criteria.
6. Every PR is independently buildable.
7. Every architecture change updates docs or ADRs.
8. Every public contract has compatibility tests.
9. Every new queue/cache/resource has a bound.
10. Every completed ticket leaves the repository cleaner or equally clean.

---

## 9. Required Validation

At minimum before merge:

- formatting;
- lint/static analysis;
- affected unit tests;
- affected integration tests;
- schema generation consistency;
- documentation-link validation;
- secret scan;
- build of changed targets.

High-risk tickets require additional gates from the quality section.
