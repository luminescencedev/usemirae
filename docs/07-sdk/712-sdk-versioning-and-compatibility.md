# 712 — SDK Versioning and Compatibility

**Status:** Proposed  
**Audience:** SDK, architecture, release, extension authors  
**Canonical:** Yes  
**Required context:** `06-quality/615-compatibility-policy.md`, `704-sdk-api-surface.md`  
**Related ADRs:** ADR-0049, ADR-0050

---

## 1. Purpose

This document defines SDK versioning, compatibility negotiation, deprecation, experimental APIs, project-data schemas, and host rejection behavior.

---

## 2. Version Domains

Separate versions:

- manifest schema;
- SDK protocol;
- public API modules;
- extension project-data schema;
- package format;
- declarative UI schema;
- media data-plane protocol.

One app version does not replace these versions.

---

## 3. Compatibility Range

Manifest declares:

- minimum SDK;
- maximum-exclusive SDK;
- required feature flags;
- optional features.

Host resolves the highest compatible protocol/API combination.

---

## 4. Semantic Compatibility

Rules:

- adding optional field is compatible;
- adding optional endpoint behind feature is compatible;
- changing field meaning is breaking;
- removing/renaming required field is breaking;
- changing timing/ownership semantics is breaking;
- tightening security bounds may be compatible but requires diagnostics;
- changing project-data schema requires migration.

---

## 5. Deprecation

A deprecated API includes:

- replacement;
- introduced deprecation version;
- earliest removal version;
- runtime warning rate limit;
- migration guide;
- test coverage.

Stable API removal requires major version or compatibility bridge.

---

## 6. Experimental APIs

Experimental APIs:

- require explicit manifest opt-in;
- may change within declared channel;
- are unavailable in stable policy unless enabled;
- cannot be required by portable stable project without warning;
- have separate documentation.

---

## 7. Compatibility Shims

Host may provide shims for:

- renamed fields;
- old event variants;
- old command shapes;
- legacy UI schema;
- bounded old SDK versions.

Shims are versioned, tested, observable, and eventually removed through policy.

---

## 8. Project Data Compatibility

Extension owns migrations for its namespace.

Host ensures:

- migration runs in isolated extension environment or declarative migrator;
- original project remains unchanged until save;
- migration failure preserves old data;
- missing extension preserves opaque data where safe.

---

## 9. Native ABI

No stable native ABI is promised for third-party extensions.

If native workers exist, they communicate through versioned process protocols.

---

## 10. Invariants

1. Version domains are separate.
2. Compatibility range is explicit.
3. Semantic changes are classified.
4. Deprecation precedes stable removal.
5. Experimental APIs are opt-in.
6. Shims are tested and observable.
7. Project data has independent migrations.
8. Native ABI is not the public compatibility boundary.
9. Incompatible extensions fail closed.
10. Compatibility fixtures are maintained.

---

## 11. Required Tests

- minimum SDK;
- maximum-exclusive SDK;
- optional field;
- breaking field change;
- deprecated endpoint;
- compatibility shim;
- experimental feature;
- project-data migration;
- missing migrator;
- manifest-version mismatch;
- media protocol mismatch;
- native worker protocol mismatch.

---

## 12. AI Implementation Notes

Do not tie extension compatibility only to the Mirae application version.

Do not change field semantics without a version decision.

Do not promise a stable in-process native ABI.

Add compatibility fixtures for every public contract change.
