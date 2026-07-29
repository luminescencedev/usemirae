# 411 — Project Validation and Repair

**Status:** Proposed  
**Audience:** Project, migration, support tooling contributors  
**Canonical:** Yes  
**Required context:** `401-project-format.md`, `408-schema-versioning-and-migrations.md`  
**Related ADRs:** ADR-0023, ADR-0028

---

## 1. Purpose

This document defines layered validation and safe repair of damaged, incomplete, or incompatible projects.

---

## 2. Validation Layers

### 2.1 Envelope validation

- format marker;
- schema version;
- bounds;
- integrity metadata;
- minimum app version.

### 2.2 Syntax/schema validation

- required fields;
- types;
- enums;
- numeric bounds;
- string/collection limits.

### 2.3 Reference validation

- entity IDs;
- source references;
- scene item parents;
- asset references;
- output references;
- extension namespaces.

### 2.4 Semantic validation

- acyclic scene graph;
- valid transforms;
- valid routing;
- compatible output profile;
- supported feature semantics;
- no secret fields.

### 2.5 Runtime capability validation

- device availability;
- encoder support;
- permissions;
- assets;
- extensions;
- output endpoints.

Runtime capability failure does not necessarily make the project invalid.

---

## 3. Validation Result

A result contains:

- severity;
- stable issue code;
- project path/entity path;
- safe message;
- repairability;
- suggested action;
- affected feature;
- whether activation may continue;
- correlation ID.

---

## 4. Repair Principles

Repair MUST:

- preserve original;
- be deterministic;
- be previewable;
- record every change;
- avoid deleting unknown user data when preservation is safe;
- never invent credentials;
- avoid changing external files;
- produce new project or explicit repaired copy by default.

---

## 5. Repair Categories

Safe automated repair may include:

- rebuild derived index;
- normalize invalid but unambiguous path form;
- restore missing default;
- remove duplicate derived cache entry;
- regenerate canonical ordering metadata;
- quarantine invalid optional extension block.

User-confirmed repair may include:

- relink asset;
- replace missing source;
- remove invalid effect;
- choose device;
- drop unsupported required feature;
- rebuild scene hierarchy.

---

## 6. Unrepairable Conditions

Examples:

- unreadable/truncated root with no recovery;
- unknown required schema semantics;
- cryptographic integrity failure with no safe parse;
- contradictory identity graph;
- malicious bounds violations;
- missing required extension data that cannot be preserved safely.

Unrepairable does not imply deletion.

---

## 7. Repair Report

Report includes:

- original identity/hash;
- repaired identity/hash;
- tool version;
- issue list;
- actions;
- discarded data;
- preserved unknown data;
- warnings;
- timestamp;
- operator decision.

---

## 8. Support Bundle

A support bundle may include:

- redacted project structure;
- validation report;
- schema versions;
- extension requirements;
- asset availability summary;
- diagnostics.

It excludes:

- credentials;
- raw media unless explicitly selected;
- full private paths by default;
- unrelated project content.

---

## 9. Invariants

1. Validation is layered.
2. Runtime unavailability is not schema corruption.
3. Repair preserves original.
4. Repair actions are logged.
5. Unknown data is preserved when safe.
6. Credentials are never invented or exported.
7. Automatic repair is limited to unambiguous changes.
8. Repair output validates again.
9. Support bundles are redacted.
10. Unrepairable projects are never deleted automatically.

---

## 10. Required Tests

- missing optional field;
- duplicate ID;
- cycle;
- missing asset;
- missing device;
- unknown required feature;
- extension absent;
- automatic repair preview;
- repaired-copy validation;
- original preservation;
- redacted support bundle;
- malicious oversized input.

---

## 11. AI Implementation Notes

Do not conflate unavailable runtime resources with corrupt project schema.

Do not repair in place by default.

Do not silently discard unknown extension data.

Every repair action must be represented in a report.
