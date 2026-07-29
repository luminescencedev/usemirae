# ADR-0045 — Accessibility as a Release Requirement

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Adding accessibility after the UI architecture is complete leads to inaccessible custom controls, broken focus, and expensive redesign.

---

## Decision

Keyboard operation, semantic controls, focus behavior, contrast, scaling, reduced motion, and assistive-technology testing will be part of design and release gates.

---

## Consequences

### Positive

- broader usability;
- better keyboard workflows for all operators;
- fewer late redesigns;
- higher UI quality.

### Negative

- additional design and test effort;
- constraints on custom interaction;
- manual testing required.

---

## Alternatives Considered

### Accessibility audit after feature completion

Rejected because structural problems are expensive to fix late.

---

## Related Specifications

- `06-quality/613-accessibility.md`
- `06-quality/616-release-quality-gates.md`
