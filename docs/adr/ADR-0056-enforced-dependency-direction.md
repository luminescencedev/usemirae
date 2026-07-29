# ADR-0056 — Enforced Dependency Direction

**Status:** Proposed  
**Date:** 2026-07-29

## Context

A large native application can quickly become coupled through direct subsystem access, platform checks, and toolkit types.

## Decision

Mirae will enforce inward-owned interfaces and one-way dependency direction through workspace structure, architecture tests, and CI.

## Consequences

### Positive

- replaceable implementations;
- testability;
- platform isolation;
- contained native dependencies;
- clearer ownership.

### Negative

- mapping and adapter code;
- up-front interface design;
- architecture tooling.

## Alternatives Considered

Convention-only dependency rules were rejected because they decay under development pressure.

## Related Specifications

- `08-development/804-dependency-rules.md`
- `05-platform/500-platform-overview.md`
