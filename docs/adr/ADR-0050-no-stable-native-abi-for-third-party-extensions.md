# ADR-0050 — No Stable Native ABI for Third-Party Extensions

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

A stable native ABI would tightly couple third-party extensions to compiler, platform, memory-layout, and engine internals.

It would also encourage in-process execution.

---

## Decision

Mirae will not promise a stable in-process native ABI for third-party extensions.

Native workers, if supported, communicate through versioned process protocols.

---

## Consequences

### Positive

- safer isolation;
- easier internal refactoring;
- language/runtime flexibility;
- explicit compatibility boundary.

### Negative

- IPC overhead;
- native extension authors need adapter/runtime process;
- some integrations are more complex.

---

## Alternatives Considered

### Stable C ABI loaded by engine

Rejected because it creates high compatibility and security cost.

---

## Related Specifications

- `07-sdk/701-extension-architecture.md`
- `07-sdk/712-sdk-versioning-and-compatibility.md`
