# ADR-0021 — Segmented Recording and Crash-Safe Finalization

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

A crash or power loss can leave a single long recording container incomplete or difficult to repair.

Live production software must preserve as much recorded media as possible.

---

## Decision

Mirae recording will support crash-resilient container modes and segmentation.

The active file lifecycle will separate active writing from final publication, retain recovery metadata, and preserve partial files after failure.

---

## Consequences

### Positive

- smaller loss window;
- easier repair;
- safer long recordings;
- explicit finalization;
- better diagnostics.

### Negative

- more files or fragments;
- container-specific complexity;
- post-processing may be required for some workflows.

---

## Alternatives Considered

### One final file with no recovery metadata

Rejected because abrupt termination can destroy usability.

### Buffer recording entirely in memory

Rejected because memory use would be unbounded and unsafe.

---

## Related Specifications

- `03-media/312-recording.md`
- future `04-project/404-autosave-and-recovery.md`
