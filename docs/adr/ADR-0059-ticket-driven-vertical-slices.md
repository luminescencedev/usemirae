# ADR-0059 — Ticket-Driven Vertical Slices

**Status:** Proposed  
**Date:** 2026-07-29

## Context

Building subsystem infrastructure in isolation for a long period risks architecture that never forms a usable product.

## Decision

Implementation will be organized as small tickets that prefer end-to-end vertical slices and finish in runnable, testable states.

## Consequences

### Positive

- early integration;
- visible progress;
- frequent validation;
- lower risk;
- better AI-agent scope.

### Negative

- some temporary minimal interfaces;
- careful sequencing required;
- vertical tickets may touch several layers.

## Alternatives Considered

Completing each subsystem fully before integration was rejected because integration risk would arrive too late.

## Related Specifications

- `08-development/812-ticket-and-sprint-workflow.md`
- `08-development/813-implementation-roadmap.md`
