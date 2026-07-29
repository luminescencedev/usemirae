# 410 — Project Portability and Bundles

**Status:** Proposed  
**Audience:** Project, asset, security, UI contributors  
**Canonical:** Yes  
**Required context:** `401-project-format.md`, `406-asset-registry.md`  
**Related ADRs:** ADR-0030

---

## 1. Purpose

This document defines exportable project bundles for backup, transfer, support, templates, and collaboration.

---

## 2. Bundle Goals

A bundle should:

- contain project schema;
- include selected managed/external assets;
- preserve stable IDs;
- include integrity manifest;
- avoid secrets;
- be extractable safely;
- describe missing/excluded dependencies;
- remain inspectable without cloud service.

---

## 3. Bundle Contents

Conceptual layout:

```text
project.mirae-bundle/
├── manifest.json
├── project/
│   └── project.json
├── assets/
│   └── sha256/
├── fonts/
├── extensions/
│   └── requirements.json
├── reports/
│   └── portability.json
└── signatures/
```

The transport container may be an archive, directory, or package format.

---

## 4. Manifest

Manifest includes:

- bundle format version;
- project ID;
- project schema version;
- creation metadata;
- content entries;
- hashes;
- byte sizes;
- required features;
- extension requirements;
- excluded items;
- encryption/signature metadata when supported.

---

## 5. Asset Inclusion Policies

For each asset:

- include managed copy;
- include external copy;
- reference only;
- exclude due to licensing;
- exclude due to size;
- exclude due to privacy;
- substitute proxy;
- require user decision.

The export report lists every decision.

---

## 6. Secrets

Bundles MUST NOT include ordinary credentials.

Possible future secure credential transfer requires a separate encrypted export design and explicit user action.

Credential references in project data become unresolved on another machine until reconnected.

---

## 7. Extension Requirements

Bundle records:

- extension ID;
- required/optional;
- minimum API/version;
- data schema version;
- source or installation hint;
- whether project opens without it.

Extension binaries are not embedded by default.

---

## 8. Import

Import flow:

1. validate archive bounds;
2. prevent path traversal;
3. validate manifest;
4. verify hashes;
5. inspect schema and features;
6. scan extension requirements;
7. choose destination;
8. deduplicate managed assets;
9. generate identity according to import mode;
10. activate only after full validation.

---

## 9. Import Modes

- restore same project identity;
- clone as new project;
- instantiate as template;
- inspect read-only;
- extract assets only.

Clone mode regenerates project identity and any identity-scoped metadata.

---

## 10. Integrity and Authenticity

Hashes detect corruption.

Digital signatures may establish publisher authenticity later.

Unsigned bundles are not treated as trusted; their content remains untrusted input.

---

## 11. Size and Resource Bounds

Bundle importer limits:

- total uncompressed bytes;
- per-entry bytes;
- entry count;
- path length;
- nesting;
- compression ratio;
- manifest size.

Zip bombs and path traversal are rejected.

---

## 12. Invariants

1. Bundles exclude secrets.
2. Every included entry is hashed.
3. Import is bounded.
4. Archive paths cannot escape destination.
5. Clone mode regenerates project identity.
6. Missing extensions are reported.
7. Asset inclusion decisions are explicit.
8. Unsigned bundles remain untrusted.
9. Import does not activate partial project.
10. Original bundle remains unchanged.

---

## 13. Required Tests

- full export/import;
- clone identity;
- excluded asset report;
- missing extension;
- corrupt hash;
- path traversal;
- compression bomb;
- duplicate managed asset;
- unsigned bundle;
- unsupported feature;
- interrupted import cleanup;
- template mode.

---

## 14. AI Implementation Notes

Do not put stream keys or tokens in bundles.

Do not extract archive entries before validating normalized paths and bounds.

Do not reuse project ID in clone mode.

Keep bundle format version separate from project schema version.
