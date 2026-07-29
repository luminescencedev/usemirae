# 805 — Generated Contracts and Schemas

**Status:** Proposed  
**Audience:** IPC, project, SDK, UI, tooling contributors  
**Canonical:** Yes  
**Required context:** `01-runtime/108-ipc-protocol.md`, `04-project/401-project-format.md`, `07-sdk/704-sdk-api-surface.md`  
**Related ADRs:** ADR-0057

---

## 1. Purpose

Cross-language and persisted contracts are defined once and generated deterministically.

---

## 2. Canonical Schema Domains

- engine IPC;
- project format;
- project bundles;
- diagnostics;
- SDK protocol;
- extension manifest;
- declarative extension UI;
- compatibility/workaround database.

---

## 3. Generation Outputs

A schema may generate:

- Rust types;
- TypeScript types;
- validators;
- JSON schemas;
- protocol fixtures;
- documentation tables;
- compatibility manifests;
- test vectors.

---

## 4. Generation Command

One root command:

```text
cargo xtask generate
```

It must:

1. validate schemas;
2. generate outputs;
3. format outputs;
4. generate fixtures;
5. verify no duplicate IDs;
6. write deterministic files;
7. report changed contracts.

---

## 5. CI Check

CI runs generation in clean checkout and fails when generated output differs.

Generated directories are either:

- committed and verified; or
- fully built during packaging.

The choice must be consistent per output class.

---

## 6. Contract Change Process

A contract change requires:

- classification as compatible/breaking;
- version change when needed;
- migration or compatibility fixture;
- generated output;
- tests;
- docs;
- ADR if architecture semantics change.

---

## 7. Handwritten Mapping

Internal types map explicitly to generated DTOs.

Do not make internal runtime structures public by deriving serializers merely for convenience.

---

## 8. Invariants

1. One canonical schema per contract.
2. Generation is deterministic.
3. Generated files are not edited manually.
4. CI checks drift.
5. Breaking changes are versioned.
6. Migrations/fixtures accompany persisted changes.
7. Internal and public types remain separate.
8. IDs/enums are stable.
9. Bounds are represented in schema.
10. Documentation is generated or synchronized.
