# ADR-0023 — Explicit Versioned Project Schema

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Serializing internal Rust structs directly would couple project compatibility to implementation details and make migrations unreliable.

---

## Decision

Mirae will define an explicit, versioned, schema-validated project format separate from runtime implementation types.

---

## Consequences

### Positive

- durable compatibility;
- deterministic migrations;
- reviewable project files;
- safer validation;
- separation from runtime handles.

### Negative

- mapping code;
- schema maintenance;
- migration fixtures;
- more explicit release discipline.

---

## Alternatives Considered

### Serialize internal domain/runtime structs directly

Rejected because internal refactors would become file-format changes and runtime-only data could leak into persistence.

---

## Related Specifications

- `04-project/401-project-format.md`
- `04-project/408-schema-versioning-and-migrations.md`
