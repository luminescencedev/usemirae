# ADR-0027 — Content-Addressed Managed Assets

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Projects need portable managed assets, integrity checks, deduplication, and stable references independent from user file paths.

---

## Decision

Mirae-managed asset blobs will be stored by content hash where practical, while projects refer to stable logical asset IDs.

---

## Consequences

### Positive

- deduplication;
- integrity;
- stable identity;
- portability;
- safer recovery.

### Negative

- hash computation cost;
- reachability and GC complexity;
- external mutable files still require separate policy.

---

## Alternatives Considered

### Use original path as identity

Rejected because paths move, differ by platform, and do not provide integrity.

### Copy assets by original filename only

Rejected because collisions and duplicate storage are likely.

---

## Related Specifications

- `04-project/406-asset-registry.md`
- `04-project/410-project-portability-and-bundles.md`
