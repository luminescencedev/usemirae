# ADR-0029 — Single-Writer Project Lock

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Two Mirae instances writing the same project could overwrite committed work even when each uses atomic saves.

---

## Decision

A project may have one active writer.

Other instances may open read-only, request handoff, or create a copy.

Lock ownership uses session-scoped random tokens and liveness checks, not PID alone.

---

## Consequences

### Positive

- prevents silent concurrent overwrite;
- clear multi-instance behavior;
- supports read-only inspection;
- compatible with external modification detection.

### Negative

- stale lock recovery;
- network filesystem limitations;
- handoff UX and IPC.

---

## Alternatives Considered

### Last writer wins

Rejected because it silently loses work.

### Merge arbitrary project files automatically

Rejected because semantic conflict resolution is not currently defined.

---

## Related Specifications

- `04-project/409-project-locking-and-multi-instance.md`
- `04-project/403-persistence.md`
