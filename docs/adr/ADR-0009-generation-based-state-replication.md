# ADR-0009 — Generation-Based State Replication

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

The UI and other replicas need efficient updates while the engine remains authoritative.

Events alone are insufficient to guarantee complete replica reconstruction after disconnects, duplicates, or missed messages.

---

## Decision

Authoritative domain state will use monotonically increasing generations per engine session.

Replicas synchronize through:

- full snapshots;
- generation-bounded patches;
- sequence/gap detection;
- resnapshot on incompatibility or missing history.

Semantic events remain separate.

---

## Consequences

### Positive

- reliable reconnect;
- explicit stale-state detection;
- efficient incremental updates;
- clean optimistic conflict handling;
- easier testing.

### Negative

- snapshot and patch schemas required;
- retained patch history must be bounded;
- replicas must handle resynchronization.

---

## Alternatives Considered

### Event replay only

Rejected because semantic events are not guaranteed to be a complete state reconstruction log.

### UI-owned state with command echoes

Rejected because it creates multiple authorities.

---

## Related Specifications

- `01-runtime/106-state-store.md`
- `01-runtime/108-ipc-protocol.md`
- `01-runtime/109-ui-engine-synchronization.md`
