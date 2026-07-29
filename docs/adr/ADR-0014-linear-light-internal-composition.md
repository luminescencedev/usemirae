# ADR-0014 — Linear-Light Internal Composition

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Blending and many effects are mathematically incorrect when applied directly to gamma-encoded or ambiguously encoded source values.

Mixed SDR, HDR, YUV, graphics, and alpha sources require a defined internal color model.

---

## Decision

Mirae will convert source imagery into an explicit linear-light working space before normal composition and effect operations that require linear values.

Surface-specific transfer functions, gamut conversion, tone mapping, and range encoding occur at defined boundaries.

---

## Consequences

### Positive

- correct blending;
- consistent mixed-source behavior;
- explicit SDR/HDR pipeline;
- fewer alpha fringes;
- accurate output metadata.

### Negative

- conversion cost;
- higher precision intermediates;
- additional metadata requirements;
- more complex preview handling.

---

## Alternatives Considered

### Blend in source-encoded space

Rejected because results are incorrect and source-dependent.

### Fixed SDR-only pipeline

Rejected because it would constrain HDR and mixed workflows.

---

## Related Specifications

- `02-rendering/204-compositor.md`
- `02-rendering/208-color-management.md`
