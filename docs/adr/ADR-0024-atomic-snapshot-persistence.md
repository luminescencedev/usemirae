# ADR-0024 — Atomic Snapshot Persistence

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Writing project files in place risks partial corruption after crash, disk error, or forced termination.

---

## Decision

Explicit project saves will serialize immutable snapshots to temporary files and publish them through platform-appropriate atomic replacement.

---

## Consequences

### Positive

- previous valid file survives failed save;
- save generation is explicit;
- serialization is isolated from mutable state;
- easier recovery and verification.

### Negative

- additional temporary storage;
- platform-specific durability behavior;
- more complex external-modification checks.

---

## Alternatives Considered

### In-place write

Rejected because partial writes can destroy the only canonical project file.

### Append-only project file as primary format

Rejected initially because compaction, random access, and recovery semantics would add complexity not required for the canonical editable format.

---

## Related Specifications

- `04-project/403-persistence.md`
- `04-project/407-local-file-system.md`
