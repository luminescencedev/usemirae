# 616 — Release Quality Gates

**Status:** Proposed  
**Audience:** Release, QA, architecture, security contributors  
**Canonical:** Yes  
**Required context:** All `06-quality` documents  
**Related ADRs:** ADR-0039, ADR-0042, ADR-0043, ADR-0044, ADR-0045

---

## 1. Purpose

This document defines minimum evidence required before a stable, beta, or nightly release can be published.

---

## 2. Gate Categories

- build integrity;
- automated tests;
- project compatibility;
- performance;
- memory and soak;
- security;
- accessibility;
- platform matrix;
- packaging/signing/update;
- documentation;
- known issues and rollback.

---

## 3. Stable Release Gates

A stable release requires:

- reproducible/traceable signed builds;
- all mandatory tests passing;
- migration corpus passing;
- no unresolved critical security issue;
- no known project corruption bug;
- reference benchmarks within thresholds or approved exception;
- memory soak without unbounded growth;
- crash-loop and recovery tests;
- keyboard/accessibility smoke pass;
- package/install/update/rollback verification;
- support matrix review;
- release notes and known limitations;
- rollback plan.

---

## 4. Beta Gates

Beta may permit:

- documented performance variance;
- experimental feature flags;
- limited platform coverage;
- known non-critical issues.

It still requires:

- no known data-loss issue;
- signed packages;
- core migrations;
- security baseline;
- rollback;
- clear experimental labeling.

---

## 5. Nightly Gates

Nightly requires:

- build;
- basic static analysis;
- unit/component suite;
- schema compatibility smoke;
- package integrity;
- automatic known-bad build suppression.

Nightly may not be safe for production.

---

## 6. Exceptions

A gate exception requires:

- owner;
- exact failed gate;
- user impact;
- scope;
- mitigation;
- expiration;
- approval;
- release-note disclosure when relevant.

“Deadline” alone is not sufficient justification for data-integrity or security exceptions.

---

## 7. Release Candidate Soak

Release candidate runs:

- long production workload;
- stream plus recording;
- device reconnect;
- UI restart;
- extension host restart;
- autosave;
- suspend/resume;
- memory monitoring;
- output reconnect.

---

## 8. Rollback Readiness

Before release:

- previous installer available;
- update metadata can halt rollout;
- incompatible project-save warning reviewed;
- crash-loop safe mode verified;
- signing keys and revocation process ready;
- support guidance prepared.

---

## 9. Invariants

1. Stable release cannot bypass data-integrity gates casually.
2. Exceptions are recorded and expire.
3. Signed package verification is mandatory.
4. Project migration corpus passes.
5. Accessibility is part of release review.
6. Performance regressions need disposition.
7. Rollback is tested.
8. Known issues are documented.
9. Nightly is clearly non-production.
10. Release evidence is archived.

---

## 10. Required Tests

This document is enforced through release automation and audit artifacts:

- gate status report;
- exception validation;
- signature verification;
- migration corpus;
- benchmark comparison;
- soak report;
- accessibility checklist;
- update rollback;
- release-note completeness;
- artifact retention.

---

## 11. AI Implementation Notes

Do not mark a release ready because the build compiles.

Do not suppress failing quality gates without creating a documented exception.

Do not approve stable release with known data corruption or unsigned artifacts.
