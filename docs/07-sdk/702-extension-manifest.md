# 702 — Extension Manifest

**Status:** Proposed  
**Audience:** SDK, package, security, distribution contributors  
**Canonical:** Yes  
**Required context:** `700-sdk-overview.md`, `713-extension-packaging-signing-and-distribution.md`  
**Related ADRs:** ADR-0049, ADR-0053

---

## 1. Purpose

The manifest declares extension identity, compatibility, components, capabilities requested, project-data schemas, resource limits, UI contributions, and distribution metadata.

A manifest is declarative. It does not grant permissions.

---

## 2. Conceptual Manifest

```json
{
  "manifestVersion": 1,
  "id": "com.example.extension",
  "name": "Example Extension",
  "version": "1.2.0",
  "publisher": {
    "id": "publisher-id",
    "name": "Example"
  },
  "sdk": {
    "minimum": "1.0",
    "maximumExclusive": "2.0"
  },
  "entrypoints": [],
  "capabilities": {
    "required": [],
    "optional": []
  },
  "projectData": [],
  "ui": [],
  "resources": {},
  "platforms": [],
  "signing": {}
}
```

Exact encoding may evolve, but responsibilities remain.

---

## 3. Identity

Extension ID:

- is globally stable;
- uses reverse-domain or approved namespace;
- is controlled by publisher identity;
- cannot be changed during ordinary update;
- scopes storage, permissions, logs, and project data.

Name is display metadata, not identity.

---

## 4. Entrypoints

Entrypoint types may include:

- runtime;
- source provider;
- output provider;
- effect provider;
- importer/exporter;
- automation;
- UI contribution;
- project migrator;
- diagnostics provider.

Each entrypoint declares runtime environment and supported platforms.

---

## 5. Capabilities

Capabilities are listed as:

- required;
- optional;
- conditional by entrypoint.

Examples:

- project read selected entities;
- project write extension namespace;
- network to declared domains;
- file picker import;
- extension storage;
- credential use for declared service;
- media source frames;
- output sink;
- UI panel;
- notifications.

The host grants them separately.

---

## 6. Resource Declaration

Manifest declares requested maxima or classes for:

- memory;
- CPU;
- threads/tasks;
- storage;
- network;
- message rate;
- logs;
- media queue;
- GPU/effect cost;
- UI contribution count.

Requested values may be reduced by host policy.

---

## 7. Project Data

Each project-data namespace declares:

- namespace ID;
- schema version;
- maximum size;
- required/optional;
- migration support;
- behavior when extension absent;
- whether data is portable in bundles.

---

## 8. UI Contributions

Manifest declares allowed contribution points:

- settings page;
- source configuration panel;
- output configuration panel;
- dock/panel;
- command palette action;
- context action;
- status indicator.

Actual content is validated at runtime.

---

## 9. Platform Support

Declare:

- OS families;
- architecture;
- minimum versions;
- package-mode requirements;
- optional native worker;
- unavailable features.

The host still validates runtime capabilities.

---

## 10. Localization

Manifest may reference localized:

- name;
- description;
- permission explanations;
- settings labels;
- command labels;
- errors safe for UI.

Stable internal IDs remain locale-independent.

---

## 11. Validation

Validation includes:

- schema version;
- field bounds;
- ID format;
- semantic version;
- entrypoint existence;
- duplicate IDs;
- capability validity;
- platform declarations;
- resource bounds;
- signature coverage;
- path normalization;
- archive-entry safety.

---

## 12. Invariants

1. Manifest is declarative.
2. Extension ID is stable.
3. Capability request is not grant.
4. Every component is declared.
5. Resource requests are bounded.
6. Project data is namespaced and versioned.
7. UI contribution points are explicit.
8. Platform support is declared and probed.
9. Signature covers canonical package manifest.
10. Unknown required manifest features reject safely.

---

## 13. Required Tests

- valid manifest;
- invalid ID;
- unsupported manifest version;
- duplicate entrypoint;
- undeclared file;
- excessive resource request;
- unknown capability;
- signature mismatch;
- localized metadata;
- platform mismatch;
- project-data namespace collision;
- path traversal.

---

## 14. AI Implementation Notes

Do not treat the manifest as executable configuration.

Do not grant permissions because they appear under `required`.

Do not use display names as stable IDs.

Validate paths, sizes, counts, and unknown required features before loading code.
