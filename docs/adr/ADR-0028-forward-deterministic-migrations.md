# ADR-0028 — Forward Deterministic Migrations

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Project schema will evolve over years. Migrations must not depend on network state, device availability, or nondeterministic runtime conditions.

---

## Decision

Project migrations will be ordered, forward, deterministic transformations between exact schema versions.

Opening may migrate in memory, while the original file remains unchanged until explicit save.

---

## Consequences

### Positive

- repeatable compatibility;
- safe testing;
- original preservation;
- clear historical fixtures.

### Negative

- migration chain maintenance;
- downgrade not automatic;
- irreversible changes require explicit handling.

---

## Alternatives Considered

### Best-effort parse current structs

Rejected because semantics become ambiguous and historical behavior is untestable.

### Network-assisted migrations

Rejected because projects must open offline and migrations must be reproducible.

---

## Related Specifications

- `04-project/408-schema-versioning-and-migrations.md`
- `04-project/411-project-validation-and-repair.md`
