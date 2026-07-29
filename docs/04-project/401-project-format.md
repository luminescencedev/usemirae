# 401 — Project Format

**Status:** Proposed  
**Audience:** Project, persistence, migration, SDK contributors  
**Canonical:** Yes  
**Required context:** `400-project-overview.md`, `00-foundations/005-domain-model.md`  
**Related ADRs:** ADR-0023, ADR-0028, ADR-0030

---

## 1. Purpose

This document defines the canonical serialized project representation and envelope.

---

## 2. Representation

The canonical editable project format is a versioned, schema-validated document.

The initial human-readable representation SHOULD use JSON or another reviewable structured encoding selected by implementation ADR.

The semantic schema is independent of the exact parser library.

---

## 3. Project Envelope

Conceptual structure:

```json
{
  "format": "mirae-project",
  "schemaVersion": 1,
  "projectId": "uuid",
  "createdAt": "RFC3339 timestamp",
  "lastSavedAt": "RFC3339 timestamp",
  "app": {
    "minimumVersion": "0.1.0",
    "savedByVersion": "0.1.0"
  },
  "features": [],
  "integrity": {
    "algorithm": "sha256",
    "contentHash": "..."
  },
  "project": {}
}
```

Exact fields may evolve before acceptance, but the envelope responsibilities are mandatory.

---

## 4. Schema Rules

The schema MUST define:

- required fields;
- defaults;
- enum representations;
- numeric bounds;
- string length bounds;
- stable IDs;
- URI/path representations;
- feature-gated fields;
- unknown-field behavior;
- extension namespaces;
- migration source version.

---

## 5. Stable IDs

All persisted entities use stable opaque IDs.

IDs:

- survive reorder;
- survive save;
- are unique in required namespace;
- do not encode vector position;
- do not encode OS path;
- do not encode runtime generation.

---

## 6. Numeric Values

Rules:

- NaN and infinity are rejected;
- normalized values define range;
- dimensions and durations define bounds;
- transforms use documented units;
- rational rates use numerator/denominator;
- timestamps use explicit representation;
- no unit is implied by field name alone when ambiguity exists.

---

## 7. Paths and URIs

Project references use explicit URI forms:

- managed project asset;
- project-relative file;
- absolute local file;
- file-library reference;
- generated resource;
- extension resource.

OS-specific raw paths are normalized at adapter boundaries.

---

## 8. Secrets

The schema stores:

- credential reference ID;
- provider kind;
- safe account label if needed.

It MUST NOT store:

- passwords;
- stream keys;
- access tokens;
- refresh tokens;
- private signing keys;
- session cookies.

---

## 9. Unknown Fields

Unknown-field policy is version- and namespace-aware.

Core unknown fields:

- may be preserved only when parser and serializer can do so safely;
- otherwise trigger compatibility diagnostic;
- must not be silently ignored if they affect declared required feature.

Extension namespaced fields may be preserved opaquely within size and security bounds.

---

## 10. Required Features

A project may declare required features.

On open:

- unsupported optional feature → degrade with diagnostic;
- unsupported required feature → reject or open read-only/repair mode;
- unknown extension feature → preserve configuration when safe.

---

## 11. Integrity

Integrity metadata detects accidental corruption, not adversarial authenticity unless a signature system is added.

Hash computation excludes the hash field itself through canonical serialization rules.

Canonicalization must be deterministic.

---

## 12. Canonical Serialization

Canonical serialization defines:

- field order where hash requires it;
- normalized Unicode policy;
- number formatting;
- newline format;
- map ordering;
- omission/default rules.

Ordinary save should produce stable diffs.

---

## 13. Extension Namespaces

Extension project data is stored under stable extension IDs.

Requirements:

- manifest-declared schema version;
- size limits;
- migration ownership;
- no executable code;
- no unrestricted file paths;
- preserved when extension absent where safe.

---

## 14. Invariants

1. Format identifies itself.
2. Schema version is explicit.
3. Project ID is stable.
4. Secrets are excluded.
5. Numeric units and bounds are defined.
6. Entity identity is position-independent.
7. Required unsupported features are not silently ignored.
8. Canonical serialization is deterministic.
9. Extension data is namespaced and bounded.
10. Runtime objects are impossible to represent directly.

---

## 15. Required Tests

- schema validation;
- stable serialization;
- hash verification;
- unknown core field;
- unknown optional feature;
- unsupported required feature;
- extension data preservation;
- secret scanning;
- invalid float;
- path normalization;
- large-field bounds;
- deterministic fixtures.

---

## 16. AI Implementation Notes

Do not derive the public project schema directly from internal runtime structs.

Do not serialize toolkit objects or arbitrary maps.

Do not store absolute platform paths without an explicit URI kind.

Keep canonical serialization independent from incidental hash-map order.
