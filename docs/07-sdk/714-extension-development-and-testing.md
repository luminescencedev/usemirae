# 714 — Extension Development and Testing

**Status:** Proposed  
**Audience:** Extension authors, SDK tooling, QA contributors  
**Canonical:** Yes  
**Required context:** `06-quality/609-testing-strategy.md`, `712-sdk-versioning-and-compatibility.md`  
**Related ADRs:** ADR-0042, ADR-0049

---

## 1. Purpose

This document defines the developer kit, local workflow, test harness, fixtures, validation, debugging, and publication checks for extensions.

---

## 2. SDK Package

The SDK should provide:

- generated bindings;
- manifest schema;
- project-data schema helpers;
- capability constants;
- test host;
- mock/fake engine services;
- media fixtures;
- UI component schema;
- validation CLI;
- packaging/signing CLI;
- compatibility checker;
- documentation examples.

---

## 3. Developer Mode

Developer mode:

- is explicit;
- visually marks development extensions;
- allows local package/directory loading;
- does not bypass capability grants;
- does not disable message/size bounds;
- may enable richer logs/debugging;
- cannot be enabled silently by package content.

---

## 4. Test Host

The test host simulates:

- lifecycle;
- capability grants/denials;
- project projections;
- commands/events;
- storage;
- file/network broker;
- credential metadata;
- source/output queues;
- UI rendering;
- host crash/restart.

Tests use deterministic clocks and IDs.

---

## 5. Contract Tests

Required extension contract tests may include:

- manifest validation;
- SDK compatibility;
- lifecycle start/stop;
- permission denial;
- data migration;
- quota handling;
- event gap;
- host restart;
- source/output fallback;
- UI accessibility;
- localization;
- uninstall/reinstall.

---

## 6. Media Fixtures

Provide bounded synthetic:

- video frames;
- audio blocks;
- encoded packets;
- discontinuities;
- color metadata;
- network failures;
- device reconnects.

Real user media is not required.

---

## 7. Debugging

Development tools may include:

- extension logs;
- SDK traces;
- permission inspector;
- state projection viewer;
- event stream viewer;
- quota dashboard;
- media queue metrics;
- UI contribution preview;
- manifest diagnostics.

Debug tools must not expose secrets.

---

## 8. Validation CLI

Commands may validate:

- package;
- manifest;
- schemas;
- signatures;
- localization;
- API compatibility;
- UI accessibility schema;
- resource declarations;
- forbidden files;
- licenses/notices.

---

## 9. Publication Checklist

- tests pass;
- compatibility range valid;
- manifest valid;
- package reproducible/traceable;
- signature valid;
- permissions justified;
- privacy disclosure complete;
- resource limits realistic;
- migrations tested;
- UI accessible/localized;
- rollback behavior tested.

---

## 10. Invariants

1. Developer mode does not bypass permissions.
2. Test host is deterministic.
3. Contract tests cover denial and failure.
4. Debug tools redact secrets.
5. Validation is automated.
6. Media fixtures are synthetic/bounded.
7. Compatibility is checked before publication.
8. Accessibility is tested.
9. Migrations have fixtures.
10. Package signing occurs after final content hash.

---

## 11. Required Tests

- SDK example build;
- test-host lifecycle;
- permission-denied fixture;
- quota fixture;
- media discontinuity;
- UI preview;
- localization validation;
- package CLI;
- compatibility checker;
- signing flow;
- publication checklist;
- developer-mode boundary.

---

## 12. AI Implementation Notes

Do not build examples that require unrestricted host access.

Do not make developer mode equivalent to full trust.

Do not publish without denial, timeout, quota, and migration tests.

Keep sample extensions aligned with current SDK schemas.
