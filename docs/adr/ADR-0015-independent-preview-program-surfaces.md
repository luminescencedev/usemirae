# ADR-0015 — Independent Preview and Program Surfaces

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Operator preview and live program have different reliability, cadence, quality, overlay, resize, and lifecycle requirements.

Coupling them would allow UI behavior to affect live output.

---

## Decision

Preview and program will be modeled as independent surfaces with separate schedules, generations, quality policies, and delivery contracts.

The engine owns program state. Preview may be recreated or absent without stopping active outputs.

---

## Consequences

### Positive

- UI restart without stopping program;
- preview resizing independent from output;
- preview-only overlays;
- different quality and frame-rate policies;
- headless operation.

### Negative

- more surface management;
- additional render work when sharing is impossible;
- explicit synchronization required for transitions.

---

## Alternatives Considered

### One shared preview/program texture

Rejected because UI lifecycle and output lifecycle would be coupled.

### UI-owned preview rendering

Rejected because it would duplicate render semantics and undermine authoritative state.

---

## Related Specifications

- `02-rendering/211-preview-and-program-surfaces.md`
- `01-runtime/103-frame-scheduler.md`
