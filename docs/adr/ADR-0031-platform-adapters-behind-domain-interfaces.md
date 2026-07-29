# ADR-0031 — Platform Adapters Behind Domain Interfaces

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Windows, macOS, and Linux expose different APIs, object models, permission flows, handles, and packaging constraints.

Allowing these types to spread through the engine would fragment the architecture.

---

## Decision

All operating-system integrations will implement Mirae-owned interfaces defined by domain and application requirements.

Native SDK types remain inside platform adapter crates.

---

## Consequences

### Positive

- stable cross-platform semantics;
- test doubles;
- contained unsafe/FFI code;
- easier platform replacement;
- reduced conditional compilation in domain code.

### Negative

- adapter design effort;
- some platform-specific capabilities require extension points;
- risk of over-generalizing interfaces.

---

## Alternatives Considered

### Platform conditionals throughout the engine

Rejected because platform behavior and native types would leak across subsystem boundaries.

---

## Related Specifications

- `05-platform/500-platform-overview.md`
- `05-platform/505-platform-capture-abstraction.md`
